package de.unipassau.testify.server;

import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
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

  public static <C extends AbstractTestCaseChromosome<C>> Map<Integer, Map<MinimizingFitnessFunction<C>, Double>> requestTraces() {
    Map<Integer, Map<MinimizingFitnessFunction<C>, Double>> coverage = new HashMap<>();
    Set<String> traces = jedis.smembers("traces");

    for (String trace : traces) {
      var result = TraceParser.<C>parse(trace);

      if (result == null) {
        continue;
      }

      var testId = result.getValue0();
      coverage.putIfAbsent(testId, new HashMap<>());

      var basicBlock = result.getValue1();
      var distance = result.getValue2();

      // Take the minimal distance for a given basic block
      coverage.get(testId)
          .compute(basicBlock, (k, v) -> (v == null) ? distance : Double.min(distance, v));
    }

    return coverage;
  }

  public static void clear() {
    jedis.del("traces");
  }

  public static void main(String[] args) {
    var coverage = RedisStorage.requestTraces();
    System.out.println(coverage);
  }
}
