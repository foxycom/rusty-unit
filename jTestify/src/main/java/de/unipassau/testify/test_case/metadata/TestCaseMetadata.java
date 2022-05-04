package de.unipassau.testify.test_case.metadata;

import de.unipassau.testify.linearity.Operator;
import java.util.List;

public class TestCaseMetadata implements MetaData {
  private int id;
  private List<Operator> log;
  private boolean fails;
  private String filePath;

  public TestCaseMetadata(int id) {
    this.id = id;
  }

  public void appendOperator(Operator operator) {
    log.add(operator);
  }

  @Override
  public int id() {
    return id;
  }

  public void setId(int id) {
    this.id = id;
  }

  @Override
  public boolean fails() {
    return fails;
  }

  @Override
  public void setFails(boolean fails) {
    this.fails = fails;
  }

  @Override
  public String filePath() {
    return filePath;
  }

  @Override
  public void setFilePath(String filePath) {
    this.filePath = filePath;
  }
}
