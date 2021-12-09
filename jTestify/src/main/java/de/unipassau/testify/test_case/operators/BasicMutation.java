package de.unipassau.testify.test_case.operators;

import de.unipassau.testify.metaheuristics.operators.Mutation;
import de.unipassau.testify.test_case.TestCase;

public class BasicMutation implements Mutation<TestCase> {

  @Override
  public TestCase apply(TestCase statements) {
    throw new RuntimeException("Not implemented");
  }
}
