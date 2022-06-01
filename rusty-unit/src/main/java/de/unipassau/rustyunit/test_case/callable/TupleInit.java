package de.unipassau.rustyunit.test_case.callable;

import de.unipassau.rustyunit.test_case.Param;
import de.unipassau.rustyunit.test_case.TestCase;
import de.unipassau.rustyunit.test_case.var.VarReference;
import de.unipassau.rustyunit.test_case.statement.Statement;
import de.unipassau.rustyunit.test_case.statement.TupleStmt;
import de.unipassau.rustyunit.type.Tuple;
import de.unipassau.rustyunit.type.Type;
import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

public class TupleInit implements Callable {

  private List<Param> params;
  private Type returnType;

  public TupleInit(List<Param> params) {
    this.params = params;

    var types = params.stream().map(Param::type).toList();
    this.returnType = new Tuple(types);
  }

  @Override
  public String getName() {
    throw new RuntimeException("getName is not implemented");
  }

  @Override
  public void setName(String name) {
    throw new RuntimeException("setName is not implemented");
  }

  @Override
  public List<Param> getParams() {
    return params;
  }

  @Override
  public void setParams(List<Param> params) {
    this.params = Objects.requireNonNull(params);

    var types = params.stream().map(Param::type).toList();
    this.returnType = new Tuple(types);
  }

  @Override
  public Type getReturnType() {
    return returnType;
  }

  @Override
  public void setReturnType(Type type) {
    throw new RuntimeException("setReturnType is not implemented");
  }

  @Override
  public Type getParent() {
    throw new RuntimeException("getParent is not implemented");
  }

  @Override
  public void setParent(Type parent) {
    throw new RuntimeException("setParent is not implemented");
  }

  @Override
  public boolean returnsValue() {
    return true;
  }

  @Override
  public boolean isPublic() {
    return true;
  }

  @Override
  public void setPublic(boolean isPublic) {

  }

  @Override
  public Statement toStmt(TestCase testCase, List<VarReference> args, VarReference returnValue) {
    return new TupleStmt(testCase, args, returnValue, this);
  }

  @Override
  public boolean generates(Type type) {
    return returnType.canBeSameAs(type) || params.stream()
        .anyMatch(p -> p.type().canBeSameAs(type));
  }

  @Override
  public String getSrcFilePath() {
    return null;
  }

  @Override
  public String toString() {
    var paramsStr = params.stream().map(Param::toString).collect(Collectors.joining(", "));
    return String.format("(%s) -> %s", paramsStr, returnType);
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof TupleInit)) {
      return false;
    }
    TupleInit tupleInit = (TupleInit) o;
    return params.equals(tupleInit.params) && returnType.equals(tupleInit.returnType);
  }

  @Override
  public int hashCode() {
    return Objects.hash(params, returnType);
  }
}
