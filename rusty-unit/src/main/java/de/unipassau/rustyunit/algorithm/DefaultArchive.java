package de.unipassau.rustyunit.algorithm;

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

public class DefaultArchive<C extends AbstractTestCaseChromosome<C>> implements Archive<C> {

  private final Set<C> archive;
  private final Set<MinimizingFitnessFunction<C>> objectives;

  private final Map<MinimizingFitnessFunction<C>, Boolean> coveredObjectives;

  public DefaultArchive(Set<MinimizingFitnessFunction<C>> objectives) {
    this.archive = new HashSet<>();
    this.objectives = objectives;
    this.coveredObjectives = new HashMap<>();
  }

  @Override
  public void update(List<C> population) {
    int nCovered = 0;
    for (var u : objectives) {
      boolean covered = false;
      C bestTestCase;
      var bestLength = Integer.MAX_VALUE;
      if ((bestTestCase = getCaseThatCovers(u)) != null) {
        bestLength = bestTestCase.size();
      }

      for (var testCase : population) {
        Objects.requireNonNull(testCase);
        var score = u.getFitness(testCase);
        var length = testCase.size();
        if (score == 0 && length <= bestLength && !testCase.metadata().fails()) {
          if (!covered) {
            coveredObjectives.put(u, true);
            nCovered++;
            covered = true;
          }
          bestTestCase = testCase;
          bestLength = length;
        }
      }

      if (bestTestCase != null) {
        archive.add(bestTestCase);
      }
    }

    long coverage = coveredObjectives.values().stream().filter(v -> v).count();
    double percent = ((double) coverage / objectives.size()) * 100;
    System.out.printf("\t>> Covered %d targets out of %d (%.2f%%)%n", coverage, objectives.size(), percent);
    for (var objective : objectives) {
      System.out.printf("%s is %s%n", objective, coveredObjectives.containsKey(objective));
    }
    System.out.printf("\t>> Archive contains %d tests%n", archive.size());
  }

  @Override
  public C getCaseThatCovers(FitnessFunction<C> objective) {
    var result = archive.stream().filter(t -> objective.getFitness(t) == 0.0).findFirst();
    return result.orElse(null);
  }

  @Override
  public void replaceBy(C origin, C by) {
    if (origin != null) {
      archive.remove(origin);
    }
    archive.add(by);

  }

  @Override
  public List<C> get() {
    return new ArrayList<>(archive);
  }
}
