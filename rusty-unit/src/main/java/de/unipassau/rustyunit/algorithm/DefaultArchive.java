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
import java.util.stream.Collectors;

public class DefaultArchive<C extends AbstractTestCaseChromosome<C>> implements Archive<C> {

  private final Set<MinimizingFitnessFunction<C>> objectives;
  private final Map<MinimizingFitnessFunction<C>, C> coveredObjectives;

  public DefaultArchive(Set<MinimizingFitnessFunction<C>> objectives) {
    this.objectives = objectives;
    this.coveredObjectives = new HashMap<>();
  }

  @Override
  public void update(List<C> population) {
    for (var u : objectives) {
      C bestTestCase;
      var bestLength = Integer.MAX_VALUE;
      if ((bestTestCase = coveredObjectives.get(u)) != null) {
        bestLength = bestTestCase.size();
      }

      for (var testCase : population) {
        Objects.requireNonNull(testCase);
        var score = u.getFitness(testCase);
        var length = testCase.size();
        if (score == 0 && length <= bestLength) {
          coveredObjectives.put(u, testCase);
          bestLength = length;
        }
      }
    }

    long coverage = coveredObjectives.keySet().size();
    double percent = ((double) coverage / objectives.size()) * 100;
    System.out.printf("\t>> Covered %d targets out of %d (%.2f%%)%n", coverage, objectives.size(), percent);
    System.out.printf("\t>> Archive contains %d tests%n", new HashSet<>(coveredObjectives.values()).size());
  }

  @Override
  public C getCaseThatCovers(FitnessFunction<C> objective) {
//    var result = archive.stream().filter(t -> objective.getFitness(t) == 0.0).findFirst();
//    return result.orElse(null);
    throw new RuntimeException("Not implemented");
  }

  @Override
  public void replaceBy(C origin, C by) {
//    if (origin != null) {
//      archive.remove(origin);
//    }
//    archive.add(by);

    throw new RuntimeException("Not implemented");
  }

  @Override
  public List<C> get() {
    return new ArrayList<>(new HashSet<>(coveredObjectives.values()));
  }
}
