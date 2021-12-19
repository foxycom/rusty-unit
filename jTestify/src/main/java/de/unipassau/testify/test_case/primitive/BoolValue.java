package de.unipassau.testify.test_case.primitive;

import de.unipassau.testify.test_case.type.prim.Bool;
import de.unipassau.testify.test_case.type.prim.Prim;

public class BoolValue implements PrimitiveValue<Boolean> {
  private final Prim type = Bool.INSTANCE;

  private boolean value;

  public BoolValue(boolean value) {
    this.value = value;
  }

  @Override
  public Boolean get() {
    return value;
  }

  @Override
  public Prim type() {
    return type;
  }

  @Override
  public PrimitiveValue<Boolean> delta() {
    return new BoolValue(!value);
  }

  @Override
  public PrimitiveValue<?> copy() {
    return new BoolValue(value);
  }

  @Override
  public boolean isBool() {
    return true;
  }

  @Override
  public BoolValue asBool() {
    return this;
  }

  @Override
  public String toString() {
    return String.format("%s", value);
  }
}