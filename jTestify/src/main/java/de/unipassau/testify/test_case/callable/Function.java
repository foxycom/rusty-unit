package de.unipassau.testify.test_case.callable;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.var.VarReference;
import de.unipassau.testify.test_case.statement.FunctionStmt;
import de.unipassau.testify.test_case.statement.Statement;
import de.unipassau.testify.test_case.type.Type;
import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

@JsonDeserialize(as = Function.class)
public class Function implements Callable {

  private String name;
  private List<Param> params;

  @JsonProperty("is_public")
  private boolean isPublic;

  @JsonProperty("return_type")
  private Type returnType;

  @JsonProperty("src_file_path")
  private String srcFilePath;

  private List<Type> generics;

  @Override
  public String getName() {
    return String.format("crate::%s", name);
  }

  @Override
  public void setName(String name) {
    this.name = name;
  }

  public List<Type> getGenerics() {
    return generics;
  }

  public void setGenerics(List<Type> generics) {
    this.generics = generics;
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
    return null;
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
  public boolean isPublic() {
    return isPublic;
  }

  @Override
  public void setPublic(boolean isPublic) {
    this.isPublic = isPublic;
  }

  @Override
  public Statement toStmt(TestCase testCase, List<VarReference> args, VarReference returnValue) {
    return new FunctionStmt(testCase, args, returnValue, this);
  }

  @Override
  public String getSrcFilePath() {
    return srcFilePath;
  }

  @Override
  public void setSrcFilePath(String path) {
    this.srcFilePath = path;
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    Function function = (Function) o;
    return isPublic == function.isPublic && name.equals(function.name) && params.equals(
          function.params) && Objects.equals(returnType, function.returnType)
          && generics.equals(function.generics);
  }

  @Override
  public int hashCode() {
    return Objects.hash(name, params, isPublic, returnType, generics);
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
