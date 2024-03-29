package de.unipassau.rustyunit.metaheuristics.algorithm;

import de.unipassau.rustyunit.Listener;
import de.unipassau.rustyunit.metaheuristics.chromosome.Chromosome;
import java.util.List;

public interface GeneticAlgorithm<C extends Chromosome<C>> extends SearchAlgorithm<List<C>> {

  /**
   * Returns a list (i.e., population) of possible admissible solutions to the given problem.
   *
   * @return the solutions
   */
  List<C> findSolution();
}