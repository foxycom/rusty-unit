package de.unipassau.testify.test_case.statement;

import com.google.common.base.Preconditions;
import com.google.common.collect.Streams;
import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.callable.TupleInit;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.util.Rnd;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.UUID;
import java.util.stream.Collectors;
import org.javatuples.Pair;

public class TupleStmt implements Statement {

  private UUID id;
  private TupleInit tupleInit;
  private TestCase testCase;
  private VarReference returnValue;
  private List<VarReference> args;

  public TupleStmt(TestCase testCase, List<VarReference> args, VarReference returnValue,
      TupleInit tupleInit) {
    this.id = UUID.randomUUID();
    this.tupleInit = tupleInit;
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
    return Optional.of(returnValue.type());
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
    Preconditions.checkArgument(args.size() == params().size());

    this.args = args;
  }

  @Override
  public void setArg(int pos, VarReference var) {
    this.args.set(pos, var);
  }

  @Override
  public List<Param> params() {
    return tupleInit.getParams();
  }

  @Override
  public List<Type> actualParamTypes() {
    return args.stream().peek(Objects::requireNonNull).map(VarReference::type).toList();
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
  public boolean mutates(VarReference var) {
    return Streams.zip(params().stream(), args.stream(), Pair::with)
        .filter(pair -> pair.getValue1().equals(var))
        .anyMatch(pair -> pair.getValue0().isByReference() && pair.getValue0().isMutable());
  }

  @Override
  public boolean uses(VarReference var) {
    return args.stream().anyMatch(a -> a.equals(var));
  }

  @Override
  public boolean mutate(TestCase testCase) {
    var p = 1d / params().size();
    boolean changed = false;
    for (int i = 0; i < params().size(); i++) {
      if (Rnd.get().nextDouble() < p) {
        var param = params().get(i).bindGenerics(returnValue.getBinding());
        var oldArg = args().get(i);
        var newArg = testCase.getArg(param.getType(), position());
        newArg.ifPresent(a -> replace(oldArg, a));

        changed = true;
      }
    }

    return changed;
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
    return tupleInit.isPublic();
  }

  @Override
  public void replace(VarReference oldVar, VarReference newVar) {
    if (args.stream().noneMatch(v -> v.equals(oldVar))) {
      throw new RuntimeException("Statement does not use this var");
    }

    var idx = args.indexOf(oldVar);
    args.set(idx, newVar);
  }

  @Override
  public boolean isTupleStmt() {
    return true;
  }

  @Override
  public TupleStmt asTupleStmt() {
    return this;
  }

  @Override
  public Statement copy(TestCase testCase) {
    var returnValueCopy = returnValue.copy(testCase);
    var argsCopy = args.stream().map(a -> a.copy(testCase))
        .collect(Collectors.toCollection(ArrayList::new));

    return new TupleStmt(testCase, argsCopy, returnValueCopy, tupleInit);
  }

  @Override
  public int position() {
    return testCase.stmtPosition(this).orElseThrow();
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof TupleStmt)) {
      return false;
    }
    TupleStmt tupleStmt = (TupleStmt) o;
    return id.equals(tupleStmt.id) && tupleInit.equals(tupleStmt.tupleInit) && returnValue.equals(
        tupleStmt.returnValue) && args.equals(tupleStmt.args);
  }

  @Override
  public int hashCode() {
    return Objects.hash(id, tupleInit, returnValue, args);
  }
}
