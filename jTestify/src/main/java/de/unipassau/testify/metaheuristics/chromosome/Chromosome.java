package de.unipassau.testify.metaheuristics.chromosome;

import de.unipassau.testify.metaheuristics.fitness_functions.FitnessFunction;
import de.unipassau.testify.metaheuristics.operators.Crossover;
import de.unipassau.testify.metaheuristics.operators.Mutation;
import de.unipassau.testify.util.SelfTyped;
import java.util.Objects;
import org.javatuples.Pair;

public abstract class Chromosome<C extends Chromosome<C>> implements SelfTyped<C> {
  /**
   * The mutation operator telling how to mutate a chromosome of the current type.
   */
  private final Mutation<C> mutation;

  /**
   * The crossover operator defining how to pair two chromosomes of the current type.
   */
  private final Crossover<C> crossover;

  /**
   * Constructs a new chromosome, using the given mutation and crossover operators for offspring
   * creation.
   *
   * @param mutation  a strategy that tells how to perform mutation, not {@code null}
   * @param crossover a strategy that tells how to perform crossover, not {@code null}
   * @throws NullPointerException if an argument is {@code null}
   */
  protected Chromosome(final Mutation<C> mutation, final Crossover<C> crossover)
      throws NullPointerException {
    this.mutation = Objects.requireNonNull(mutation);
    this.crossover = Objects.requireNonNull(crossover);
  }

  /**
   * Constructs a new chromosome, using the {@link Mutation#identity() identity mutation} and
   * {@link Crossover#identity() identity crossover} operators for offspring creation.
   *
   * @apiNote This constructor primarily intended for use during unit testing, e.g., when aspects
   * of an algorithm are tested that do not rely on a particular mutation or crossover operator.
   */
  protected Chromosome() {
    this(Mutation.identity(), Crossover.identity());
  }

  /**
   * Creates a copy of this chromosome that uses the same mutation and crossover operators as the
   * given {@code other} chromosome.
   *
   * @param other the chromosome to copy
   * @throws NullPointerException if the given chromosome is {@code null}
   * @apiNote Can be called by copy constructors of implementing subclasses.
   */
  protected Chromosome(final C other) throws NullPointerException {
    Objects.requireNonNull(other);
    this.mutation = other.getMutation();
    this.crossover = other.getCrossover();
  }

  /**
   * Returns the mutation operator used by this chromosome.
   *
   * @return the mutation operator
   */
  public Mutation<C> getMutation() {
    return mutation;
  }

  /**
   * Returns the crossover operator used by this chromosome.
   *
   * @return the crossover operator
   */
  public Crossover<C> getCrossover() {
    return crossover;
  }

  /**
   * Applies the mutation operator to this chromosome and returns the resulting offspring.
   *
   * @return the mutated chromosome
   * @apiNote Intended as syntactic sugar for {@link Mutation#apply}
   */
  public final C mutate() {
    return mutation.apply(self());
  }

  /**
   * Applies the crossover operator to this chromosome and the given other given chromosome and
   * returns the resulting offspring.
   *
   * @param other the chromosome with which to pair, not {@code null}
   * @return the offspring
   * @throws NullPointerException if {@code other} is {@code null}
   * @apiNote Intended as syntactic sugar for {@link Crossover#apply}
   */
  public final Pair<C, C> crossover(final C other) {
    Objects.requireNonNull(other);
    return crossover.apply(self(), other);
  }

  /**
   * Computes and returns the fitness of this chromosome using the supplied fitness function.
   *
   * @param fitnessFunction the fitness function with which to compute the fitness of this
   *                        chromosome, not {@code null}
   * @return the fitness of this chromosome as computed by the given fitness function
   * @throws NullPointerException if the given fitness function is {@code null}
   * @apiNote This method is primarily intended as syntactic sugar to allow for a more idiomatic,
   * OOP-like use.
   */
  public final double getFitness(final FitnessFunction<C> fitnessFunction) {
    Objects.requireNonNull(fitnessFunction);
    return fitnessFunction.getFitness(self());
  }

  /**
   * Creates a copy of this chromosome. Implementors should clearly indicate whether a shallow or
   * deep copy is made.
   *
   * @return a copy of this chromosome
   */
  public abstract C copy();

  /**
   * {@inheritDoc}
   */
  @Override
  public abstract boolean equals(final Object other); // enforce custom implementation

  /**
   * {@inheritDoc}
   */
  @Override
  public abstract int hashCode(); // enforce custom implementation
}
