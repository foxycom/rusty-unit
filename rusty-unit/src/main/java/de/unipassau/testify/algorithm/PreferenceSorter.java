package de.unipassau.testify.algorithm;

import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import de.unipassau.testify.mir.BasicBlock;
import java.util.List;
import java.util.Map;
import java.util.Set;

public interface PreferenceSorter<C extends AbstractTestCaseChromosome<C>> {
  Map<Integer, List<C>> sort(List<C> population);
  Map<Integer, List<C>> sort(List<C> population, Set<MinimizingFitnessFunction<C>> targets);
}
