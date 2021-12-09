package de.unipassau.testify.test_case.statement;

import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.callable.StructInit;
import de.unipassau.testify.test_case.type.Type;
import java.util.List;
import java.util.Optional;
import java.util.UUID;

public class StructInitStmt implements Statement {

  private UUID id;
  private TestCase testCase;
  private List<VarReference> args;
  private VarReference returnValue;
  private StructInit structInit;

  public StructInitStmt(TestCase testCase,
      List<VarReference> args,
      VarReference returnValue,
      StructInit structInit) {
    this.id = UUID.randomUUID();
    this.testCase = testCase;
    this.args = args;
    this.returnValue = returnValue;
    this.structInit = structInit;
  }

  @Override
  public UUID id() {
    return id;
  }

  @Override
  public Optional<Type> returnType() {
    return Optional.of(structInit.getReturnType());
  }

  @Override
  public Optional<VarReference> returnValue() {
    return Optional.of(returnValue);
  }

  @Override
  public boolean returnsValue() {
    return true;
  }

  @Override
  public boolean isStructInitStmt() {
    return true;
  }

  @Override
  public StructInitStmt asStructInitStmt() {
    return this;
  }

  public List<Param> params() {
    return structInit.getParams();
  }

  public List<VarReference> args() {
    return args;
  }
}
