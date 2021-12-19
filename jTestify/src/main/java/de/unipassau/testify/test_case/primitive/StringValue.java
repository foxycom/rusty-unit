package de.unipassau.testify.test_case.primitive;

import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.prim.Prim;
import de.unipassau.testify.test_case.type.prim.Str;

public class StringValue implements PrimitiveValue<String> {
  private final Prim type = Str.INSTANCE;
  private String value;

  public StringValue(String value) {
    this.value = value;
  }

  @Override
  public String get() {
    return value;
  }

  @Override
  public Prim type() {
    return type;
  }

  @Override
  public PrimitiveValue<String> delta() {
    throw new RuntimeException("Not implemented yet");
  }

  @Override
  public PrimitiveValue<?> copy() {
    return new StringValue(value);
  }

  @Override
  public boolean isString() {
    return true;
  }

  @Override
  public StringValue asString() {
    return this;
  }

  @Override
  public String toString() {
    return String.format("\"%s\"", value);
  }
}
