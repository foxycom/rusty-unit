package de.unipassau.testify.test_case;

import de.unipassau.testify.test_case.type.Type;

public class Primitive {

  private Object value;
  private Type type;

  public Primitive(Object value, Type type) {
    this.value = value;
    this.type = type;
  }

  public Primitive(Primitive other) {
    this.value = other.value;
    this.type = other.type.copy();
  }

  public Primitive copy() {
    return new Primitive(this);
  }

  public Type type() {
    return type;
  }

  @Override
  public String toString() {
    var primType = type.asPrimitive();
    if (primType.isNumeric()) {
      return String.format("%s%s", value, type.getName());
    } else {
      return value.toString();
    }
  }
}
