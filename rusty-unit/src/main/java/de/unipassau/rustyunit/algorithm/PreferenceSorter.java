package de.unipassau.rustyunit.algorithm;

import de.unipassau.rustyunit.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.rustyunit.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import java.util.List;
import java.util.Map;
import java.util.Set;

public interface PreferenceSorter<C extends AbstractTestCaseChromosome<C>> {
  Map<Integer, List<C>> sort(List<C> population);
  Map<Integer, List<C>> sort(List<C> population, Set<MinimizingFitnessFunction<C>> targets);
}
