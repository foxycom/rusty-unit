package de.unipassau.rustyunit.algorithm;

import de.unipassau.rustyunit.exec.TestCaseRunner;
import de.unipassau.rustyunit.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.rustyunit.metaheuristics.fitness_functions.FitnessFunction;
import de.unipassau.rustyunit.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.stream.Collectors;
import org.javatuples.Pair;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class DefaultArchive<C extends AbstractTestCaseChromosome<C>> implements Archive<C> {
  private static final Logger logger = LoggerFactory.getLogger(DefaultArchive.class);

  private final Set<MinimizingFitnessFunction<C>> objectives;
  private final Map<MinimizingFitnessFunction<C>, C> coveredObjectives;
  private final Map<MinimizingFitnessFunction<C>, Double> fitness;

  private final Stats stats;
  public class StatsImpl implements Stats {
    @Override
    public int coveredTargets() {
      return coveredObjectives.keySet().size();
    }

    @Override
    public double fitness() {
      double sum = 0;
      for (Double value : fitness.values()) {
        sum += value;
      }

      return sum;
    }

    @Override
    public double coverage() {
      return ((double) coveredObjectives.keySet().size() / objectives.size()) * 100;
    }
  }

  public DefaultArchive(Set<MinimizingFitnessFunction<C>> objectives) {
    this.objectives = objectives;
    this.coveredObjectives = new HashMap<>();
    this.fitness = new HashMap<>();
    this.stats = new StatsImpl();
  }

  @Override
  public void update(List<C> population) {
    for (var u : objectives) {
      C bestTestCase;
      var bestLength = Integer.MAX_VALUE;
      if ((bestTestCase = coveredObjectives.get(u)) != null) {
        bestLength = bestTestCase.size();
      }
      double minScore = Double.MAX_VALUE;
      for (var testCase : population) {
        Objects.requireNonNull(testCase);
        var score = u.getFitness(testCase);
        if (score < minScore) {
          minScore = score;
        }
        var length = testCase.size();
        if (score == 0 && length <= bestLength) {
          coveredObjectives.put(u, testCase);
          bestLength = length;
        }
      }
      if (minScore < Double.MAX_VALUE) {
        fitness.put(u, minScore);
      }
    }

    long coverage = coveredObjectives.keySet().size();
    double percent = ((double) coverage / objectives.size()) * 100;
    logger.info("\t>> Covered {} targets out of {} ({}%)", coverage, objectives.size(), percent);
    logger.info("\t>> Archive contains {} tests", new HashSet<>(coveredObjectives.values()).size());
  }

  @Override
  public Stats coverage() {
    return stats;
  }

  @Override
  public int size() {
    return new HashSet<>(coveredObjectives.values()).size();
  }

  @Override
  public int numberOfObjectives() {
    return objectives.size();
  }

  @Override
  public List<C> get() {
    return new ArrayList<>(new HashSet<>(coveredObjectives.values()));
  }
}
