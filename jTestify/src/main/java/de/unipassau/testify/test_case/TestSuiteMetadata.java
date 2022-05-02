package de.unipassau.testify.test_case;

public class TestSuiteMetadata {
  private long executionTime;

  public TestSuiteMetadata(long executionTime) {
    this.executionTime = executionTime;
  }

  public long executionTime() {
    return executionTime;
  }
}
