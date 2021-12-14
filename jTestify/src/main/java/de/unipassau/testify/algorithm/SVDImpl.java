package de.unipassau.testify.algorithm;

import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import java.util.Comparator;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class SVDImpl<C extends AbstractTestCaseChromosome<C>> implements SVD<C> {
  private final List<MinimizingFitnessFunction<C>> objectives;

  public SVDImpl(List<MinimizingFitnessFunction<C>> objectives) {
    this.objectives = objectives;
  }


  @Override
  public void compute(List<C> population) {
    Map<C, Integer> distances = new HashMap<>();
    for (int i = 0; i < population.size(); i++) {
      distances.put(population.get(i), 0);
      for (int j = 0; j < population.size(); j++) {
        if (i == j) {
          continue;
        }
        var v = svd(population.get(i), population.get(j));

        if (distances.get(population.get(i)) < v) {
          distances.put(population.get(i), v);
        }
      }
    }

    population.sort(Comparator.comparingInt(distances::get));
  }

  private int svd(C a, C b) {
    var count = 0;
    for (var m : objectives) {
      if (b.getFitness(m) < a.getFitness(m)) {
        count += 1;
      }
    }
    return count;
  }
}
