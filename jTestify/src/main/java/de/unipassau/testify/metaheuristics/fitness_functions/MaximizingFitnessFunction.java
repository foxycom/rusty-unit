package de.unipassau.testify.metaheuristics.fitness_functions;

import java.util.function.DoubleUnaryOperator;

public interface MaximizingFitnessFunction<C> extends FitnessFunction<C> {

  /**
   * Always returns {@code false} as this is a maximizing fitness function.
   *
   * @return always {@code false}
   */
  @Override
  default boolean isMinimizing() {
    return false;
  }

  /**
   * {@inheritDoc}
   */
  @Override
  default MaximizingFitnessFunction<C> andThenAsDouble(final DoubleUnaryOperator after) {
    return (MaximizingFitnessFunction<C>) FitnessFunction.super.andThenAsDouble(after);
  }
}
