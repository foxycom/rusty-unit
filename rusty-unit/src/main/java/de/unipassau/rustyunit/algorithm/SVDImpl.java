package de.unipassau.rustyunit.algorithm;

import de.unipassau.rustyunit.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.rustyunit.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import java.util.Comparator;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;

public class SVDImpl<C extends AbstractTestCaseChromosome<C>> implements SVD<C> {
  private final Set<MinimizingFitnessFunction<C>> objectives;

  public SVDImpl(Set<MinimizingFitnessFunction<C>> objectives) {
    this.objectives = objectives;
  }


  @Override
  public void compute(List<C> population) {
    compute(population, objectives);
  }

  @Override
  public void compute(List<C> population, Set<MinimizingFitnessFunction<C>> targets) {
    Map<C, Integer> distances = new HashMap<>();
    for (int i = 0; i < population.size(); i++) {
      distances.put(population.get(i), 0);
      for (int j = 0; j < population.size(); j++) {
        if (i == j) {
          continue;
        }
        var v = svd(population.get(i), population.get(j), targets);

        if (distances.get(population.get(i)) < v) {
          distances.put(population.get(i), v);
        }
      }
    }

    population.sort(Comparator.comparingInt(distances::get));
  }

  private int svd(C a, C b, Set<MinimizingFitnessFunction<C>> targets) {
    var count = 0;
    for (var m : targets) {
      if (b.getFitness(m) < a.getFitness(m)) {
        count += 1;
      }
    }
    return count;
  }
}
