package de.unipassau.testify.test_case.type.prim;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.Constants;
import de.unipassau.testify.test_case.primitive.PrimitiveValue;
import de.unipassau.testify.test_case.primitive.UIntValue;
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

@JsonDeserialize(as = UInt.class)
public interface UInt extends Prim {

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

  default long minValue() {
    return 0;
  }

  @Override
  default Set<Trait> implementedTraits() {
    return implementedTraits;
  }

  @Override
  default PrimitiveValue<Long> random() {
    // TODO get contsant pool
    var value = (long) (Rnd.get().nextDouble() * Constants.MAX_INT);
    return new UIntValue(value, this);
  }

  @Override
  default boolean isUnsignedInt() {
    return true;
  }

  @JsonDeserialize(as = UInt8.class)
  enum UInt8 implements UInt {
    INSTANCE;

    @Override
    public boolean isNumeric() {
      return true;
    }

    @Override
    public String getName() {
      return "u8";
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
  }

  @JsonDeserialize(as = UInt16.class)
  enum UInt16 implements UInt {
    INSTANCE;

    @Override
    public boolean isNumeric() {
      return true;
    }

    @Override
    public String getName() {
      return "u16";
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
  }

  @JsonDeserialize(as = UInt32.class)
  enum UInt32 implements UInt {
    INSTANCE;

    @Override
    public boolean isNumeric() {
      return true;
    }

    @Override
    public String getName() {
      return "u32";
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
  }

  @JsonDeserialize(as = UInt64.class)
  enum UInt64 implements UInt {
    INSTANCE;

    @Override
    public boolean isNumeric() {
      return true;
    }

    @Override
    public String getName() {
      return "u64";
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
  }

  @JsonDeserialize(as = UInt128.class)
  enum UInt128 implements UInt {
    INSTANCE;

    @Override
    public boolean isNumeric() {
      return true;
    }

    @Override
    public String getName() {
      return "u128";
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
  }

  @JsonDeserialize(as = USize.class)
  enum USize implements UInt {
    INSTANCE;

    @Override
    public boolean isNumeric() {
      return true;
    }

    @Override
    public String getName() {
      return "usize";
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
  }
}
