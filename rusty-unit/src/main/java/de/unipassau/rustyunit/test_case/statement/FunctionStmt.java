package de.unipassau.rustyunit.test_case.statement;

import de.unipassau.rustyunit.test_case.Param;
import de.unipassau.rustyunit.test_case.TestCase;
import de.unipassau.rustyunit.test_case.var.VarReference;
import de.unipassau.rustyunit.test_case.callable.Callable;
import de.unipassau.rustyunit.test_case.callable.Function;
import de.unipassau.rustyunit.type.Type;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.stream.Collectors;

public class FunctionStmt extends CallableStmt {

  private final Function function;

  public FunctionStmt(TestCase testCase,
      List<VarReference> args,
      VarReference returnValue, Function function) {
    super(testCase, args, returnValue);
    this.function = function;
  }

  @Override
  public Optional<Type> returnType() {
    return Optional.ofNullable(function.getReturnType());
  }

  @Override
  public boolean returnsValue() {
    return function.returnsValue();
  }

  @Override
  public List<Param> params() {
    return function.getParams();
  }

  @Override
  public String getSrcFilePath() {
    return function.getSrcFilePath();
  }

  @Override
  public boolean isPublic() {
    return function.isPublic();
  }

  @Override
  public Callable getCallable() {
    return function;
  }

  @Override
  public Statement copy(TestCase testCase) {
    var argsCopy = args.stream()
        .map(a -> a.copy(testCase))
        .collect(Collectors.toCollection(ArrayList::new));
    VarReference returnValueCopy = null;
    if (returnValue != null) {
      returnValueCopy = returnValue.copy(testCase);
    }

    return new FunctionStmt(testCase, argsCopy, returnValueCopy, function);
  }

  @Override
  public Optional<Type> parent() {
    return Optional.empty();
  }

  @Override
  public String name() {
    return function.getName();
  }
}
