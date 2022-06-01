package de.unipassau.rustyunit.algorithm;

import de.unipassau.rustyunit.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.rustyunit.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import java.util.Set;

public interface DominationStrategy<C extends AbstractTestCaseChromosome<C>> {
  boolean dominates(C candidate1, C candidate2, Set<MinimizingFitnessFunction<C>> objectives);
}
