package de.unipassau.testify.mir;

import java.io.StringWriter;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Set;
import org.jgrapht.Graph;
import org.jgrapht.graph.DefaultDirectedGraph;
import org.jgrapht.graph.DefaultEdge;
import org.jgrapht.nio.Attribute;
import org.jgrapht.nio.DefaultAttribute;
import org.jgrapht.nio.dot.DOTExporter;
import org.json.JSONObject;

public class CDG {
  private final Graph<BasicBlock, DefaultEdge> graph;

  public CDG(Graph<BasicBlock, DefaultEdge> graph) {
    this.graph = graph;
  }

  // {"nodes":[18446744073709551615,0,1,2],"node_holes":[],"edge_property":"directed","edges":[[0,1,1],[0,2,1],[0,3,1],[0,0,1]]}
  public static CDG parse(int globalId, String input) {
    Graph<BasicBlock, DefaultEdge> graph = new DefaultDirectedGraph<>(DefaultEdge.class);
    var root = new JSONObject(input);


    Map<Integer, BasicBlock> nodesMap = new HashMap<>();
    var nodes = root.getJSONArray("nodes");
    for (int i = 0; i < nodes.length(); i++) {
      var block = new BasicBlock(globalId, nodes.getInt(i));
      nodesMap.put(i, block);
      graph.addVertex(block);
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
    DOTExporter<BasicBlock, DefaultEdge> exporter = new DOTExporter<>();
    var writer = new StringWriter();
    exporter.setVertexAttributeProvider(v -> {
      Map<String, Attribute> map = new LinkedHashMap<>();
      map.put("label", DefaultAttribute.createAttribute(v.toString()));
      return map;
    });
    exporter.exportGraph(graph, writer);
    return writer.toString();
  }

  public Set<BasicBlock> targets() {
    return graph.vertexSet();
  }

  public static void main(String[] args) {
    var cdg = parse(205, "{\"nodes\":[42069,1,3,4,5,6,7,8,9,10,11,12,13,14,15,2,0,16],\"node_holes\":[],\"edge_property\":\"directed\",\"edges\":[[1,2,1],[1,3,1],[1,4,1],[1,5,1],[1,6,1],[1,7,1],[1,8,1],[1,9,1],[1,10,1],[1,11,1],[1,12,1],[1,13,1],[1,14,1],[1,15,1],[0,16,1],[0,1,1],[0,17,1],[0,0,1]]}");
    System.out.println(cdg.toDot());
  }
}