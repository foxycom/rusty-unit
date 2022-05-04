package de.unipassau.testify.test_case.type.prim;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.Constants;
import de.unipassau.testify.test_case.primitive.FloatValue;
import de.unipassau.testify.test_case.primitive.PrimitiveValue;
import de.unipassau.testify.test_case.type.traits.Trait;
import de.unipassau.testify.test_case.type.traits.std.clone.Clone;
import de.unipassau.testify.test_case.type.traits.std.cmp.Eq;
import de.unipassau.testify.test_case.type.traits.std.cmp.Ord;
import de.unipassau.testify.test_case.type.traits.std.cmp.PartialEq;
import de.unipassau.testify.test_case.type.traits.std.cmp.PartialOrd;
import de.unipassau.testify.test_case.type.traits.std.default_.Default;
import de.unipassau.testify.test_case.type.traits.std.hash.Hash;
import de.unipassau.testify.test_case.type.traits.std.marker.Copy;
import de.unipassau.testify.util.Rnd;
import java.util.Set;

@JsonDeserialize(as = Float.class)
public interface Float extends Prim {

  Set<Trait> implementedTraits = Set.of(
      Copy.getInstance(),
      Clone.getInstance(),
      Hash.getInstance(),
      Ord.getInstance(),
      PartialOrd.getInstance(),
      Eq.getInstance(),
      PartialEq.getInstance(),
      Default.getInstance()
  );

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
    public String encode() {
      return getName();
    }

    @Override
    public String toString() {
      return encode();
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
      return -java.lang.Float.MAX_VALUE;
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
    public String encode() {
      return getName();
    }

    @Override
    public String toString() {
      return encode();
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
      return -Double.MAX_VALUE;
    }
  }
}
