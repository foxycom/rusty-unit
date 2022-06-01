package de.unipassau.rustyunit.metaheuristics.chromosome;

import java.util.function.Supplier;

public interface ChromosomeGenerator<C extends Chromosome<C>> extends Supplier<C> {

  /**
   * Creates and returns a random chromosome. Implementations must ensure that the returned
   * chromosome represents a valid and admissible solution for the problem at hand.
   *
   * @return a random chromosome
   */
  @Override
  C get();
}