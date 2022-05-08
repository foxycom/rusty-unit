package de.unipassau.testify.metaheuristics.chromosome;

import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import de.unipassau.testify.metaheuristics.operators.Crossover;
import de.unipassau.testify.metaheuristics.operators.Mutation;
import de.unipassau.testify.test_case.metadata.MetaData;
import de.unipassau.testify.test_case.metadata.TestCaseMetadata;
import de.unipassau.testify.test_case.statement.Statement;
import java.util.Iterator;
import java.util.List;
import java.util.Set;

public abstract class AbstractTestCaseChromosome<C extends AbstractTestCaseChromosome<C>>
    extends Chromosome<C>
    implements Iterable<Statement> {

  protected AbstractTestCaseChromosome(final Mutation<C> mutation, final Crossover<C> crossover) {
    super(mutation, crossover);
  }

  protected AbstractTestCaseChromosome(final C other) {
    super(other);
  }

  public abstract int size();

  public boolean isEmpty() {
    return size() == 0;
  }

  public abstract int getId();

  /**
   * Returns the statements of this test case chromosome.
   *
   * @return the sequence of statements
   */
  public abstract List<Statement> getStatements();

  /**
   * Returns an iterator over the statements of this test case chromosome.
   *
   * @return a statement iterator
   */
  @Override
  public Iterator<Statement> iterator() {
    return getStatements().iterator();
  }

  public abstract MetaData metadata();

  public abstract Set<MinimizingFitnessFunction<C>> coveredObjectives();

  public abstract Set<MinimizingFitnessFunction<C>> uncoveredObjectives();
}
