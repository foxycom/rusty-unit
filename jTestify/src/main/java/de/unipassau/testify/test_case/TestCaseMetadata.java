package de.unipassau.testify.test_case;

import de.unipassau.testify.linearity.Operator;
import java.util.List;

public class TestCaseMetadata {
  private int id;
  private List<Operator> log;

  public void appendOperator(Operator operator) {
    log.add(operator);
  }
}
