package de.unipassau.testify.server;

import de.unipassau.testify.mir.BasicBlock;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import org.javatuples.Pair;
import redis.clients.jedis.Jedis;

public class RedisStorage {

  private static final Jedis jedis = new Jedis();

  public static Map<Integer, List<Pair<BasicBlock, Double>>> requestTraces() {
    Map<Integer, List<Pair<BasicBlock, Double>>> coverage = new HashMap<>();
    Set<String> traces = jedis.smembers("traces");

    for (String trace : traces) {
      System.out.printf("Line is %s%n", trace);
      var result = TraceParser.parse(trace);
      coverage.putIfAbsent(result.getValue0(), new ArrayList<>());
      coverage.get(result.getValue0()).add(Pair.with(result.getValue1(), result.getValue2()));
    }

    return coverage;
  }

  public static void main(String[] args) {
    var coverage = RedisStorage.requestTraces();
    System.out.println(coverage);
  }
}
