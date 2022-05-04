package de.unipassau.testify.algorithm;

import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.metaheuristics.fitness_functions.FitnessFunction;
import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

public class ArchiveImpl<C extends AbstractTestCaseChromosome<C>> implements Archive<C> {

  private final Set<C> archive;
  private final Set<MinimizingFitnessFunction<C>> objectives;

  public ArchiveImpl(Set<MinimizingFitnessFunction<C>> objectives) {
    this.archive = new HashSet<>();
    this.objectives = objectives;
  }

  @Override
  public void update(List<C> population) {
    for (var u : objectives) {
      C bestTestCase;
      var bestLength = Integer.MAX_VALUE;
      if ((bestTestCase = getCaseThatCovers(u)) != null) {
        bestLength = bestTestCase.size();
      }

      for (var testCase : population) {
        var score = u.getFitness(testCase);
        var length = testCase.size();
        if (score == 0 && length <= bestLength && !testCase.metadata().fails()) {
          // replace bestTestCase with testCase in archive
          replaceBy(bestTestCase, testCase);
          bestTestCase = testCase;
          bestLength = length;
        }
      }
    }
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
