package de.unipassau.testify.test_case.operators;

import de.unipassau.testify.metaheuristics.operators.Crossover;
import de.unipassau.testify.test_case.TestCase;
import org.javatuples.Pair;

public class SinglePointCrossover implements Crossover<TestCase> {

  @Override
  public Pair<TestCase, TestCase> apply(TestCase parent1, TestCase parent2) {
    throw new RuntimeException("Not implemented");
  }
}
