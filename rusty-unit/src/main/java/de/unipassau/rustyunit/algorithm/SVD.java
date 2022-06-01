package de.unipassau.rustyunit.algorithm;

import de.unipassau.rustyunit.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.rustyunit.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import java.util.List;
import java.util.Set;

public interface SVD<C extends AbstractTestCaseChromosome<C>> {
  void compute(List<C> population);
  void compute(List<C> population, Set<MinimizingFitnessFunction<C>> targets);
}
