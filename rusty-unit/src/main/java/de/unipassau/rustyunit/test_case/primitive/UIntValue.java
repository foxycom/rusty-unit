package de.unipassau.rustyunit.test_case.primitive;

import de.unipassau.rustyunit.Constants;
import de.unipassau.rustyunit.test_case.type.prim.Prim;
import de.unipassau.rustyunit.test_case.type.prim.UInt;
import de.unipassau.rustyunit.util.Rnd;
import java.math.BigInteger;
import java.util.Objects;

public class UIntValue implements PrimitiveValue<BigInteger> {
  private final UInt type;
  private BigInteger value;

  public UIntValue(BigInteger value, UInt type) {
    if (value.compareTo(type.minValue()) < 0) {
      value = value.negate().mod(type.maxValue());
    } else if (value.compareTo(type.maxValue()) > 0) {
      value = value.mod(type.maxValue());
    }

    this.type = type;
    this.value = value;
  }

  public PrimitiveValue<BigInteger> negate() {
    throw new RuntimeException("Not with me");
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
    // TODO use constants from source code
    var newValue = BigInteger.valueOf((long) (Rnd.get().nextGaussian() * Constants.MAX_DELTA));
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

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof UIntValue)) {
      return false;
    }
    UIntValue uIntValue = (UIntValue) o;
    return type.equals(uIntValue.type) && value.equals(uIntValue.value);
  }

  @Override
  public int hashCode() {
    return Objects.hash(type, value);
  }
}
