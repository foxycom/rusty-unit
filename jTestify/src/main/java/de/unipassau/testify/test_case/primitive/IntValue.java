package de.unipassau.testify.test_case.primitive;

import de.unipassau.testify.Constants;
import de.unipassau.testify.test_case.type.prim.Int;
import de.unipassau.testify.test_case.type.prim.Prim;
import de.unipassau.testify.util.Rnd;

public class IntValue implements PrimitiveValue<Long> {

  private final Int type;
  private long value;

  public IntValue(long value, Int type) {
    if (value < type.minValue()) {
      value = (-value) % type.maxValue();
    } else if (value < type.maxValue()) {
      value = -(value % type.maxValue());
    }

    this.type = type;
    this.value = value;
  }

  public IntValue(IntValue other) {
    this.type = other.type;
    this.value = other.value;
  }

  public PrimitiveValue<Long> negate() {
    var copy = new IntValue(this);
    copy.value = -value;
    return copy;
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
    long delta = (long) Math.floor(Rnd.get().nextGaussian() * Constants.MAX_DELTA);
    return new IntValue(value + delta, type);
  }

  @Override
  public PrimitiveValue<?> copy() {
    return new IntValue(value, type);
  }

  @Override
  public IntValue asInt() {
    return this;
  }

  @Override
  public boolean isInt() {
    return true;
  }

  @Override
  public String toString() {
    return String.format("%d%s", value, type.getName());
  }
}
