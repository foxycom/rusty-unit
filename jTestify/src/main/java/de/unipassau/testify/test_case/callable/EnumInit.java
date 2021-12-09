package de.unipassau.testify.test_case.callable;

import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.statement.EnumStmt;
import de.unipassau.testify.test_case.statement.Statement;
import de.unipassau.testify.test_case.type.Enum;
import de.unipassau.testify.test_case.type.Enum.EnumVariant;
import de.unipassau.testify.test_case.type.Type;
import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

public class EnumInit implements Callable {

  private Enum type;
  private EnumVariant variant;

  public EnumInit(Enum type, EnumVariant variant) {
    this.type = type;
    this.variant = variant;
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
  public Statement toStmt(TestCase testCase, List<VarReference> args, VarReference returnValue) {
    return new EnumStmt(Objects.requireNonNull(testCase), Objects.requireNonNull(args),
        Objects.requireNonNull(returnValue), this);
  }

  public EnumVariant getVariant() {
    return variant;
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
}
