package de.unipassau.rustyunit.test_case.callable;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.rustyunit.test_case.Param;
import de.unipassau.rustyunit.test_case.TestCase;
import de.unipassau.rustyunit.test_case.var.VarReference;
import de.unipassau.rustyunit.test_case.statement.MethodStmt;
import de.unipassau.rustyunit.test_case.statement.Statement;
import de.unipassau.rustyunit.type.Type;
import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

@JsonDeserialize(as = Method.class)
public class Method implements Callable {

  @JsonProperty("global_id")
  private String globalId;


  private List<Param> params;
  @JsonProperty("return_type")
  private Type returnType;

  @JsonProperty("of_trait")
  private String ofTrait;

  private Type parent;

  @Override
  public Method asMethod() {
    return this;
  }

  @JsonProperty("src_file_path")
  private String srcFilePath;
  private String name;
  private List<Type> generics;


  @JsonProperty("is_public")
  private boolean isPublic;

  public Method() {
  }

  public Method(String name, List<Type> generics, List<Param> params, Type returnType,
      Type parent) {
    this.params = params;
    this.returnType = returnType;
    this.parent = parent;
    this.name = name;
    this.generics = generics;
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

  public List<Type> generics() {
    return generics;
  }

  public void setGenerics(List<Type> generics) {
    this.generics = generics;
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

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof Method)) {
      return false;
    }
    Method method = (Method) o;
    return isPublic == method.isPublic && params.equals(method.params) && Objects.equals(
        returnType, method.returnType) && parent.equals(method.parent) && name.equals(method.name);
  }

  @Override
  public int hashCode() {
    return Objects.hash(params, returnType, parent, name, isPublic);
  }
}
