package de.unipassau.rustyunit.type.prim;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.rustyunit.Constants;
import de.unipassau.rustyunit.mir.MirAnalysis;
import de.unipassau.rustyunit.test_case.primitive.PrimitiveValue;
import de.unipassau.rustyunit.test_case.primitive.UIntValue;
import de.unipassau.rustyunit.type.traits.Trait;
import de.unipassau.rustyunit.type.traits.std.clone.Clone;
import de.unipassau.rustyunit.type.traits.std.cmp.Eq;
import de.unipassau.rustyunit.type.traits.std.cmp.Ord;
import de.unipassau.rustyunit.type.traits.std.cmp.PartialEq;
import de.unipassau.rustyunit.type.traits.std.cmp.PartialOrd;
import de.unipassau.rustyunit.type.traits.std.default_.Default;
import de.unipassau.rustyunit.type.traits.std.hash.Hash;
import de.unipassau.rustyunit.type.traits.std.marker.Copy;
import de.unipassau.rustyunit.util.Rnd;
import java.math.BigInteger;
import java.util.Set;
import java.util.stream.Collectors;

@JsonDeserialize(as = UInt.class)
public interface UInt extends Prim {

  @Override
  default PrimitiveValue<?> from(String value) {
    var val = new BigInteger(value);
    if (val.compareTo(minValue()) < 0) {
      val = minValue();
    } else if (val.compareTo(maxValue()) > 0) {
      val = maxValue();
    }
    return new UIntValue(val, this);
  }

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

  BigInteger maxValue();

  default BigInteger minValue() {
    return new BigInteger("0");
  }

  @Override
  default Set<Trait> implementedTraits() {
    return implementedTraits;
  }

  @Override
  default PrimitiveValue<BigInteger> random() {
    if (Rnd.get().nextDouble() < Constants.P_CONSTANT_POOL) {
      var possibleConstants = MirAnalysis.constantPool().stream().filter(c -> c.type().equals(this))
          .map(c -> (PrimitiveValue<BigInteger>) c).collect(Collectors.toSet());
      if (possibleConstants.size() >= 2) {
        return Rnd.choice(possibleConstants);
      }
    }

    var value = BigInteger.valueOf((long) (Rnd.get().nextDouble() * Constants.MAX_INT));
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
    public BigInteger maxValue() {
      return BigInteger.valueOf(Byte.MAX_VALUE);
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
    public BigInteger maxValue() {
      return BigInteger.valueOf(Short.MAX_VALUE);
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
    public BigInteger maxValue() {
      return BigInteger.valueOf(Integer.MAX_VALUE);
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
    public BigInteger maxValue() {
      return BigInteger.valueOf(Long.MAX_VALUE);
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
    public BigInteger maxValue() {
      return BigInteger.valueOf(Long.MAX_VALUE);
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
    public BigInteger maxValue() {
      return BigInteger.valueOf(Long.MAX_VALUE);
    }
  }
}
