package de.unipassau.testify.test_case;

import de.unipassau.testify.linearity.Operator;
import java.util.List;

public class TestCaseMetadata {
  private int id;
  private List<Operator> log;
  private boolean fails;

  public TestCaseMetadata(int id) {
    this.id = id;
  }

  public void appendOperator(Operator operator) {
    log.add(operator);
  }

  public int id() {
    return id;
  }

  public void setId(int id) {
    this.id = id;
  }

  public boolean fails() {
    return fails;
  }

  public void setFails(boolean fails) {
    this.fails = fails;
  }
}
