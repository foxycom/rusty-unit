use petgraph::algo::dominators::{simple_fast, Dominators};
use petgraph::algo::has_path_connecting;
use petgraph::graph::NodeIndex;
use petgraph::visit::{GraphBase, IntoNeighbors, Reversed, Visitable};
use petgraph::Graph;
use rustc_data_structures::graph::WithSuccessors;
use rustc_middle::mir::{BasicBlock, Body};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

pub const ENTRY: usize = usize::MAX;

pub fn cfg(body: &Body<'_>) -> (Graph<usize, usize>, HashMap<BasicBlock, NodeIndex>) {
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

pub fn predecessors(graph: Graph<usize, usize>, node: NodeIndex) -> Vec<NodeIndex> {
    todo!()
}

pub fn cdg(body: &Body<'_>) -> Graph<usize, usize> {
    let (cfg, cfg_table) = cfg(body);
    let mut reversed_cfg = cfg.clone();
    reversed_cfg.reverse();

    // TODO this may be not always the case
    let exit_block = body.basic_blocks().last().unwrap();
    let root = cfg_table.get(&exit_block).unwrap();
    let dominators = simple_fast(&reversed_cfg, *root);

    //let post_dominators = post_dominators(body);
    //let immediate_post_dominators = immediate_post_dominators(&post_dominators);

    let cfg_edges = cfg
        .edge_indices()
        .filter(|edge| {
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
    let entry_index = *cdg_table.entry(ENTRY).or_insert_with(|| cdg.add_node(ENTRY));

    let mut dependent_nodes = HashSet::new();
    for edge in &cfg_edges {
        let (a, b) = cfg.edge_endpoints(*edge).unwrap();
        let a_dominators = dominators.dominators(a).unwrap().collect::<Vec<_>>();
        let b_dominators = dominators.dominators(b).unwrap().collect::<Vec<_>>();

        let lca = lca(&a_dominators, &b_dominators).unwrap();

        let a_name = *cfg.node_weight(a).unwrap();
        // Insert a
        let a_index = *cdg_table.entry(a_name).or_insert_with(|| cdg.add_node(a_name));
        dominators.dominators(b).unwrap().take_while(|d| d != &lca).for_each(|b_d| {
            let b_name = *cfg.node_weight(b_d).unwrap();
            let b_index = *cdg_table.entry(b_name).or_insert_with(|| cdg.add_node(b_name));
            cdg.add_edge(a_index, b_index, 1usize);
            dependent_nodes.insert(b_name);
        });

        if lca == a {
            let lca_name = *cfg.node_weight(lca).unwrap();
            let lca_index = *cdg_table.entry(lca_name).or_insert_with(|| cdg.add_node(lca_name));
            cdg.add_edge(a_index, lca_index, 1usize);
            dependent_nodes.insert(lca_name);
        }
    }

    cfg.node_indices().filter_map(|n| {
        let name = cfg.node_weight(n).unwrap();
        if dependent_nodes.contains(name) {
            None
        } else {
            Some(*name)
        }
    }).for_each(|n| {
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

pub fn depth_map(tree: &HashMap<usize, HashSet<usize>>) -> HashMap<usize, usize> {
    let mut depth_map = HashMap::new();
    let entry = 0usize;

    depth_map_inner(0, entry, tree, &mut depth_map);

    depth_map
}

fn depth_map_inner(
    depth: usize,
    block: usize,
    tree: &HashMap<usize, HashSet<usize>>,
    depth_map: &mut HashMap<usize, usize>,
) {
    depth_map.insert(block, depth);
    if let Some(children) = tree.get(&block) {
        for child in children {
            depth_map_inner(depth + 1, *child, tree, depth_map);
        }
    }
}

pub fn least_common_ancestor_of(
    a: usize,
    b: usize,
    tree: &HashMap<usize, HashSet<usize>>,
    depth_map: &HashMap<usize, usize>,
) -> usize {
    let a_ancestors = ancestors_of(a, tree);
    let b_ancestors = ancestors_of(b, tree);
    todo!()
}

pub fn ancestors_of(a: usize, tree: &HashMap<usize, HashSet<usize>>) -> Vec<usize> {
    // TODO what about cyclic dependencies?
    //let mut ancestors = vec![];
    todo!()
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
    use super::*;
    #[test]
    fn test_is_postdominated_by_immediate_pd() {
        let pdt: HashMap<usize, HashSet<usize>> = serde_json::from_str("{\"0\":[2,1,0,8],\"5\":[8,5],\"2\":[2,8],\"8\":[8],\"7\":[8,7],\"6\":[7,6,8],\"1\":[8,1,2],\"3\":[5,4,3,8],\"4\":[4,5,8]}").unwrap();

        assert!(is_postdominated_by(0, 2, &pdt));
    }

    #[test]
    fn smoke_test_tree_from_map() {
        let map: HashMap<usize, HashSet<usize>> = serde_json::from_str("{\"0\":[1],\"4\":[5],\"8\":[],\"1\":[2],\"3\":[4],\"6\":[7],\"7\":[8],\"2\":[3,6],\"5\":[8]}").unwrap();

        let tree = Graph::from_map(map);
        println!("{}", serde_json::to_string(&tree).unwrap());
    }
}
