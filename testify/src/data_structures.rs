use petgraph::algo::dominators::{simple_fast};
use petgraph::dot::Dot;
use petgraph::graph::NodeIndex;
use petgraph::{Direction, Graph};
use rustc_data_structures::graph::WithSuccessors;
use rustc_middle::mir::{BasicBlock, Body, TerminatorKind};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::Write;
use std::iter::{FromIterator};
use std::path::Path;
use std::process::Command;
use log::{debug, error};
use rustc_ast::ptr::P;
use rustc_middle::ty::layout::MaybeResult;
use crate::DOT_DIR;

pub const ENTRY: usize = 42069;

pub fn truncated_cfg(body: &Body<'_>) -> (Graph<usize, usize>, HashMap<BasicBlock, NodeIndex>) {
  let mut graph = Graph::new();
  let mut table = HashMap::new();

  let basic_blocks: Vec<BasicBlock> = body
      .basic_blocks()
      .iter_enumerated()
      .map(|(block, data)| block)
      .collect();

  let mut queue = VecDeque::new();
  let bb0 = basic_blocks.first().unwrap();
  // Insert it as is, there may be methods with only one basic block
  table.entry(*bb0).or_insert_with(|| graph.add_node(bb0.as_usize()));

  successors(body, *bb0)
      .iter()
      .map(|successor| (*bb0, *successor))
      .for_each(|pair| {
        queue.push_back(pair);
      });
  while !queue.is_empty() {
    let next = queue.pop_front();
    if let Some((from, to)) = next {
      let from_index = *table
          .entry(from)
          .or_insert_with(|| graph.add_node(from.as_usize()));
      let to_index = *table
          .entry(to)
          .or_insert_with(|| graph.add_node(to.as_usize()));

      let edge_already_exists = graph
          .edges_connecting(from_index, to_index)
          .peekable()
          .peek()
          .is_some();

      if !edge_already_exists {
        graph.add_edge(from_index, to_index, 1);
        successors(body, to)
            .iter()
            .map(|successor| (to, *successor))
            .for_each(|pair| {
              queue.push_back(pair);
            });
      }
    }
  }

  // There might be cases where the cfg has multiple exits, e.g., with an explicit panic!
  // Thus, we let all possible exits point to an artificial exit node
  let mut exit_nodes = vec![];
  for node in graph.node_indices() {
    let outgoing_edges = graph.edges_directed(node, Direction::Outgoing);
    if outgoing_edges.count() == 0 {
      exit_nodes.push(node);
    }
  }

  let absolute_exit = graph.add_node(ENTRY);

  for exit_node in exit_nodes {
    graph.add_edge(exit_node, absolute_exit, 1);
  }

  (graph, table)
}

pub fn original_cfg(body: &Body<'_>) -> (Graph<usize, usize>, HashMap<BasicBlock, NodeIndex>) {
  let mut graph = Graph::new();
  let mut table = HashMap::new();

  let basic_blocks: Vec<BasicBlock> = body
      .basic_blocks()
      .iter_enumerated()
      .map(|(block, data)| block)
      .collect();

  for block in &basic_blocks {
    let index = *table
        .entry(*block)
        .or_insert_with(|| graph.add_node(block.as_usize()));
    for successor in body.successors(*block) {
      let successor_index = *table
          .entry(successor)
          .or_insert_with(|| graph.add_node(successor.as_usize()));
      graph.add_edge(index, successor_index, 1);
    }
  }

  (graph, table)
}

pub fn successors<'tcx>(body: &'tcx Body<'tcx>, block: BasicBlock) -> Vec<BasicBlock> {
  let terminator = body.basic_blocks().get(block).unwrap().terminator();

  if let TerminatorKind::Unreachable = &terminator.kind {
    let block_data = body.basic_blocks().get(block).unwrap();
    let stmts = &block_data.statements;
    assert!(stmts.is_empty());
  }

  successors_of_terminator_kind(body, &terminator.kind)
}

pub fn successors_of_terminator_kind<'tcx>(
  body: &'tcx Body<'tcx>,
  kind: &'tcx TerminatorKind<'tcx>,
) -> Vec<BasicBlock> {
  use self::TerminatorKind::*;
  match *kind {
    Resume
    | Abort
    | GeneratorDrop
    | Return
    | Unreachable
    | Call {
      destination: None,
      cleanup: None,
      ..
    }
    | InlineAsm {
      destination: None, ..
    } => vec![],
    Goto { target: ref t }
    | Call {
      destination: None,
      cleanup: Some(ref t),
      ..
    }
    | Call {
      destination: Some((_, ref t)),
      cleanup: None,
      ..
    }
    | Yield {
      resume: ref t,
      drop: None,
      ..
    }
    | DropAndReplace {
      target: ref t,
      unwind: None,
      ..
    }
    | Drop {
      target: ref t,
      unwind: None,
      ..
    }
    | Assert {
      target: ref t,
      cleanup: None,
      ..
    }
    | FalseUnwind {
      real_target: ref t,
      unwind: None,
    }
    | InlineAsm {
      destination: Some(ref t),
      ..
    } => {
      if is_reachable(body, *t) {
        vec![*t]
      } else {
        vec![]
      }
    }
    Call {
      destination: Some((_, ref t)),
      cleanup: Some(ref u),
      ..
    }
    | Yield {
      resume: ref t,
      drop: Some(ref u),
      ..
    }
    | DropAndReplace {
      target: ref t,
      unwind: Some(ref u),
      ..
    }
    | Drop {
      target: ref t,
      unwind: Some(ref u),
      ..
    }
    | Assert {
      target: ref t,
      cleanup: Some(ref u),
      ..
    }
    | FalseUnwind {
      real_target: ref t,
      unwind: Some(ref u),
    } => {
      if is_reachable(body, *t) {
        vec![*t]
      } else {
        vec![]
      }
    }
    SwitchInt { ref targets, .. } => {
      let targets = targets
          .all_targets()
          .iter()
          .filter_map(|t| {
            if is_reachable(body, *t) {
              Some(*t)
            } else {
              None
            }
          }).collect::<Vec<_>>();
      targets
    }
    FalseEdge {
      ref real_target,
      ref imaginary_target,
    } => {
      // TODO prolly also check for reachability?
      vec![*real_target, *imaginary_target]
    }
  }
}

fn is_reachable<'tcx>(body: &'tcx Body<'tcx>, block: BasicBlock) -> bool {
  let block_data = body.basic_blocks().get(block).unwrap();
  let terminator = block_data.terminator.as_ref();
  if let Some(terminator) = terminator {
    if let TerminatorKind::Unreachable = &terminator.kind {
      return false;
    }
  }
  true
}

/*
pub fn cfg(body: &Body<'_>) -> HashMap<usize, HashSet<usize>> {
    let mut cfg = HashMap::new();
    let basic_blocks: Vec<BasicBlock> = body
        .basic_blocks()
        .iter_enumerated()
        .map(|(block, data)| block)
        .collect();
    for block in &basic_blocks {
        let successors: HashSet<usize> = body
            .successors(*block)
            .map(|block| block.as_usize())
            .collect();
        cfg.insert(block.as_usize(), successors);
    }

    cfg
}*/

pub fn post_dominators(body: &Body<'_>) -> HashMap<usize, HashSet<usize>> {
  let basic_blocks = body.basic_blocks();
  let basic_blocks: Vec<BasicBlock> = basic_blocks
      .iter_enumerated()
      .map(|(block, data)| block)
      .rev()
      .collect();

  let &entry = basic_blocks.first().unwrap();
  let mut dominators = HashMap::new();

  let mut entry_hash_set = HashSet::new();
  entry_hash_set.insert(entry);
  dominators.insert(entry, entry_hash_set);
  for block in &basic_blocks[1..] {
    dominators.insert(*block, HashSet::from_iter(basic_blocks.iter().cloned()));
  }

  let mut changed = true;
  while changed {
    changed = false;
    for block in &basic_blocks[1..] {
      let current_dominators = dominators.get(block).unwrap();
      let predecessors = body.successors(*block);
      let mut new_dominators = predecessors.map(|p| dominators.get(&p).unwrap()).fold(
        HashSet::from_iter(basic_blocks.iter().cloned()),
        |acc, x| acc.intersection(x).cloned().collect(),
      );
      new_dominators.insert(*block);
      if current_dominators != &new_dominators {
        dominators.entry(*block).and_modify(|e| *e = new_dominators);
        changed = true;
      }
    }
  }

  let mut dominator_ids = HashMap::new();
  for (block, dominators) in &dominators {
    let dominators = dominators.iter().map(|d| d.as_usize()).collect();
    dominator_ids.insert(block.as_usize(), dominators);
  }

  dominator_ids
}

pub fn immediate_post_dominators(
  post_dominators: &HashMap<usize, HashSet<usize>>,
) -> Graph<usize, usize> {
  let mut ipd = post_dominators.clone();
  for (n, d) in ipd.iter_mut() {
    // Make strict
    d.remove(n);

    let to_remove: Vec<usize> = d
        .iter()
        .map(|p| {
          let to_remove = d
              .iter()
              .filter(|&k| p != k && is_postdominated_by(*p, *k, post_dominators))
              .map(|k| *k)
              .collect::<Vec<_>>();
          to_remove
        })
        .flatten()
        .collect();

    d.retain(|pd| !to_remove.contains(&pd));
  }

  map_to_graph(ipd)
}

fn map_to_graph(m: HashMap<usize, HashSet<usize>>) -> Graph<usize, usize> {
  let mut graph = Graph::new();
  let mut table = HashMap::new();
  for (block, successors) in m.iter() {
    let index = *table
        .entry(*block)
        .or_insert_with(|| graph.add_node(*block));
    for successor in successors {
      let successor_index = *table
          .entry(*successor)
          .or_insert_with(|| graph.add_node(*successor));
      graph.add_edge(successor_index, index, 1);
    }
  }

  graph
}

pub fn visualize_graph<A: Display, B: Display>(graph: &Graph<A, B>, global_id: &str) {
  let dot_path = Path::new(DOT_DIR).join(format!("{}.dot", global_id));
  let png_path = Path::new(DOT_DIR).join(format!("{}.png", global_id));
  if let Ok(_) = log_graph_to(graph, dot_path.as_path()) {
    Command::new("dot")
        .arg("-Tpng")
        .arg(dot_path.to_str().unwrap())
        .arg("-o")
        .arg(png_path.to_str().unwrap())
        .output()
        .expect(&format!("Could not store {}", png_path.to_str().unwrap()));
  } else {
    error!("Could not write graph DOT file {}", dot_path.to_str().unwrap());
  }
}

pub fn log_graph_to<A: Display, B: Display, P>(graph: &Graph<A, B>, path: P) -> std::io::Result<()>
  where
      P: AsRef<std::path::Path>
{
  let dot = Dot::new(graph);
  let mut file = OpenOptions::new()
      .write(true)
      .truncate(true)
      .create(true)
      .open(path)
      .unwrap();
  let output = format!("{}", dot);
  file.write_all(output.as_bytes())
}

/// A graph with nodes that are identified by basic block ids
pub fn cdg(cfg: &Graph<usize, usize>) -> Graph<usize, usize> {
  let mut reversed_cfg = cfg.clone();
  reversed_cfg.reverse();

  let root = cfg
      .node_indices()
      .find(|i| cfg.neighbors(*i).peekable().peek().is_none())
      .unwrap();
  // println!("Root is {}", cfg.node_weight(root).unwrap());
  let dominators = simple_fast(&reversed_cfg, root);

  let cfg_edges = cfg
      .edge_indices()
      .filter(|edge| {
        // Find edges from a to b where b is not a strict dominator of a
        let (a, b) = cfg.edge_endpoints(*edge).unwrap();
        if let Some(mut dominators) = dominators.strict_dominators(a) {
          dominators.find(|d| d == &b).is_none()
        } else {
          true
        }
      })
      .collect::<Vec<_>>();

  let mut cdg = Graph::new();
  let mut cdg_table = HashMap::new();
  let entry_index = *cdg_table
      .entry(ENTRY)
      .or_insert_with(|| cdg.add_node(ENTRY));

  let mut dependent_nodes = HashSet::new();
  for edge in &cfg_edges {
    let (a, b) = cfg.edge_endpoints(*edge).unwrap();
    let a_dominators = dominators.dominators(a).unwrap().collect::<Vec<_>>();
    let b_dominators = dominators.dominators(b).unwrap().collect::<Vec<_>>();

    let lca = lca(&a_dominators, &b_dominators).unwrap();

    let a_name = *cfg.node_weight(a).unwrap();
    let a_index = *cdg_table
        .entry(a_name)
        .or_insert_with(|| cdg.add_node(a_name));
    dominators
        .dominators(b)
        .unwrap()
        .take_while(|d| d != &lca)
        .for_each(|b_d| {
          let b_name = *cfg.node_weight(b_d).unwrap();
          let b_index = *cdg_table
              .entry(b_name)
              .or_insert_with(|| cdg.add_node(b_name));
          cdg.add_edge(a_index, b_index, 1usize);
          dependent_nodes.insert(b_name);
        });

    if lca == a {
      let lca_name = *cfg.node_weight(lca).unwrap();
      let lca_index = *cdg_table
          .entry(lca_name)
          .or_insert_with(|| cdg.add_node(lca_name));
      cdg.add_edge(a_index, lca_index, 1usize);
      dependent_nodes.insert(lca_name);
    }
  }

  println!("{}", Dot::new(&cdg));

  cfg.node_indices()
      .filter_map(|n| {
        let name = cfg.node_weight(n).unwrap();
        if dependent_nodes.contains(name) {
          None
        } else {
          Some(*name)
        }
      })
      .for_each(|n| {
        let index = *cdg_table.entry(n).or_insert_with(|| cdg.add_node(n));
        cdg.add_edge(entry_index, index, 1usize);
      });

  cdg
}

pub fn lca<T: PartialEq + Copy>(a_seq: &[T], b_seq: &[T]) -> Option<T> {
  for a_d in a_seq {
    for b_d in b_seq {
      if a_d == b_d {
        return Some(*a_d);
      }
    }
  }
  None
}

pub fn edges(tree: HashMap<usize, HashSet<usize>>) -> HashSet<(usize, usize)> {
  let mut edges = HashSet::new();

  for (parent, children) in tree.iter() {
    for child in children {
      edges.insert((*parent, *child));
    }
  }
  edges
}

fn is_postdominated_by(a: usize, b: usize, tree: &HashMap<usize, HashSet<usize>>) -> bool {
  if let Some(post_dominators) = tree.get(&a) {
    if post_dominators.contains(&b) {
      // Immediate post-dominator
      true
    } else {
      /*// Non-immediate post-dominator
      post_dominators
          .iter()
          .filter_map(|pd| {
              if let Some(post_dominators) = tree.get(pd) {
                  Some(post_dominators)
              } else {
                  None
              }
          })
          .flatten()
          .any(|pd| is_postdominated_by(a, *pd, tree))*/
      false
    }
  } else {
    false
  }
}

#[cfg(test)]
mod tests {
  use rustc_data_structures::vec_linked_list::iter;
  use super::*;

  #[test]
  fn test_is_postdominated_by_immediate_pd() {
    let pdt: HashMap<usize, HashSet<usize>> = serde_json::from_str("{\"0\":[2,1,0,8],\"5\":[8,5],\"2\":[2,8],\"8\":[8],\"7\":[8,7],\"6\":[7,6,8],\"1\":[8,1,2],\"3\":[5,4,3,8],\"4\":[4,5,8]}").unwrap();

    assert!(is_postdominated_by(0, 2, &pdt));
  }

  #[test]
  fn test_cdg() {
    let mut cfg = Graph::new();
    let edges: [(usize, usize); 2] = [(1, 2), (2, 2)];
    let mut indexes = HashMap::new();
    edges.iter().for_each(|(from, to)| {
      indexes.entry(from).or_insert_with(|| cfg.add_node(from));
      indexes.entry(to).or_insert_with(|| cfg.add_node(to));
    });
    edges.iter().for_each(|(from, to)| {
      cfg.add_edge(*indexes.get(from).unwrap(), *indexes.get(to).unwrap(), 1);
    })
  }
}
