package de.unipassau.testify.test_case.fitness;

import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import de.unipassau.testify.test_case.TestCase;

public class Fitness implements MinimizingFitnessFunction<TestCase> {

  // Each branch goal represents a single fitness instance
  private final int branchId;

  public Fitness(final int branchId) {
    this.branchId = branchId;
  }

  public int getBranchId() {
    return branchId;
  }

  @Override
  public double getFitness(TestCase testCase) throws NullPointerException {
    var coverage = testCase.getCoverage();
    return coverage.getOrDefault(branchId, Double.MAX_VALUE);
  }
}

