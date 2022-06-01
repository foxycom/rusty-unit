package de.unipassau.rustyunit.test_case.callable;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.rustyunit.test_case.Param;
import de.unipassau.rustyunit.test_case.TestCase;
import de.unipassau.rustyunit.test_case.var.VarReference;
import de.unipassau.rustyunit.test_case.statement.EnumStmt;
import de.unipassau.rustyunit.test_case.statement.Statement;
import de.unipassau.rustyunit.type.AbstractEnum;
import de.unipassau.rustyunit.type.AbstractEnum.EnumVariant;
import de.unipassau.rustyunit.type.Type;
import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

@JsonDeserialize(as = EnumInit.class)
public class EnumInit implements Callable {

  @JsonProperty("return_type")
  private Type type;
  private EnumVariant variant;
  @JsonProperty("is_public")
  private boolean isPublic;
  @JsonProperty("src_file_path")
  private String srcFilePath;

  public EnumInit(AbstractEnum type, EnumVariant variant, boolean isPublic) {
    this.type = type;
    this.variant = variant;
    this.isPublic = isPublic;
  }

  public EnumInit() {
  }

  @Override
  public String getName() {
    return variant.getName();
  }

  @Override
  public void setName(String name) {
    this.variant.setName(name);
  }

  @Override
  public List<Param> getParams() {
    return variant.getParams();
  }

  @Override
  public void setParams(List<Param> params) {
    variant.setParams(params);
  }

  @Override
  public Type getReturnType() {
    return type;
  }

  @Override
  public void setReturnType(Type type) {
    this.type = type.asEnum();
  }

  @Override
  public Type getParent() {
    return type;
  }

  @Override
  public void setParent(Type parent) {
    this.type = parent.asEnum();
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
  public String getSrcFilePath() {
    return srcFilePath;
  }

  @Override
  public void setSrcFilePath(String path) {
    this.srcFilePath = path;
  }

  @Override
  public Statement toStmt(TestCase testCase, List<VarReference> args, VarReference returnValue) {
    return new EnumStmt(Objects.requireNonNull(testCase), Objects.requireNonNull(args),
        Objects.requireNonNull(returnValue), this);
  }

  public EnumVariant getVariant() {
    return variant;
  }

  @Override
  public boolean generates(Type type) {
    return getReturnType().canBeSameAs(type) || variant.getParams().stream()
        .anyMatch(p -> p.type().canBeSameAs(type));
  }

  @Override
  public String toString() {
    var sb = new StringBuilder(type.getName());
    sb.append("::").append(variant.getName());
    if (variant.hasParams()) {
      var variantsStr = variant.getParams().stream().map(Param::toString)
          .collect(Collectors.joining(", "));
      sb.append("(").append(variantsStr).append(")");
    }

    return sb.toString();
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof EnumInit)) {
      return false;
    }
    EnumInit enumInit = (EnumInit) o;
    return isPublic == enumInit.isPublic && type.equals(enumInit.type) && variant.equals(
        enumInit.variant);
  }

  @Override
  public int hashCode() {
    return Objects.hash(type, variant, isPublic);
  }
}
