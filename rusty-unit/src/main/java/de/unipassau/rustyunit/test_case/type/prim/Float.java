package de.unipassau.rustyunit.test_case.type.prim;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import com.google.common.base.Preconditions;
import de.unipassau.rustyunit.Constants;
import de.unipassau.rustyunit.mir.MirAnalysis;
import de.unipassau.rustyunit.test_case.primitive.FloatValue;
import de.unipassau.rustyunit.test_case.primitive.PrimitiveValue;
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
import java.math.BigDecimal;
import java.util.Set;
import java.util.stream.Collectors;

@JsonDeserialize(as = Float.class)
public interface Float extends Prim {

  @Override
  default PrimitiveValue<?> from(String value) {
    var val = new BigDecimal(value);
    Preconditions.checkState(val.compareTo(minValue()) >= 0 && val.compareTo(maxValue()) <= 0);
    return new FloatValue(val, this);
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

  BigDecimal maxValue();

  BigDecimal minValue();

  @Override
  default Set<Trait> implementedTraits() {
    return implementedTraits;
  }

  @Override
  default PrimitiveValue<?> random() {
    if (Rnd.get().nextDouble() < Constants.P_CONSTANT_POOL) {
      var possibleConstants = MirAnalysis.constantPool().stream().filter(c -> c.type().equals(this))
          .map(c -> (PrimitiveValue<BigDecimal>) c).collect(Collectors.toSet());
      if (possibleConstants.size() >= 2) {
        return Rnd.choice(possibleConstants);
      }
    }

    var newValue = new BigDecimal(Rnd.get().nextGaussian() * Constants.MAX_INT);
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
    public BigDecimal maxValue() {
      return BigDecimal.valueOf(java.lang.Float.MAX_VALUE);
    }

    @Override
    public BigDecimal minValue() {
      return BigDecimal.valueOf(-java.lang.Float.MAX_VALUE);
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
    public BigDecimal maxValue() {
      return BigDecimal.valueOf(Double.MAX_VALUE);
    }

    @Override
    public BigDecimal minValue() {
      return BigDecimal.valueOf(-Double.MAX_VALUE);
    }
  }
}
