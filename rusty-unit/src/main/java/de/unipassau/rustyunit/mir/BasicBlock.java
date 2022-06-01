package de.unipassau.rustyunit.mir;

import com.google.common.base.Preconditions;
import de.unipassau.rustyunit.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import de.unipassau.rustyunit.test_case.TestCase;
import java.util.HashMap;
import java.util.Objects;

public class BasicBlock implements MinimizingFitnessFunction<TestCase> {

  private static final int DUMMY_ID = 42069;
  private final String globalId;
  private final int blockId;

  private HashMap<Integer, Double> cache;

  public BasicBlock(String globalId, int blockId) {
    Preconditions.checkState(blockId >= 0);
    this.globalId = globalId;
    this.blockId = blockId;
    this.cache = new HashMap<>();
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
    if (cache.containsKey(testCase.getId())) {
      return cache.get(testCase.getId());
    }
    var branchDistance = testCase.branchDistance();

    var coveredTargets = branchDistance.keySet();
    var cdg = testCase.mir().getCdgFor(globalId);

    var path = cdg.pathTo(this);
    int approachLevel = cdg.approachLevel(this, coveredTargets);
    Preconditions.checkState(approachLevel >= 0 && approachLevel <= path.size());

    double fitness;
    if (approachLevel == 0) {
      fitness = normalize(branchDistance.get(this));
    } else if (approachLevel == path.size()) {
      fitness = Double.MAX_VALUE;
    } else {
      var nearestCoveredObject = path.get(path.size() - approachLevel - 1);
      var localFitness = testCase.branchDistance().get(nearestCoveredObject);
      fitness = approachLevel + normalize(localFitness);
    }

    cache.put(testCase.getId(), fitness);
    return fitness;
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
    if (!(o instanceof BasicBlock that)) {
      return false;
    }
    return blockId == that.blockId && globalId.equals(that.globalId);
  }

  @Override
  public int hashCode() {
    return Objects.hash(globalId, blockId);
  }


}
