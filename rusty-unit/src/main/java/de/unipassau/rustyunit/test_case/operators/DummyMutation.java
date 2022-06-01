package de.unipassau.rustyunit.test_case.operators;

import de.unipassau.rustyunit.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.rustyunit.metaheuristics.operators.Mutation;

public class DummyMutation<C extends AbstractTestCaseChromosome<C>> implements Mutation<C> {

  @Override
  public C apply(C testCase) {
    return testCase;
  }
}
