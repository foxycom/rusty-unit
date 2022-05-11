package de.unipassau.testify.test_case.type.prim;


import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import com.google.common.base.Preconditions;
import de.unipassau.testify.Constants;
import de.unipassau.testify.mir.MirAnalysis;
import de.unipassau.testify.test_case.primitive.PrimitiveValue;
import de.unipassau.testify.test_case.primitive.StringValue;
import de.unipassau.testify.test_case.type.traits.Trait;
import de.unipassau.testify.test_case.type.traits.std.clone.Clone;
import de.unipassau.testify.test_case.type.traits.std.cmp.Eq;
import de.unipassau.testify.test_case.type.traits.std.cmp.PartialEq;
import de.unipassau.testify.test_case.type.traits.std.default_.Default;
import de.unipassau.testify.test_case.type.traits.std.hash.Hash;
import de.unipassau.testify.util.Rnd;
import java.math.BigInteger;
import java.util.Set;
import java.util.stream.Collectors;
import org.apache.commons.lang3.RandomStringUtils;

@JsonDeserialize(as = Str.class)
public enum Str implements Prim {
  INSTANCE;

  private static final Set<Trait> implementedTraits;

  static {
    implementedTraits = Set.of(
        Clone.getInstance(),
        Eq.getInstance(),
        PartialEq.getInstance(),
        Hash.getInstance(),
        Default.getInstance()
    );
  }

  public Set<Trait> implementedTraits() {
    return implementedTraits;
  }

  @Override
  public String encode() {
    return "&" + getName();
  }

  @Override
  public String getName() {
    return "str";
  }

  @Override
  public boolean isString() {
    return true;
  }

  @Override
  public PrimitiveValue<?> from(String value) {
    return new StringValue(value);
  }

  @Override
  public void setName(String name) {

  }

  @Override
  public boolean isRef() {
    // Primitive string (&str) is always ref
    return true;
  }


  @Override
  public PrimitiveValue<String> random() {
    if (Rnd.get().nextDouble() < Constants.P_CONSTANT_POOL) {
      var possibleConstants = MirAnalysis.constantPool().stream().filter(c -> c.type().equals(this))
          .map(c -> (PrimitiveValue<String>) c).collect(Collectors.toSet());
      Preconditions.checkState(possibleConstants.size() >= 5);
      return Rnd.choice(possibleConstants);
    }

    var string = RandomStringUtils.randomAlphanumeric(0, Constants.MAX_STRING_LENGTH);
    return new StringValue(string);
  }


  @Override
  public String toString() {
    return encode();
  }

}
