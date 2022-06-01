package de.unipassau.rustyunit.mir;

import com.google.common.base.Preconditions;
import de.unipassau.rustyunit.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.rustyunit.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import java.io.StringWriter;
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
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class CDG<M extends MinimizingFitnessFunction<C>, C extends AbstractTestCaseChromosome<C>> {
  private static final Logger logger = LoggerFactory.getLogger(CDG.class);

  private final Graph<M, DefaultEdge> graph;
  private final Map<M, List<M>> pathCache;
  private final Map<M, Integer> distanceCache;
  private final Map<M, Set<M>> dependenceCache;

  private final int branches;
  private final M root;

  private double averageDepth;

  private int assertions;

  public CDG(Graph<M, DefaultEdge> graph, int branches, int assertions) {
    this.graph = graph;
    this.branches = branches;
    this.assertions = assertions;
    this.pathCache = new HashMap<>();
    this.distanceCache = new HashMap<>();
    this.dependenceCache = new HashMap<>();
    this.root = root(graph);

    int allLength = 0;
    for (M object : graph.vertexSet()) {
      var path = DijkstraShortestPath.findPathBetween(graph, root, object).getVertexList();
      allLength += (path.size() - 1);
      pathCache.put(object, path);
      distanceCache.put(object, path.size() - 1);
      dependenceCache.put(object, allSubTargets(object));
    }

    averageDepth = ((double) allLength) / graph.vertexSet().size();
  }

  // {"nodes":[18446744073709551615,0,1,2],"node_holes":[],"edge_property":"directed","edges":[[0,1,1],[0,2,1],[0,3,1],[0,0,1]]}
  public static <M extends MinimizingFitnessFunction<C>, C extends AbstractTestCaseChromosome<C>> CDG<M, C> parse(String globalId,
      String input, int branches, int assertions) {
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

    return new CDG(graph, branches, assertions);
  }

  public int assertions() {
    return assertions;
  }

  public int branches() {
    return branches;
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

  public int approachLevel(M object, Set<M> coveredObjects) {
    var path = pathTo(object);
    // Last element is object itself
    var i = path.size() - 1;
    while (i >= 0) {
      var ascendant = path.get(i);
      if (coveredObjects.contains(ascendant)) {
        break;
      }
      i--;
    }

    return path.size() - i - 1;
  }

  public Set<M> targets() {
    return graph.vertexSet().stream().filter(v -> !v.isDummy()).collect(Collectors.toSet());
  }

  public Set<M> independentTargets() {
    var root = graph.vertexSet().stream().filter(MinimizingFitnessFunction::isDummy).findAny()
        .get();
    var neighbours = Graphs.neighborSetOf(graph, root);
    neighbours.remove(root);
    return neighbours;
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

  private M root(Graph<M, DefaultEdge> graph) {
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
    var path = pathCache.get(target);
    Preconditions.checkNotNull(path);
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

  public double averageDepth() {
    return averageDepth;
  }

  @Override
  public String toString() {
    return toDot();
  }
}
