package de.unipassau.testify.test_case.statement;

import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.callable.RefItem;
import de.unipassau.testify.test_case.type.Type;
import java.util.Collections;
import java.util.List;
import java.util.Optional;
import java.util.UUID;

public class RefStmt implements Statement {
  private VarReference arg;
  private RefItem refItem;
  private TestCase testCase;
  private VarReference returnValue;

  public RefStmt(TestCase testCase, VarReference arg, VarReference returnValue, RefItem refItem) {
    this.testCase = testCase;
    this.arg = arg;
    this.refItem = refItem;
    this.returnValue = returnValue;
  }

  @Override
  public UUID id() {
    return null;
  }

  @Override
  public Optional<Type> returnType() {
    return Optional.of(refItem.getReturnType());
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
  public List<VarReference> args() {
    return Collections.singletonList(arg);
  }

  @Override
  public void setArgs(List<VarReference> args) {
    if (args.size() != 1) {
      throw new RuntimeException("There should be exactly one arg");
    }

    this.arg = args.get(0);
  }

  @Override
  public List<Param> params() {
    return refItem.getParams();
  }

  @Override
  public List<Type> actualParamTypes() {
    return Collections.singletonList(arg.type());
  }

  @Override
  public TestCase testCase() {
    return testCase;
  }

  @Override
  public boolean isRefStmt() {
    return true;
  }

  @Override
  public RefStmt asRefStmt() {
    return this;
  }

  public VarReference arg() {
    return arg;
  }

  @Override
  public boolean uses(VarReference var) {
    return arg.equals(var);
  }

  @Override
  public void replace(VarReference oldVar, VarReference newVar) {
    if (arg.equals(oldVar)) {
      throw new RuntimeException("Statement does not use this var");
    }

    this.arg = newVar;
  }

  @Override
  public Statement copy(TestCase testCase) {
    var returnValueCopy = returnValue.copy(testCase);
    var argCopy = arg.copy(testCase);
    return new RefStmt(testCase, argCopy, returnValueCopy, refItem);
  }

  @Override
  public int position() {
    return testCase.stmtPosition(this).orElseThrow();
  }
}
