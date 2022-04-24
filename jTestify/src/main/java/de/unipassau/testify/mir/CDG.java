package de.unipassau.testify.mir;

import com.google.common.base.Preconditions;
import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import java.io.StringWriter;
import java.util.ArrayList;
import java.util.Collections;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.stream.Collectors;
import org.jgrapht.Graph;
import org.jgrapht.Graphs;
import org.jgrapht.alg.shortestpath.DijkstraShortestPath;
import org.jgrapht.graph.DefaultDirectedGraph;
import org.jgrapht.graph.DefaultEdge;
import org.jgrapht.nio.Attribute;
import org.jgrapht.nio.DefaultAttribute;
import org.jgrapht.nio.dot.DOTExporter;
import org.json.JSONObject;

public class CDG<M extends MinimizingFitnessFunction<C>, C extends AbstractTestCaseChromosome<C>> {

  private final Graph<M, DefaultEdge> graph;
  private final Map<M, List<M>> cache;

  public CDG(Graph<M, DefaultEdge> graph) {
    this.graph = graph;
    this.cache = new HashMap<>();
  }

  // {"nodes":[18446744073709551615,0,1,2],"node_holes":[],"edge_property":"directed","edges":[[0,1,1],[0,2,1],[0,3,1],[0,0,1]]}
  public static <M extends MinimizingFitnessFunction<C>, C extends AbstractTestCaseChromosome<C>> CDG<M, C> parse(String globalId,
      String input) {
    Graph<BasicBlock, DefaultEdge> graph = new DefaultDirectedGraph<>(DefaultEdge.class);
    var root = new JSONObject(input);

    Map<Integer, BasicBlock> nodesMap = new HashMap<>();
    var nodes = root.getJSONArray("nodes");
    for (int i = 0; i < nodes.length(); i++) {
      var target = new BasicBlock(globalId, nodes.getInt(i));
      nodesMap.put(i, target);
      graph.addVertex(target);
    }

    var edges = root.getJSONArray("edges");
    for (int i = 0; i < edges.length(); i++) {
      var edge = edges.getJSONArray(i);
      var from = nodesMap.get(edge.getInt(0));
      var to = nodesMap.get(edge.getInt(1));

      graph.addEdge(from, to);
    }

    return new CDG(graph);
  }

  public String toDot() {
    DOTExporter<M, DefaultEdge> exporter = new DOTExporter<>();
    var writer = new StringWriter();
    exporter.setVertexAttributeProvider(v -> {
      Map<String, Attribute> map = new LinkedHashMap<>();
      map.put("label", DefaultAttribute.createAttribute(v.toString()));
      return map;
    });
    exporter.exportGraph(graph, writer);
    return writer.toString();
  }

  public Set<M> targets() {
    return graph.vertexSet();
  }

  public Set<M> independentTargets() {
    var root = graph.vertexSet().stream().filter(MinimizingFitnessFunction::isDummy).findAny()
        .get();
    return Graphs.neighborSetOf(graph, root);
  }

  public Set<M> dependentTargets(M target) {
    return graph.outgoingEdgesOf(target).stream().map(graph::getEdgeTarget)
        .collect(Collectors.toSet());
  }

  public Set<M> allSubTargets(M basicBlock) {
    var subTargets = new HashSet<M>();
    allSubTargets(basicBlock, subTargets);
    return subTargets;
  }

  private void allSubTargets(M target, Set<M> subTargets) {
    graph.outgoingEdgesOf(target)
        .stream()
        .filter(e -> !graph.getEdgeTarget(e).equals(target))
        .forEach(e -> {
          var subTarget = graph.getEdgeTarget(e);
          subTargets.add(subTarget);
          allSubTargets(subTarget, subTargets);
        });
  }

  private M root() {
    var root = graph.vertexSet().stream().filter(MinimizingFitnessFunction::isDummy).toList();
    Preconditions.checkState(root.size() == 1);
    return root.get(0);
  }

  /**
   * Returns the path to the given target in the CDG including the given target. The path a
   * linear ordered collection of blocks in the CDG starting from root.
   *
   * @param target The target to find the path for.
   * @return The path in the CDG.
   */
  public List<M> pathTo(M target) {
    if (cache.containsKey(target)) {
      return cache.get(target);
    }

    var path = DijkstraShortestPath.findPathBetween(graph, root(), target).getVertexList();
    cache.put(target, path);

    return path;
  }

  /**
   * Returns the set of parents of the given node without self references.
   *
   * @param target The target to find parents of.
   * @return The set of parents.
   */
  public Set<M> realParents(M target) {
    var edges = graph.incomingEdgesOf(target);
    return edges.stream().map(graph::getEdgeSource).filter(s -> !s.equals(target)).collect(Collectors.toSet());
  }

  public static void main(String[] args) {
    var cdg = parse("hello",
        "{\"nodes\":[42069,1,3,4,5,6,7,8,9,10,11,12,13,14,15,2,0,16],\"node_holes\":[],\"edge_property\":\"directed\",\"edges\":[[1,2,1],[1,3,1],[1,4,1],[1,5,1],[1,6,1],[1,7,1],[1,8,1],[1,9,1],[1,10,1],[1,11,1],[1,12,1],[1,13,1],[1,14,1],[1,15,1],[0,16,1],[0,1,1],[0,17,1],[0,0,1]]}");
    System.out.println(cdg.independentTargets());
  }

  @Override
  public String toString() {
    return toDot();
  }
}
