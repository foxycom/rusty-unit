package de.unipassau.testify.metaheuristics.chromosome;

import de.unipassau.testify.metaheuristics.operators.Crossover;
import de.unipassau.testify.metaheuristics.operators.Mutation;
import de.unipassau.testify.test_case.statement.Statement;
import java.util.Iterator;
import java.util.List;

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
}
