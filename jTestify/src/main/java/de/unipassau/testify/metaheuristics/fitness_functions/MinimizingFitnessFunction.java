package de.unipassau.testify.metaheuristics.fitness_functions;

import java.util.function.DoubleUnaryOperator;

public interface MinimizingFitnessFunction<C> extends FitnessFunction<C> {

  default boolean isDummy() {
    return false;
  }
  /**
   * Always returns {@code true} as this is a minimizing fitness function.
   *
   * @return always {@code true}
   */
  @Override
  default boolean isMinimizing() {
    return true;
  }

  /**
   * {@inheritDoc}
   */
  @Override
  default MinimizingFitnessFunction<C> andThenAsDouble(final DoubleUnaryOperator after) {
    return (MinimizingFitnessFunction<C>) FitnessFunction.super.andThenAsDouble(after);
  }
}