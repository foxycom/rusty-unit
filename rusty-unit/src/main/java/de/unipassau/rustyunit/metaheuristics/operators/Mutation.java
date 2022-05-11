package de.unipassau.rustyunit.metaheuristics.operators;

import de.unipassau.rustyunit.metaheuristics.chromosome.Chromosome;
import java.util.function.UnaryOperator;

public interface Mutation<C extends Chromosome<C>> extends UnaryOperator<C> {
  /**
   * A mutation operator that always returns the parent chromosome as offspring.
   *
   * @param <C> the type of chromosome
   * @return a mutation operator that always returns the parent as offspring
   * @apiNote Can be useful for creating dummy chromosomes when writing unit tests.
   */
  static <C extends Chromosome<C>> Mutation<C> identity() {
    return C::copy;
  }

  /**
   * Applies mutation to the given chromosome {@code c} and returns the resulting offspring.
   * Usually, it is desirable that the parent chromosome not be modified in-place. Instead, it is
   * advisable to create a copy of the parent, mutate the copy and return it as offspring. While
   * this is not an absolute requirement implementations that do not conform to this should
   * clearly indicate this fact.
   *
   * @param c the parent chromosome to mutate
   * @return the offspring formed by mutating the parent
   */
  @Override
  C apply(final C c);
}
