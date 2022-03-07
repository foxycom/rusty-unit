package de.unipassau.testify.test_case.statement.array;

import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.callable.ArrayInit;
import de.unipassau.testify.test_case.statement.Statement;
import de.unipassau.testify.test_case.type.Type;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.UUID;
import java.util.stream.Collectors;
import java.util.stream.IntStream;

public class DirectArrayInitStmt implements Statement {

  private final UUID id;
  private VarReference returnValue;
  private List<VarReference> args;
  private TestCase testCase;
  private ArrayInit arrayInit;

  public DirectArrayInitStmt(TestCase testCase, List<VarReference> args, VarReference returnValue,
      ArrayInit arrayInit) {
    this.id = UUID.randomUUID();
    this.arrayInit = arrayInit;
    this.testCase = testCase;
    this.returnValue = returnValue;
    this.args = args;
  }

  @Override
  public UUID id() {
    return id;
  }

  @Override
  public Optional<Type> returnType() {
    return Optional.of(arrayInit.getReturnType());
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
    return args;
  }

  @Override
  public void setArgs(List<VarReference> args) {
    if (args.size() != params().size()) {
      throw new RuntimeException("Unequal number of args and params");
    }

    this.args = args;
  }

  @Override
  public void setArg(int pos, VarReference var) {
    args.set(pos, var);
  }

  @Override
  public List<Param> params() {
    return arrayInit.getParams();
  }

  @Override
  public List<Type> actualParamTypes() {
    return args.stream().peek(Objects::requireNonNull).map(VarReference::type).toList();
  }

  @Override
  public TestCase testCase() {
    return testCase;
  }

  @Override
  public String getSrcFilePath() {
    return null;
  }

  @Override
  public boolean isPublic() {
    return arrayInit.isPublic();
  }

  @Override
  public void replace(VarReference oldVar, VarReference newVar) {
    if (!args.contains(oldVar)) {
      throw new RuntimeException("There's something wrong");
    }

    args.replaceAll(a -> a.equals(oldVar) ? newVar : a);
  }

  @Override
  public Statement copy(TestCase testCase) {
    var argsCopy = args.stream()
        .map(a -> a.copy(testCase))
        .collect(Collectors.toCollection(ArrayList::new));
    var returnValueCopy = returnValue.copy(testCase);
    return new DirectArrayInitStmt(testCase, argsCopy, returnValueCopy, arrayInit);
  }

  @Override
  public int position() {
    return testCase.stmtPosition(this).orElseThrow();
  }

  @Override
  public boolean isArrayStmt() {
    return true;
  }

  @Override
  public DirectArrayInitStmt asArrayStmt() {
    return this;
  }

  @Override
  public boolean consumes(VarReference var) {
    var typeBinding = returnValue.getBinding();

    var pos = IntStream.range(0, args.size()).filter(i -> args.get(i).equals(var)).findFirst();
    if (pos.isPresent()) {
      return !params().get(pos.getAsInt()).bindGenerics(typeBinding).isByReference();
    } else {
      return false;
    }
  }

  @Override
  public boolean borrows(VarReference var) {
    var typeBinding = returnValue.getBinding();

    var pos = IntStream.range(0, args.size()).filter(i -> args.get(i).equals(var)).findFirst();
    if (pos.isPresent()) {
      return params().get(pos.getAsInt()).bindGenerics(typeBinding).isByReference();
    } else {
      return false;
    }
  }

  @Override
  public boolean mutates(VarReference var) {
    throw new RuntimeException("mutates is not implemented");
  }

}
