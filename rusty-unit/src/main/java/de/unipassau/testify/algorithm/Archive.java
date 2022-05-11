package de.unipassau.testify.algorithm;

import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.metaheuristics.fitness_functions.FitnessFunction;
import java.util.List;

public interface Archive<C extends AbstractTestCaseChromosome<C>> {
  void update(final List<C> population);
  C getCaseThatCovers(FitnessFunction<C> objective);
  void replaceBy(C origin, C by);
  List<C> get();
}
