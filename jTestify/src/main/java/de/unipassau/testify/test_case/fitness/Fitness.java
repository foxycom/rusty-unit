package de.unipassau.testify.test_case.fitness;

import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import de.unipassau.testify.mir.BasicBlock;
import de.unipassau.testify.test_case.TestCase;

public class Fitness implements MinimizingFitnessFunction<TestCase> {

  // Each branch goal represents a single fitness instance
  private final BasicBlock basicBlock;

  public Fitness(final BasicBlock basicBlock) {
    this.basicBlock = basicBlock;
  }

  public BasicBlock getBasicBlock() {
    return basicBlock;
  }

  @Override
  public double getFitness(TestCase testCase) throws NullPointerException {
    var coverage = testCase.getCoverage();

    return coverage.getOrDefault(basicBlock, Double.MAX_VALUE);
  }
}

