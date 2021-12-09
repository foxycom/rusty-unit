package de.unipassau.testify.test_case.callable;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.statement.Statement;
import de.unipassau.testify.test_case.type.Type;
import java.util.List;
import java.util.stream.Collectors;

@JsonDeserialize(as = Function.class)
public class Function implements Callable {

  private String name;
  private List<Param> params;

  @JsonProperty("return_type")
  private Type returnType;

  @JsonProperty("src_file_id")
  private int srcFileId;

  @Override
  public String getName() {
    return name;
  }

  @Override
  public void setName(String name) {
    this.name = name;
  }

  @Override
  public List<Param> getParams() {
    return params;
  }

  @Override
  public void setParams(List<Param> params) {
    this.params = params;
  }

  @Override
  public Type getReturnType() {
    return returnType;
  }

  @Override
  public void setReturnType(Type type) {
    this.returnType = type;
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
    return returnType != null;
  }

  @Override
  public Statement toStmt(TestCase testCase, List<VarReference> args, VarReference returnValue) {
    throw new RuntimeException("Not implemented");
  }

  @Override
  public String toString() {
    var paramsStr = params.stream().map(Param::toString).collect(Collectors.joining(", "));
    String returnStr;
    if (returnsValue()) {
      returnStr = returnType.toString();
    } else {
      returnStr = "()";
    }
    return String.format("%s(%s) -> %s", name, paramsStr, returnStr);
  }
}
