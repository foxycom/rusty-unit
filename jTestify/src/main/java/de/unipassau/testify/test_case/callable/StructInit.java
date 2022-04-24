package de.unipassau.testify.test_case.callable;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.statement.Statement;
import de.unipassau.testify.test_case.statement.StructInitStmt;
import de.unipassau.testify.test_case.type.Type;
import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

@JsonDeserialize(as = StructInit.class)
public class StructInit implements Callable {

  private List<Param> params;
  @JsonProperty("return_type")
  private Type returnType;
  @JsonProperty("src_file_path")
  private String srcFilePath;

  @JsonProperty("is_public")
  private boolean isPublic;

  public StructInit() {
  }

  public StructInit(List<Param> params, Type returnType, String srcFilePath) {
    this.params = params;
    this.returnType = returnType;
    this.srcFilePath = srcFilePath;
  }

  @Override
  public String getName() {
    return returnType.getName();
  }

  @Override
  public void setName(String name) {
    returnType.setName(name);
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
    returnType = type;
  }

  @Override
  public Type getParent() {
    return returnType;
  }

  @Override
  public void setParent(Type parent) {
    throw new RuntimeException("Not implemented");
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
    return new StructInitStmt(Objects.requireNonNull(testCase), Objects.requireNonNull(args),
        Objects.requireNonNull(returnValue), this);
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
    var sb = new StringBuilder();
    sb.append(returnType.fullName()).append(" {");
    String fields = params.stream().map(Param::toString).collect(Collectors.joining(", "));
    sb.append(fields).append("}");
    return sb.toString();
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof StructInit)) {
      return false;
    }
    StructInit that = (StructInit) o;
    return isPublic == that.isPublic && params.equals(that.params) && returnType.equals(
        that.returnType);
  }

  @Override
  public int hashCode() {
    return Objects.hash(params, returnType, isPublic);
  }
}
