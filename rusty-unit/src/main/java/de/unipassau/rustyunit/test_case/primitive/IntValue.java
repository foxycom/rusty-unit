package de.unipassau.rustyunit.test_case.primitive;

import de.unipassau.rustyunit.Constants;
import de.unipassau.rustyunit.type.prim.Int;
import de.unipassau.rustyunit.type.prim.Prim;
import de.unipassau.rustyunit.util.Rnd;
import java.math.BigInteger;
import java.util.Objects;

public class IntValue implements PrimitiveValue<BigInteger> {

  private final Int type;

  private BigInteger value;

  public IntValue(BigInteger value, Int type) {
    if (value.compareTo(type.minValue()) < 0) {
      value = value.negate().mod(type.maxValue());
    } else if (value.compareTo(type.maxValue()) > 0) {
      value = value.mod(type.maxValue()).negate();
    }

    this.type = type;
    this.value = value;
  }

  public IntValue(IntValue other) {
    this.type = other.type;
    this.value = other.value;
  }

  public PrimitiveValue<BigInteger> negate() {
    var copy = new IntValue(this);
    copy.value = copy.value.negate();
    return copy;
  }

  @Override
  public BigInteger get() {
    return value;
  }

  @Override
  public Prim type() {
    return type;
  }

  @Override
  public PrimitiveValue<BigInteger> delta() {
    BigInteger delta = BigInteger.valueOf((long) (Math.floor(Rnd.get().nextGaussian() * Constants.MAX_DELTA)));
    return new IntValue(value.add(delta), type);
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

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof IntValue)) {
      return false;
    }
    IntValue intValue = (IntValue) o;
    return type.equals(intValue.type) && value.equals(intValue.value);
  }

  @Override
  public int hashCode() {
    return Objects.hash(type, value);
  }
}
