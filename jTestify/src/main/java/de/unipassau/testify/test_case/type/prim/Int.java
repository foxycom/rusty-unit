package de.unipassau.testify.test_case.type.prim;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.Constants;
import de.unipassau.testify.test_case.primitive.PrimitiveValue;
import de.unipassau.testify.test_case.primitive.IntValue;
import de.unipassau.testify.test_case.type.Trait;
import de.unipassau.testify.util.Rnd;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

@JsonDeserialize(as = Int.class)
public interface Int extends Prim {

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

  long maxValue();

  long minValue();

  @Override
  default PrimitiveValue<?> random() {
    var newValue = (long) (Rnd.get().nextGaussian() * Constants.MAX_INT);
    return new IntValue(newValue, this);
  }

  @Override
  default boolean isSignedInt() {
    return true;
  }

  @JsonDeserialize(as = Int8.class)
  enum Int8 implements Int {
    INSTANCE;

    @Override
    public boolean isNumeric() {
      return true;
    }

    @Override
    public String getName() {
      return "i8";
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
      return 8;
    }

    @Override
    public long maxValue() {
      return Byte.MAX_VALUE;
    }

    @Override
    public long minValue() {
      return Byte.MIN_VALUE;
    }
  }

  @JsonDeserialize(as = Int16.class)
  enum Int16 implements Int {
    INSTANCE;

    @Override
    public boolean isNumeric() {
      return true;
    }

    @Override
    public String getName() {
      return "i16";
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
      return 16;
    }

    @Override
    public long maxValue() {
      return Short.MAX_VALUE;
    }

    @Override
    public long minValue() {
      return Short.MIN_VALUE;
    }
  }

  @JsonDeserialize(as = Int32.class)
  enum Int32 implements Int {
    INSTANCE;

    @Override
    public boolean isNumeric() {
      return true;
    }

    @Override
    public String getName() {
      return "i32";
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
    public long maxValue() {
      return Integer.MAX_VALUE;
    }

    @Override
    public long minValue() {
      return Integer.MIN_VALUE;
    }
  }

  @JsonDeserialize(as = Int64.class)
  enum Int64 implements Int {
    INSTANCE;

    @Override
    public boolean isNumeric() {
      return true;
    }

    @Override
    public String getName() {
      return "i64";
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
    public long maxValue() {
      return Long.MAX_VALUE;
    }

    @Override
    public long minValue() {
      return Long.MIN_VALUE;
    }
  }

  @JsonDeserialize(as = Int128.class)
  enum Int128 implements Int {
    INSTANCE;

    @Override
    public boolean isNumeric() {
      return true;
    }

    @Override
    public String getName() {
      return "i128";
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
      return 128;
    }

    @Override
    public long maxValue() {
      return Long.MAX_VALUE;
    }

    @Override
    public long minValue() {
      return Long.MIN_VALUE;
    }
  }

  @JsonDeserialize(as = ISize.class)
  enum ISize implements Int {
    INSTANCE;

    @Override
    public boolean isNumeric() {
      return true;
    }

    @Override
    public String getName() {
      return "isize";
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
    public long maxValue() {
      return Long.MAX_VALUE;
    }

    @Override
    public long minValue() {
      return Long.MIN_VALUE;
    }
  }
}
