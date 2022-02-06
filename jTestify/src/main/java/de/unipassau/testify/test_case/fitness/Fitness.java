package de.unipassau.testify.test_case.fitness;

import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import de.unipassau.testify.mir.Branch;
import de.unipassau.testify.test_case.TestCase;

public class Fitness implements MinimizingFitnessFunction<TestCase> {

  // Each branch goal represents a single fitness instance
  private final Branch branch;

  public Fitness(final Branch branch) {
    this.branch = branch;
  }

  public Branch getBranch() {
    return branch;
  }

  @Override
  public double getFitness(TestCase testCase) throws NullPointerException {
    var coverage = testCase.getCoverage();
    throw new RuntimeException("Not implemented yet");
    //return coverage.getOrDefault(branchId, Double.MAX_VALUE);
  }
}

