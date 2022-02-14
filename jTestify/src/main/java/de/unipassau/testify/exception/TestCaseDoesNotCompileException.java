package de.unipassau.testify.exception;

public class TestCaseDoesNotCompileException extends Exception {

  public TestCaseDoesNotCompileException() {
    super("Some tests did not compile");
  }
}
