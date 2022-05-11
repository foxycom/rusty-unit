package de.unipassau.rustyunit.test_case.statement;

import static java.util.stream.Collectors.toCollection;

import de.unipassau.rustyunit.test_case.Param;
import de.unipassau.rustyunit.test_case.TestCase;
import de.unipassau.rustyunit.test_case.var.VarReference;
import de.unipassau.rustyunit.test_case.callable.Callable;
import de.unipassau.rustyunit.test_case.callable.StaticMethod;
import de.unipassau.rustyunit.type.Type;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

public class StaticMethodStmt extends CallableStmt {
  private final StaticMethod method;

  public StaticMethodStmt(TestCase testCase, List<VarReference> args, VarReference returnValue, StaticMethod method) {
    super(testCase, args, returnValue);
    this.method = method;
  }

  public Optional<String> ofTrait() {
    return Optional.ofNullable(method.ofTrait());
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
  public boolean isStaticMethodStmt() {
    return true;
  }

  @Override
  public StaticMethodStmt asStaticMethodStmt() {
    return this;
  }

  @Override
  public Callable getCallable() {
    return method;
  }

  @Override
  public Statement copy(TestCase testCase) {
    var argsCopy = args.stream()
        .map(a -> a.copy(testCase))
        .collect(toCollection(ArrayList::new));
    var returnValueCopy = (returnValue == null ) ? null : returnValue.copy(testCase);
    return new StaticMethodStmt(testCase, argsCopy, returnValueCopy, method);
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
