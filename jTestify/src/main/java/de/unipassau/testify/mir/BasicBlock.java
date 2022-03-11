package de.unipassau.testify.mir;

import com.google.common.base.Preconditions;
import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import de.unipassau.testify.test_case.TestCase;

public record BasicBlock(String globalId, int blockId) implements MinimizingFitnessFunction<TestCase> {

  private static final int DUMMY_ID = 42069;

  public BasicBlock {
    Preconditions.checkState(blockId >= 0);
  }

  public static BasicBlock of(String globalId, int blockId) {
    return new BasicBlock(globalId, blockId);
  }

  @Override
  public boolean isDummy() {
    return blockId == DUMMY_ID;
  }

  @Override
  public String toString() {
    return String.format("%s:%d", globalId, blockId);
  }

  @Override
  public double getFitness(TestCase testCase) throws NullPointerException {
    var coverage = testCase.getCoverage();

    return coverage.getOrDefault(this, Double.MAX_VALUE);
  }

  @Override
  public String id() {
    return globalId;
  }
}
