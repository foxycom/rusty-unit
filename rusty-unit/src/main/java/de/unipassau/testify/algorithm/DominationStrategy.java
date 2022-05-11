package de.unipassau.testify.algorithm;

import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import java.util.Collection;
import java.util.List;
import java.util.Set;

public interface DominationStrategy<C extends AbstractTestCaseChromosome<C>> {
  boolean dominates(C candidate1, C candidate2, Set<MinimizingFitnessFunction<C>> objectives);
}
