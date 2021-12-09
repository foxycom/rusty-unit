package de.unipassau.testify.test_case;

import com.fasterxml.jackson.annotation.JsonProperty;
import de.unipassau.testify.test_case.type.Type;

public class Param {

  @JsonProperty("ty")
  private Type type;
  private boolean mutable;
  private String name;

  public Param() {
  }

  public Param(Type type, boolean mutable, String name) {
    this.type = type;
    this.mutable = mutable;
    this.name = name;
  }

  public String getName() {
    return name;
  }

  public void setName(String name) {
    this.name = name;
  }

  public boolean isByReference() {
    return type.isRef();
  }

  public boolean isMutable() {
    return mutable;
  }

  public void setMutable(boolean mutable) {
    this.mutable = mutable;
  }

  public Type getType() {
    return type;
  }

  public void setType(Type type) {
    this.type = type;
  }

  public boolean isPrimitive() {
    return type.isPrim();
  }

  public boolean isGeneric() {
    return type.isGeneric();
  }

  @Override
  public String toString() {
    var sb = new StringBuilder();
    if (mutable) {
      sb.append("mut ");
    }

    if (name != null) {
      sb.append(name).append(": ");
    }
    sb.append(type);

    return sb.toString();
  }
}
