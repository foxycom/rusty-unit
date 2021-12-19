package de.unipassau.testify.test_case.primitive;

import de.unipassau.testify.Constants;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.prim.Float;
import de.unipassau.testify.test_case.type.prim.Prim;
import de.unipassau.testify.util.Rnd;
import java.math.BigDecimal;
import java.math.RoundingMode;

public class FloatValue implements PrimitiveValue<Double> {
  private final Float type;
  private double value;

  public FloatValue(double value, Float type) {
    if (value < type.minValue() || value > type.maxValue()) {
      throw new RuntimeException("Out of bounds");
    }

    this.type = type;
    this.value = value;
  }

  public FloatValue(FloatValue other) {
    this.type = other.type;
    this.value = other.value;
  }

  public PrimitiveValue<Double> negate() {
    var copy = new FloatValue(this);
    copy.value = -value;
    return copy;
  }

  @Override
  public Double get() {
    return value;
  }

  @Override
  public Prim type() {
    return type;
  }

  @Override
  public PrimitiveValue<Double> delta() {
    var p = Rnd.get().nextDouble();
    if (p < 1d / 3d) {
      var newValue = value + Rnd.get().nextGaussian() * Constants.MAX_DELTA;
      return new FloatValue(newValue, type);
    } else if (p < 2d / 3d) {
      var newValue = value + Rnd.get().nextGaussian();
      return new FloatValue(newValue, type);
    } else {
      int precision = Rnd.get().nextInt(15);
      return chopPrecision(precision);
    }
  }

  public PrimitiveValue<Double> chopPrecision(int precision) {
    var bd = new BigDecimal(value).setScale(precision, RoundingMode.HALF_EVEN);
    return new FloatValue(bd.doubleValue(), type);
  }

  @Override
  public PrimitiveValue<?> copy() {
    return new FloatValue(value, type);
  }

  @Override
  public boolean isFloat() {
    return true;
  }

  @Override
  public FloatValue asFloat() {
    return this;
  }

  @Override
  public String toString() {
    return String.format("%f%s", value, type.getName());
  }
}
