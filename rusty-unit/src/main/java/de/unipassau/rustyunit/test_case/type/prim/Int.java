package de.unipassau.rustyunit.test_case.type.prim;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.rustyunit.Constants;
import de.unipassau.rustyunit.mir.MirAnalysis;
import de.unipassau.rustyunit.test_case.primitive.PrimitiveValue;
import de.unipassau.rustyunit.test_case.primitive.IntValue;
import de.unipassau.rustyunit.test_case.type.traits.Trait;
import de.unipassau.rustyunit.test_case.type.traits.std.clone.Clone;
import de.unipassau.rustyunit.test_case.type.traits.std.cmp.Eq;
import de.unipassau.rustyunit.test_case.type.traits.std.cmp.Ord;
import de.unipassau.rustyunit.test_case.type.traits.std.cmp.PartialEq;
import de.unipassau.rustyunit.test_case.type.traits.std.cmp.PartialOrd;
import de.unipassau.rustyunit.test_case.type.traits.std.default_.Default;
import de.unipassau.rustyunit.test_case.type.traits.std.hash.Hash;
import de.unipassau.rustyunit.test_case.type.traits.std.marker.Copy;
import de.unipassau.rustyunit.util.Rnd;
import java.math.BigInteger;
import java.util.Set;
import java.util.stream.Collectors;

@JsonDeserialize(as = Int.class)
public interface Int extends Prim {

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

  @Override
  default PrimitiveValue<?> from(String value) {
    var val = new BigInteger(value);

    if (val.compareTo(minValue()) < 0) {
      val = minValue();
    } else if (val.compareTo(maxValue()) > 0) {
      val = maxValue();
    }
    return new IntValue(val, this);
  }

  int bits();

  BigInteger maxValue();

  BigInteger minValue();

  @Override
  default Set<Trait> implementedTraits() {
    return implementedTraits;
  }

  @Override
  default PrimitiveValue<?> random() {
    if (Rnd.get().nextDouble() < Constants.P_CONSTANT_POOL) {
      var possibleConstants = MirAnalysis.constantPool().stream().filter(c -> c.type().equals(this))
          .map(c -> (PrimitiveValue<BigInteger>) c).collect(Collectors.toSet());
      if (possibleConstants.size() >= 2) {
        return Rnd.choice(possibleConstants);
      }
    }

    var newValue = BigInteger.valueOf((long) (Rnd.get().nextGaussian() * Constants.MAX_INT));
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
    public BigInteger maxValue() {
      return BigInteger.valueOf(Byte.MAX_VALUE);
    }

    @Override
    public BigInteger minValue() {
      return BigInteger.valueOf(Byte.MIN_VALUE);
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
    public BigInteger maxValue() {
      return BigInteger.valueOf(Short.MAX_VALUE);
    }

    @Override
    public BigInteger minValue() {
      return BigInteger.valueOf(Short.MIN_VALUE);
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
    public BigInteger maxValue() {
      return BigInteger.valueOf(Integer.MAX_VALUE);
    }

    @Override
    public BigInteger minValue() {
      return BigInteger.valueOf(Integer.MIN_VALUE);
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
    public BigInteger maxValue() {
      return BigInteger.valueOf(Long.MAX_VALUE);
    }

    @Override
    public BigInteger minValue() {
      return BigInteger.valueOf(Long.MIN_VALUE);
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
    public BigInteger maxValue() {
      return BigInteger.valueOf(Long.MAX_VALUE);
    }

    @Override
    public BigInteger minValue() {
      return BigInteger.valueOf(Long.MIN_VALUE);
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
    public BigInteger maxValue() {
      return BigInteger.valueOf(Long.MAX_VALUE);
    }

    @Override
    public BigInteger minValue() {
      return BigInteger.valueOf(Long.MIN_VALUE);
    }
  }
}
