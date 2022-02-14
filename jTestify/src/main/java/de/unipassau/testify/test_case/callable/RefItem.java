package de.unipassau.testify.test_case.callable;

import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.statement.RefStmt;
import de.unipassau.testify.test_case.statement.Statement;
import de.unipassau.testify.test_case.type.Generic;
import de.unipassau.testify.test_case.type.Ref;
import de.unipassau.testify.test_case.type.Type;
import java.util.Collections;
import java.util.List;
import java.util.stream.Collectors;

public enum RefItem implements Callable {
  INSTANCE(new Param(
      new Generic("T", Collections.emptyList()),
      true,
      null
  ), true);


  private List<Param> params;
  private Type returnType;
  private boolean isPublic;

  RefItem(Param param, boolean isPublic) {
    this.params = Collections.singletonList(param);
    this.returnType = new Ref(param.getType(), true);
    this.isPublic = isPublic;
  }

  @Override
  public String getName() {
    throw new RuntimeException("Not with me");
  }

  @Override
  public void setName(String name) {
    throw new RuntimeException("Not with me");
  }

  @Override
  public List<Param> getParams() {
    return params;
  }

  @Override
  public void setParams(List<Param> params) {
    if (params.size() != 1) {
      throw new RuntimeException("Must be exactly one param");
    }

    this.params = params;
    this.returnType = new Ref(params.get(0).getType(), true);
  }

  @Override
  public Type getReturnType() {
    return returnType;
  }

  @Override
  public void setReturnType(Type type) {
    throw new RuntimeException("Huh? No!");
  }

  @Override
  public Type getParent() {
    throw new RuntimeException("Not with me");
  }

  @Override
  public void setParent(Type parent) {
    throw new RuntimeException("Not with me");
  }

  @Override
  public boolean returnsValue() {
    return true;
  }

  @Override
  public boolean isPublic() {
    return isPublic;
  }

  @Override
  public void setPublic(boolean isPublic) {
    this.isPublic = isPublic;
  }

  @Override
  public Statement toStmt(TestCase testCase, List<VarReference> args, VarReference returnValue) {
    if (args.size() != 1) {
      throw new RuntimeException("Must be exactly one argument");
    }
    return new RefStmt(testCase, args.get(0), returnValue, this);
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
