package de.unipassau.rustyunit.algorithm;

import de.unipassau.rustyunit.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.rustyunit.metaheuristics.fitness_functions.FitnessFunction;
import java.util.List;
import org.javatuples.Pair;

public interface Archive<C extends AbstractTestCaseChromosome<C>> {

  interface Stats {
    int coveredTargets();

    double fitness();

    double coverage();
  }
  void update(final List<C> population);

  Stats coverage();
  int size();

  int numberOfObjectives();

  List<C> get();
}
