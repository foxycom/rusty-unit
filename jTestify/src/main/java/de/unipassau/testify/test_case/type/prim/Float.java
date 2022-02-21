package de.unipassau.testify.test_case.type.prim;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.Constants;
import de.unipassau.testify.test_case.primitive.FloatValue;
import de.unipassau.testify.test_case.primitive.PrimitiveValue;
import de.unipassau.testify.test_case.type.Trait;
import de.unipassau.testify.util.Rnd;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

@JsonDeserialize(as = Float.class)
public interface Float extends Prim {

  Set<Trait> implementedTraits = new HashSet<>(Set.of(
      new Trait("std::marker::Copy"),
      new Trait("std::clone::Clone"),
      new Trait("std::hash::Hash"),
      new Trait("std::cmp::Ord"),
      new Trait("std::cmp::PartialOrd"),
      new Trait("std::cmp::Eq"),
      new Trait("std::cmp::PartialEq"),
      new Trait("std::default::Default")
  ));

  int bits();

  double maxValue();

  double minValue();

  @Override
  default Set<Trait> implementedTraits() {
    return implementedTraits;
  }

  @Override
  default PrimitiveValue<?> random() {
    // TODO take value from constant pool
    var newValue = Rnd.get().nextGaussian() * Constants.MAX_INT;
    return new FloatValue(newValue, this);
  }

  @Override
  default boolean isFloat() {
    return true;
  }

  @JsonDeserialize(as = Float32.class)
  enum Float32 implements Float {
    INSTANCE;

    @Override
    public boolean isNumeric() {
      return true;
    }

    @Override
    public String getName() {
      return "f32";
    }

    @Override
    public void setName(String name) {

    }

    @Override
    public String toString() {
      return getName();
    }

    @Override
    public int bits() {
      return 32;
    }

    @Override
    public double maxValue() {
      return java.lang.Float.MAX_VALUE;
    }

    @Override
    public double minValue() {
      return java.lang.Float.MIN_VALUE;
    }
  }

  @JsonDeserialize(as = Float64.class)
  enum Float64 implements Float {

    INSTANCE;

    @Override
    public boolean isNumeric() {
      return true;
    }

    @Override
    public String getName() {
      return "f64";
    }

    @Override
    public void setName(String name) {

    }

    @Override
    public String toString() {
      return getName();
    }

    @Override
    public int bits() {
      return 64;
    }

    @Override
    public double maxValue() {
      return Double.MAX_VALUE;
    }

    @Override
    public double minValue() {
      return Double.MIN_VALUE;
    }
  }
}
