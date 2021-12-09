package de.unipassau.testify.test_case.statement;

import com.google.common.collect.Streams;
import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.type.Type;
import java.util.List;
import java.util.Optional;
import java.util.UUID;
import org.javatuples.Pair;

public abstract class CallableStmt implements Statement {

  protected UUID id;
  private TestCase testCase;
  protected List<VarReference> args;
  protected VarReference returnValue;

  public CallableStmt(TestCase testCase, List<VarReference> args,
      VarReference returnValue) {
    this.id = UUID.randomUUID();
    this.testCase = testCase;
    this.args = args;
    this.returnValue = returnValue;
  }

  public List<VarReference> args() {
    return args;
  }

  @Override
  public UUID id() {
    return id;
  }

  public void setArgs(List<VarReference> args) {
    this.args = args;
  }

  public void setArg(int pos, VarReference arg) {
    args.set(pos, arg);
  }

  public abstract Optional<Type> parent();

  public abstract String name();

  public abstract List<Param> params();

  @Override
  public Optional<VarReference> returnValue() {
    return Optional.ofNullable(returnValue);
  }

  @Override
  public boolean isCallableStmt() {
    return true;
  }

  @Override
  public CallableStmt asCallableStmt() {
    return this;
  }

  @Override
  public boolean consumes(VarReference var) {
    return Streams.zip(params().stream(), args.stream(), Pair::with)
        .filter(pair -> pair.getValue1() == var)
        .anyMatch(pair -> !pair.getValue0().isByReference());
  }

  @Override
  public boolean borrows(VarReference var) {
    return Streams.zip(params().stream(), args.stream(), Pair::with)
        .filter(pair -> pair.getValue1() == var)
        .anyMatch(pair -> pair.getValue0().isByReference());
  }

  @Override
  public boolean mutates(VarReference var) {
    return Streams.zip(params().stream(), args.stream(), Pair::with)
        .filter(pair -> pair.getValue1() == var)
        .anyMatch(pair -> pair.getValue0().isByReference() && pair.getValue0().isMutable());
  }
}
