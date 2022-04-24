package de.unipassau.testify.mir;

import com.google.common.base.Preconditions;
import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import de.unipassau.testify.test_case.TestCase;
import java.util.HashMap;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.Set;
import java.util.stream.IntStream;

public class BasicBlock implements
    MinimizingFitnessFunction<TestCase> {

  private static final int DUMMY_ID = 42069;
  private final String globalId;

  private final int blockId;

  public BasicBlock(String globalId, int blockId) {
    Preconditions.checkState(blockId >= 0);
    this.globalId = globalId;
    this.blockId = blockId;
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

  private double normalize(double value) {
    return value / (value + 1.0);
  }

  @Override
  public double getFitness(TestCase testCase) throws NullPointerException {
    var branchDistance = testCase.branchDistance();
    if (branchDistance.containsKey(this)) {
      return normalize(branchDistance.get(this));
    } else {
      var coveredTargets = testCase.branchDistance().keySet();
      var cdg = testCase.mir().getCdgFor(globalId);
      var pathToThis = cdg.pathTo(this);

      // Determine nearest covered parent block index in the tree path
      var parentIndex = IntStream.range(0, pathToThis.size())
          .filter(i -> coveredTargets.contains(pathToThis.get(i)))
          .findFirst();

      if (parentIndex.isEmpty()) {
        return Double.MAX_VALUE;
      }

      int approachLevel = pathToThis.size() - parentIndex.getAsInt();
      Preconditions.checkState(approachLevel > 0);

      var fitness = approachLevel + normalize(testCase.branchDistance().get(pathToThis.get(parentIndex.getAsInt())));
      return fitness;
    }
  }

  public String globalId() {
    return globalId;
  }

  public int blockId() {
    return blockId;
  }

  @Override
  public String id() {
    return globalId;
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof BasicBlock)) {
      return false;
    }
    BasicBlock that = (BasicBlock) o;
    return blockId == that.blockId && globalId.equals(that.globalId);
  }

  @Override
  public int hashCode() {
    return Objects.hash(globalId, blockId);
  }
}
