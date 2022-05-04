package de.unipassau.testify.test_case;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.TypeBinding;
import java.util.Objects;

@JsonDeserialize(as = Param.class)
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

  public Param(Param other) {
    this.type = other.getType().copy();
    this.mutable = other.mutable;
    this.name = other.name;
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

  public Param bindGenerics(TypeBinding binding) {
    if (binding == null) {
      throw new RuntimeException("Something is wrong");
    }

    var copy = new Param(this);
    if (type.isGeneric()) {
      copy.type = binding.getBindingFor(type.asGeneric());
    } else {
      copy.type = type.bindGenerics(binding);
    }

    return copy;
  }

  public boolean isPrimitive() {
    return type.isPrim();
  }

  public boolean isGeneric() {
    return type.isGeneric();
  }

  public Param copy() {
    return new Param(this);
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

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    Param param = (Param) o;
    return mutable == param.mutable && type.equals(param.type) && Objects.equals(name,
          param.name);
  }

  @Override
  public int hashCode() {
    return Objects.hash(type, mutable, name);
  }
}
