package de.unipassau.rustyunit.test_case.callable;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.rustyunit.test_case.Param;
import de.unipassau.rustyunit.test_case.TestCase;
import de.unipassau.rustyunit.test_case.var.VarReference;
import de.unipassau.rustyunit.test_case.statement.Statement;
import de.unipassau.rustyunit.test_case.statement.StaticMethodStmt;
import de.unipassau.rustyunit.type.Type;
import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

@JsonDeserialize(as = StaticMethod.class)
public class StaticMethod implements Callable {

  @JsonProperty("global_id")
  private String globalId;

  private String name;
  private List<Param> params;
  @JsonProperty("return_type")
  private Type returnType;
  private Type parent;
  @JsonProperty("src_file_path")
  private String srcFilePath;
  private List<Type> generics = Collections.emptyList();

  @JsonProperty("is_public")
  private boolean isPublic;

  @JsonProperty("of_trait")
  private String ofTrait;

  public StaticMethod() {
  }

  public StaticMethod(String name, List<Param> params,
      Type returnType, Type parent, String srcFilePath) {
    this.name = name;
    this.params = params;
    this.returnType = returnType;
    this.parent = parent;
    this.srcFilePath = srcFilePath;
  }

  @Override
  public String globalId() {
    return globalId;
  }

  public void setGlobalId(String globalId) {
    this.globalId = globalId;
  }

  public void setOfTrait(String ofTrait) {
    this.ofTrait = ofTrait;
  }

  public String ofTrait() {
    return ofTrait;
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
  public String getName() {
    return name;
  }

  @Override
  public void setName(String name) {
    this.name = name;
  }

  @Override
  public Type getReturnType() {
    return returnType;
  }

  @Override
  public void setReturnType(Type returnType) {
    this.returnType = returnType;
  }

  @Override
  public Type getParent() {
    return parent;
  }

  public List<Type> generics() {
    return generics;
  }

  public void setGenerics(List<Type> generics) {
    this.generics = generics;
  }

  @Override
  public Statement toStmt(TestCase testCase, List<VarReference> args, VarReference returnValue) {
    if (returnsValue()) {
      Objects.requireNonNull(returnValue);
    }

    return new StaticMethodStmt(Objects.requireNonNull(testCase), Objects.requireNonNull(args),
        returnValue, this);
  }

  @Override
  public boolean isStaticMethod() {
    return true;
  }

  @Override
  public StaticMethod asStaticMethod() {
    return this;
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

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof StaticMethod)) {
      return false;
    }
    StaticMethod that = (StaticMethod) o;
    return isPublic == that.isPublic && name.equals(that.name) && params.equals(that.params)
        && Objects.equals(returnType, that.returnType) && parent.equals(that.parent);
  }

  @Override
  public int hashCode() {
    return Objects.hash(name, params, returnType, parent, isPublic);
  }
}
