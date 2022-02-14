package de.unipassau.testify.test_case.callable;

import com.google.common.base.Preconditions;
import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.statement.Statement;
import de.unipassau.testify.test_case.statement.TupleStmt;
import de.unipassau.testify.test_case.type.Generic;
import de.unipassau.testify.test_case.type.Tuple;
import de.unipassau.testify.test_case.type.Type;
import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

public enum TupleInit implements Callable {
  DEFAULT(Collections.emptyList()),
  SINGLE(
      List.of(new Param(new Generic("T", Collections.emptyList()), false, null))
  ),
  PAIR(
      List.of(
          new Param(new Generic("A", Collections.emptyList()), false, null),
          new Param(new Generic("B", Collections.emptyList()), false, null)
      )
  ),
  TRIPLETT(
      List.of(
          new Param(new Generic("A", Collections.emptyList()), false, null),
          new Param(new Generic("B", Collections.emptyList()), false, null),
          new Param(new Generic("C", Collections.emptyList()), false, null)
      )
  );


  private List<Param> params;
  private Type returnType;

  TupleInit(List<Param> params) {
    this.params = params;

    var types = params.stream().map(Param::getType).toList();
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

    var types = params.stream().map(Param::getType).toList();
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
        .anyMatch(p -> p.getType().canBeSameAs(type));
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
}
