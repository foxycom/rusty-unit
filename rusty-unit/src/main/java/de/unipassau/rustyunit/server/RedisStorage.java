package de.unipassau.rustyunit.server;

import de.unipassau.rustyunit.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.rustyunit.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import java.util.HashMap;
import java.util.Map;
import java.util.Set;
import redis.clients.jedis.Jedis;

public class RedisStorage {

  private static String db = "traces";
  private static int run = 0;
  private static final Jedis jedis = new Jedis();

  public static <C extends AbstractTestCaseChromosome<C>> Map<Integer, Map<MinimizingFitnessFunction<C>, Double>> requestTraces() {
    Map<Integer, Map<MinimizingFitnessFunction<C>, Double>> coverage = new HashMap<>();
    Set<String> traces = jedis.smembers(db);

    for (String trace : traces) {
      var result = TraceParser.<C>parse(run, trace);

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

  public static void setRun(int run) {
    RedisStorage.db = "traces-" + run;
    RedisStorage.run = run;
  }

  public static void clear() {
    jedis.del(db);
  }

}
