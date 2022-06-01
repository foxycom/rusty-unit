package de.unipassau.rustyunit.metaheuristics.operators;

import de.unipassau.rustyunit.metaheuristics.chromosome.Chromosome;
import java.util.Objects;
import java.util.function.BiFunction;
import org.javatuples.Pair;

public interface Crossover<C extends Chromosome<C>> extends BiFunction<C, C, Pair<C, C>> {
  /**
   * A crossover operator that returns the two given parent chromosomes as offspring without
   * actually modifying them.
   *
   * @param <C> the type of chromosomes
   * @return a crossover operator that returns the parents as offspring
   * @apiNote Can be useful for creating dummy chromosomes when writing unit tests.
   */
  static <C extends Chromosome<C>> Crossover<C> identity() {
    return (c, d) -> Pair.with(c.copy(), d.copy());
  }

  /**
   * Applies this crossover operator to the two given non-null parent chromosomes {@code parent1}
   * and {@code parent2}, and returns the resulting pair of offspring chromosomes.
   * <p>
   * Note: an offspring can equal one of its parents (in terms of {@link Chromosome#equals
   * equals()}. While not an absolute requirement, it is generally advisable parents and offspring
   * be different in terms of reference equality ({@code offspring != parent}) as it tends to
   * simplify the implementation of some search algorithms.
   *
   * @param parent1 a parent
   * @param parent2 another parent
   * @return the offspring formed by applying crossover to the two parents
   * @throws NullPointerException if an argument is {@code null}
   */
  @Override
  Pair<C, C> apply(final C parent1, final C parent2);

  /**
   * Applies crossover to the given pair of parent chromosomes and returns the resulting pair of
   * offspring chromosomes.
   *
   * @param parents the parent chromosomes
   * @return the offspring formed by applying crossover to the two parents
   * @throws NullPointerException if an argument is {@code null}
   * @apiNote This method is equivalent to {@link #apply(C, C)} but instead of taking the
   * parents as individual arguments it receives them as pair.
   */
  default Pair<C, C> apply(final Pair<? extends C, ? extends C> parents) {
    Objects.requireNonNull(parents);
    return apply(parents.getValue0(), parents.getValue1());
  }
}
