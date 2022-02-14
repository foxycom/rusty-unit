package de.unipassau.testify.test_case.callable;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.statement.MethodStmt;
import de.unipassau.testify.test_case.statement.Statement;
import de.unipassau.testify.test_case.type.Type;
import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

@JsonDeserialize(as = Method.class)
public class Method implements Callable {

  private List<Param> params;
  @JsonProperty("return_type")
  private Type returnType;
  private Type parent;
  @JsonProperty("src_file_path")
  private String srcFilePath;
  private String name;

  @JsonProperty("is_public")
  private boolean isPublic;

  public Method() {
  }

  public Method(String name, List<Param> params, Type returnType,
      Type parent) {
    this.params = params;
    this.returnType = returnType;
    this.parent = parent;
    this.name = name;
  }

  public Param getSelfParam() {
    return params.get(0);
  }

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
    return parent;
  }

  @Override
  public void setParent(Type parent) {
    this.parent = parent;
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
    if (returnsValue()) {
      Objects.requireNonNull(returnValue);
    }

    return new MethodStmt(Objects.requireNonNull(testCase), Objects.requireNonNull(args),
        returnValue, this);
  }

  @Override
  public boolean isMethod() {
    return true;
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
  public String toString() {
    var paramsStr = params.stream().map(Param::toString).collect(Collectors.joining(", "));
    String returnStr;
    if (returnsValue()) {
      returnStr = returnType.toString();
    } else {
      returnStr = "()";
    }
    return String.format("%s::%s(%s) -> %s", parent.fullName(), name, paramsStr, returnStr);
  }
}
