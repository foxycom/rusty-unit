package de.unipassau.rustyunit.test_case;

public enum TestCaseFactory {
  INSTANCE;

  public boolean mutateParameter(TestCase testCase, int pos) {
    throw new RuntimeException("Not implemented yet");
  }
}
