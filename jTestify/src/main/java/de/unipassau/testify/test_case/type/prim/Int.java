package de.unipassau.testify.test_case.type.prim;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.Constants;
import de.unipassau.testify.test_case.primitive.PrimitiveValue;
import de.unipassau.testify.test_case.primitive.IntValue;
import de.unipassau.testify.test_case.type.traits.AbstractTrait;
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
import java.util.HashSet;
import java.util.Set;

@JsonDeserialize(as = Int.class)
public interface Int extends Prim {

  Set<Trait> implementedTraits = Set.of(
      Copy.INSTANCE,
      Clone.INSTANCE,
      Hash.INSTANCE,
      Ord.INSTANCE,
      PartialOrd.INSTANCE,
      Eq.INSTANCE,
      PartialEq.INSTANCE,
      Default.INSTANCE
  );

  int bits();

  long maxValue();

  long minValue();

  @Override
  default Set<Trait> implementedTraits() {
    return implementedTraits;
  }

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
    public String encode() {
      return getName();
    }

    @Override
    public String toString() {
      return encode();
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
    public String encode() {
      return getName();
    }

    @Override
    public String toString() {
      return encode();
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
    public String encode() {
      return getName();
    }

    @Override
    public String toString() {
      return encode();
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
    public long maxValue() {
      return Long.MAX_VALUE;
    }

    @Override
    public long minValue() {
      return Long.MIN_VALUE;
    }
  }
}
