package de.unipassau.testify.test_case.statement;

import com.google.common.collect.Streams;
import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.callable.StructInit;
import de.unipassau.testify.test_case.type.Type;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.UUID;
import org.javatuples.Pair;

public class StructInitStmt implements Statement {

  private final UUID id;
  private final TestCase testCase;
  private List<VarReference> args;
  private VarReference returnValue;
  private final StructInit structInit;

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

  @Override
  public List<Type> actualParamTypes() {
    return args.stream().peek(Objects::requireNonNull).map(VarReference::type).toList();
  }

  @Override
  public TestCase testCase() {
    return testCase;
  }

  @Override
  public List<VarReference> args() {
    return args;
  }

  @Override
  public void setArgs(List<VarReference> args) {
    if (args.size() != params().size()) {
      throw new RuntimeException("Unequal size of args and params");
    }

    this.args = args;
  }

  @Override
  public void setArg(int pos, VarReference var) {
    args.set(pos, var);
  }

  @Override
  public boolean consumes(VarReference var) {
    return Streams.zip(params().stream(), args.stream(), Pair::with)
        .filter(pair -> pair.getValue1().equals(var))
        .anyMatch(pair -> !pair.getValue0().isByReference());
  }

  @Override
  public boolean borrows(VarReference var) {
    return Streams.zip(params().stream(), args.stream(), Pair::with)
        .filter(pair -> pair.getValue1().equals(var))
        .anyMatch(pair -> pair.getValue0().isByReference());
  }

  @Override
  public boolean uses(VarReference var) {
    return args.stream().anyMatch(a -> a.equals(var));
  }

  @Override
  public void replace(VarReference oldVar, VarReference newVar) {
    if (!args.contains(oldVar)) {
      throw new RuntimeException("There's something wrong");
    }

    var typeBinding = testCase.popTypeBindingsFor(oldVar);
    testCase.setTypeBindingsFor(newVar, typeBinding);

    args = args.stream().map(a -> {
      if (a.equals(oldVar)) {
        return newVar;
      } else {
        return a;
      }
    }).toList();
  }

  @Override
  public Statement copy(TestCase testCase) {
    var argsCopy = args.stream().map(a -> a.copy(testCase)).toList();
    var returnValueCopy = returnValue.copy(testCase);
    return new StructInitStmt(testCase, argsCopy, returnValueCopy, structInit);
  }

  @Override
  public int position() {
    return testCase.stmtPosition(this).orElseThrow();
  }
}
