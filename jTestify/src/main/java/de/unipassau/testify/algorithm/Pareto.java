package de.unipassau.testify.algorithm;

import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import java.util.List;

public class Pareto<C extends AbstractTestCaseChromosome<C>> implements DominationStrategy<C> {

  @Override
  public boolean dominates(C c1, C c2,
      List<MinimizingFitnessFunction<C>> objectives) {
    if (objectives.parallelStream().anyMatch(m -> c2.getFitness(m) < c1.getFitness(m))) {
      return false;
    }

    return objectives.parallelStream().anyMatch(m -> c1.getFitness(m) < c2.getFitness(m));
  }
}