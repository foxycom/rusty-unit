package de.unipassau.testify.algorithm;

import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import de.unipassau.testify.mir.BasicBlock;
import java.util.List;
import java.util.Set;

public interface SVD<C extends AbstractTestCaseChromosome<C>> {
  void compute(List<C> population);
  void compute(List<C> population, Set<MinimizingFitnessFunction<C>> targets);
}
