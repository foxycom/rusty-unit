package de.unipassau.testify.exception;

public class TestCaseFailsException extends Exception {

  public TestCaseFailsException() {
    super("Some test case fails");
  }
}
