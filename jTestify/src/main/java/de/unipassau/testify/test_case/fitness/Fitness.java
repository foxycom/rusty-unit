package de.unipassau.testify.test_case.fitness;

import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import de.unipassau.testify.mir.BasicBlock;
import de.unipassau.testify.test_case.TestCase;
import java.util.Objects;

public class Fitness implements MinimizingFitnessFunction<TestCase> {

  // Each branch goal represents a single fitness instance
  private final BasicBlock basicBlock;

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    Fitness fitness = (Fitness) o;
    return basicBlock.equals(fitness.basicBlock);
  }

  @Override
  public int hashCode() {
    return Objects.hash(basicBlock);
  }

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

  @Override
  public String id() {
    return basicBlock.id();
  }
}

