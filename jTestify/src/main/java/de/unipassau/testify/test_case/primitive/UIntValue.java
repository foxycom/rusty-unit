package de.unipassau.testify.test_case.primitive;

import de.unipassau.testify.Constants;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.prim.Prim;
import de.unipassau.testify.test_case.type.prim.Str;
import de.unipassau.testify.test_case.type.prim.UInt;
import de.unipassau.testify.util.Rnd;

public class UIntValue implements PrimitiveValue<Long> {
  private final UInt type;
  private long value;

  public UIntValue(long value, UInt type) {
    if (value < type.minValue()) {
      value = (-value) % type.maxValue();
    } else if (value > type.maxValue()) {
      value = value % type.maxValue();
    }

    this.type = type;
    this.value = value;
  }

  public PrimitiveValue<Long> negate() {
    throw new RuntimeException("Not with me");
  }

  @Override
  public Long get() {
    return value;
  }

  @Override
  public Prim type() {
    return type;
  }

  @Override
  public PrimitiveValue<Long> delta() {
    // TODO use constants from source code
    var newValue = (long) (Rnd.get().nextGaussian() * Constants.MAX_DELTA);
    return new UIntValue(newValue, type);
  }

  @Override
  public PrimitiveValue<?> copy() {
    return new UIntValue(value, type);
  }

  @Override
  public boolean isUnsinedInt() {
    return true;
  }

  @Override
  public UIntValue asUnsignedInt() {
    return this;
  }

  @Override
  public String toString() {
    return String.format("%d%s", value, type.getName());
  }
}
