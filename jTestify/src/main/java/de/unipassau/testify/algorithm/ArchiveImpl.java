package de.unipassau.testify.algorithm;

import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.metaheuristics.fitness_functions.FitnessFunction;
import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import java.util.ArrayList;
import java.util.List;

public class ArchiveImpl<C extends AbstractTestCaseChromosome<C>> implements Archive<C> {

  private final List<C> archive;
  private final List<MinimizingFitnessFunction<C>> objectives;

  public ArchiveImpl(List<MinimizingFitnessFunction<C>> objectives) {
    this.archive = new ArrayList<>();
    this.objectives = objectives;
  }

  @Override
  public void update(List<C> population) {
    for (var u : objectives) {
      C bestTestCase;
      var bestLength = Integer.MAX_VALUE;
      if ((bestTestCase = getCaseThatCovers(u)) != null) {
        bestLength = bestTestCase.getStatements().size();
      }

      for (var testCase : population) {
        var score = u.getFitness(testCase);
        var length = testCase.getStatements().size();
        if (score == 0 && length <= bestLength) {
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
    if (origin == null) {
      archive.add(by);
    } else {
      var idx = archive.indexOf(origin);
      archive.set(idx, by);
    }
  }

  @Override
  public List<C> get() {
    return archive;
  }
}
