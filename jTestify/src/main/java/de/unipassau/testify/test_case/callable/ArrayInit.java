package de.unipassau.testify.test_case.callable;

import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.statement.Statement;
import de.unipassau.testify.test_case.statement.array.DirectArrayInitStmt;
import de.unipassau.testify.test_case.type.Array;
import de.unipassau.testify.test_case.type.Type;
import java.util.List;
import java.util.stream.Collectors;
import java.util.stream.IntStream;

public class ArrayInit implements Callable {
  private final Array array;
  private final List<Param> params;

  public ArrayInit(Array array) {
    this.array = array;
    this.params = IntStream.range(0, array.length())
        .mapToObj(i -> new Param(array.type(), false, null))
        .collect(Collectors.toList());
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
    throw new RuntimeException("setParams is not implemented");
  }

  @Override
  public Type getReturnType() {
    return array;
  }

  @Override
  public void setReturnType(Type type) {
    throw new RuntimeException("setReturnType is not implemented");
  }

  @Override
  public Type getParent() {
    return null;
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
    throw new RuntimeException("setPublic is not implemented");
  }

  @Override
  public Statement toStmt(TestCase testCase, List<VarReference> args, VarReference returnValue) {
    return new DirectArrayInitStmt(testCase, args, returnValue, this);
  }
}
