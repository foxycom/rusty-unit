package de.unipassau.testify.metaheuristics.fitness_functions;

import java.util.Comparator;
import java.util.Objects;
import java.util.function.DoubleUnaryOperator;
import java.util.function.Function;
import java.util.function.ToDoubleFunction;

public interface FitnessFunction<C> extends ToDoubleFunction<C>, Function<C, Double> {
  /**
   * <p>
   * Computes and returns the fitness value of the given solution {@code c}. Minimizing fitness
   * functions must return lower values for better solutions, whereas maximizing fitness functions
   * are expected to return higher values. Implementations must ensure that the returned value is
   * always non-negative and never {@code NaN}.
   * </p>
   * <p>
   * When two solutions {@code c1} and {@code c2} are equal it is generally recommended to return
   * the same fitness value for both of them. That is, {@code c1.equals(c2)} implies {@code
   * getFitness(c1) == getFitness(c2)}. While this is not an absolute requirement implementations
   * that do not conform to this should clearly indicate this fact.
   * </p>
   *
   * @param c the solution to rate
   * @return the fitness value of the given solutions
   * @throws NullPointerException if {@code null} is given
   */
  double getFitness(final C c) throws NullPointerException;

  /**
   * Alias for {@link #getFitness(C)}.
   */
  @Override
  default double applyAsDouble(final C c) throws NullPointerException {
    return getFitness(c);
  }

  /**
   * An alias for {@link #applyAsDouble} that returns a boxed {@code Double} instead of a
   * primitive {@code double}.
   */
  @Override
  default Double apply(final C c) {
    return applyAsDouble(c);
  }

  /**
   * Returns a composed fitness function that first applies this function to its input, and then
   * applies the {@code after} function to the result. If evaluation of either function throws an
   * exception, it is relayed to the caller of the composed function.
   *
   * @param after the function to apply after this function is applied
   * @return a composed function that first applies this function and then applies the {@code
   * after} function
   * @throws NullPointerException if after is null
   */
  default FitnessFunction<C> andThenAsDouble(final DoubleUnaryOperator after) {
    Objects.requireNonNull(after);
    return isMinimizing()
        ? (MinimizingFitnessFunction<C>) c -> after.applyAsDouble(this.applyAsDouble(c))
        : (MaximizingFitnessFunction<C>) c -> after.applyAsDouble(this.applyAsDouble(c));
  }

  /**
   * Tells whether this function is a minimizing fitness function. The opposite of {@link
   * #isMaximizing()}.
   *
   * @return {@code true} if this is a minimizing fitness function, {@code false} if this is a
   * maximizing fitness function
   */
  boolean isMinimizing();

  /**
   * Tells whether this function is a maximizing fitness function. The opposite of {@link
   * #isMinimizing()}.
   *
   * @return {@code true} if this is a maximizing fitness function, {@code false} if this is a
   * minimizing fitness function
   */
  default boolean isMaximizing() {
    return !isMinimizing();
  }

  /**
   * Returns a comparator that compares two solutions by their fitness, taking into account
   * whether this is a maximizing or a minimizing fitness function. In other words, given two
   * solutions {@code c1} and {@code c2} with fitness values {@code f1} and {@code f2},
   * respectively, the comparator will return a positive integer if {@code f1} is better than
   * {@code f2}, zero ({@code 0}) if the two fitness values are equal, and a negative integer if
   * {@code f1} is worse than {@code f2}. If this is a minimizing fitness function, smaller
   * fitness values are considered better, and, on the contrary, if this is a maximizing fitness
   * function, larger fitness values are considered better.
   * <p>
   * Example usage:
   * <pre>{@code
   * FitnessFunction<C> ff = ...;
   * C c1 = ...; // first solution to compare
   * C c2 = ...; // second solution to compare
   *
   * int flag = ff.comparator().compare(c1, c2);
   * if (flag > 0) {
   *     // c1 is better than c2
   * } else if (flag < 0) {
   *     // c2 is better than c1
   * } else {
   *     // c1 and c2 are equally good
   * }
   * }</pre>
   *
   * @return a {@code Comparator<C>} that uses this fitness function as extractor for its sort key
   */
  default Comparator<C> comparator() {
    final Comparator<C> comparator = Comparator.comparingDouble(this);
    return isMinimizing() ? comparator.reversed() : comparator;
  }
}
