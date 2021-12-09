package de.unipassau.testify.test_case.statement;

import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.callable.Method;
import de.unipassau.testify.test_case.type.Type;
import java.util.List;
import java.util.Optional;

public class MethodStmt extends CallableStmt {
  private Method method;

  public MethodStmt(TestCase testCase,
      List<VarReference> args, VarReference returnValue, Method method) {
    super(testCase, args, returnValue);
    this.method = method;
  }

  @Override
  public Optional<Type> returnType() {
    return Optional.ofNullable(method.getReturnType());
  }

  @Override
  public boolean returnsValue() {
    return method.returnsValue();
  }

  @Override
  public boolean isMethodStmt() {
    return true;
  }

  @Override
  public MethodStmt asMethodStmt() {
    return this;
  }

  @Override
  public Optional<Type> parent() {
    return Optional.of(method.getParent());
  }

  @Override
  public String name() {
    return method.getName();
  }

  @Override
  public List<Param> params() {
    return method.getParams();
  }
}
