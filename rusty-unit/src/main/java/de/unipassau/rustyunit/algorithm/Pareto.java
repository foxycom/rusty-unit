package de.unipassau.rustyunit.algorithm;

import de.unipassau.rustyunit.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.rustyunit.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import java.util.Set;

public class Pareto<C extends AbstractTestCaseChromosome<C>> implements DominationStrategy<C> {

  @Override
  public boolean dominates(C c1, C c2,
      Set<MinimizingFitnessFunction<C>> objectives) {
    if (objectives.stream().anyMatch(m -> c2.getFitness(m) < c1.getFitness(m))) {
      return false;
    }

    return objectives.stream().anyMatch(m -> c1.getFitness(m) < c2.getFitness(m));
  }
}