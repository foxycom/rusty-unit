package de.unipassau.rustyunit.exception;

public class TestCaseFailsException extends Exception {

  public TestCaseFailsException() {
    super("Some test case fails");
  }
}
