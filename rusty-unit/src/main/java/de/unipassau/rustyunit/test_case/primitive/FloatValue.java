package de.unipassau.rustyunit.test_case.primitive;

import de.unipassau.rustyunit.Constants;
import de.unipassau.rustyunit.type.prim.Float;
import de.unipassau.rustyunit.type.prim.Prim;
import de.unipassau.rustyunit.util.Rnd;
import java.math.BigDecimal;
import java.math.RoundingMode;
import java.util.Locale;
import java.util.Objects;

public class FloatValue implements PrimitiveValue<BigDecimal> {

    private final Float type;
    private BigDecimal value;

    public FloatValue(BigDecimal value, Float type) {
        if (value.compareTo(type.minValue()) < 0 || value.compareTo(type.maxValue()) > 0) {
            throw new RuntimeException(
                  String.format("Out of bounds (%f, %f): %f", type.minValue(), type.maxValue(),
                        value));
        }

        this.type = type;
        this.value = value;
    }

    public FloatValue(FloatValue other) {
        this.type = other.type;
        this.value = other.value;
    }

    public PrimitiveValue<BigDecimal> negate() {
        var copy = new FloatValue(this);
        copy.value = copy.value.negate();
        return copy;
    }

    @Override
    public BigDecimal get() {
        return value;
    }

    @Override
    public Prim type() {
        return type;
    }

    @Override
    public PrimitiveValue<BigDecimal> delta() {
        var p = Rnd.get().nextDouble();
        if (p < 1d / 3d) {
            var newValue = value.add(
                BigDecimal.valueOf(Rnd.get().nextGaussian() * Constants.MAX_DELTA));
            return new FloatValue(newValue, type);
        } else if (p < 2d / 3d) {
            var newValue = value.add(BigDecimal.valueOf(Rnd.get().nextGaussian()));
            return new FloatValue(newValue, type);
        } else {
            int precision = Rnd.get().nextInt(15);
            return chopPrecision(precision);
        }
    }

    public PrimitiveValue<BigDecimal> chopPrecision(int precision) {
        var bd = value.setScale(precision, RoundingMode.HALF_EVEN);
        return new FloatValue(bd, type);
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
        return String.format(Locale.US, "%f%s", value, type.getName());
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) {
            return true;
        }
        if (!(o instanceof FloatValue)) {
            return false;
        }
        FloatValue that = (FloatValue) o;
        return type.equals(that.type) && value.equals(that.value);
    }

    @Override
    public int hashCode() {
        return Objects.hash(type, value);
    }
}
