package de.unipassau.rustyunit.metaheuristics.operators;

import de.unipassau.rustyunit.metaheuristics.chromosome.Chromosome;
import java.util.List;
import java.util.function.Function;

public interface Selection<C extends Chromosome<C>> extends Function<List<C>, C> {

  /**
   * Selects a chromosome to be used as parent for mutation or crossover from the given non-null
   * and non-empty population of chromosomes, and returns the result.
   *
   * @param population the population of chromosomes from which to select
   * @return the selected chromosome
   * @throws NullPointerException   if the population is {@code null}
   */
  @Override
  C apply(List<C> population);
}