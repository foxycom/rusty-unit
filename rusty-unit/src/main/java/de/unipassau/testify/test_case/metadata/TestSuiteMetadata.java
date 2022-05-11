package de.unipassau.testify.test_case.metadata;

public class TestSuiteMetadata {
  private long executionTime;
  private int seed;

  public TestSuiteMetadata(int seed, long executionTime) {
    this.seed = seed;
    this.executionTime = executionTime;
  }

  public int seed() {
    return seed;
  }

  public long executionTime() {
    return executionTime;
  }
}
