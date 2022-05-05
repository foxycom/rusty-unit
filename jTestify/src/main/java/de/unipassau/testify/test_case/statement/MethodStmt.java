package de.unipassau.testify.test_case.statement;

import static java.util.stream.Collectors.toCollection;

import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.var.VarReference;
import de.unipassau.testify.test_case.callable.Callable;
import de.unipassau.testify.test_case.callable.Method;
import de.unipassau.testify.test_case.type.Type;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

public class MethodStmt extends CallableStmt {

  private final Method method;

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
  public Callable getCallable() {
    return method;
  }

  @Override
  public void replaceAt(int pos, VarReference var) {
    args.set(pos, var);
  }

  @Override
  public Statement copy(TestCase testCase) {
    var argsCopy = args.stream()
        .map(a -> a.copy(testCase))
        .collect(toCollection(ArrayList::new));
    VarReference returnValueCopy = null;
    if (returnValue != null) {
      returnValueCopy = returnValue.copy(testCase);
    }
    return new MethodStmt(testCase, argsCopy, returnValueCopy, method);
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

  @Override
  public String getSrcFilePath() {
    return method.getSrcFilePath();
  }

  @Override
  public boolean isPublic() {
    return method.isPublic();
  }


}
