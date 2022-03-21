package de.unipassau.testify.test_case.type.prim;


import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.Constants;
import de.unipassau.testify.test_case.primitive.PrimitiveValue;
import de.unipassau.testify.test_case.primitive.StringValue;
import de.unipassau.testify.test_case.type.traits.AbstractTrait;
import de.unipassau.testify.test_case.type.traits.Trait;
import de.unipassau.testify.test_case.type.traits.std.clone.Clone;
import de.unipassau.testify.test_case.type.traits.std.cmp.Eq;
import de.unipassau.testify.test_case.type.traits.std.cmp.PartialEq;
import de.unipassau.testify.test_case.type.traits.std.default_.Default;
import de.unipassau.testify.test_case.type.traits.std.hash.Hash;
import java.util.Set;
import org.apache.commons.lang3.RandomStringUtils;

@JsonDeserialize(as = Str.class)
public enum Str implements Prim {
  INSTANCE;

  private static final Set<Trait> implementedTraits;

  static {
    implementedTraits = Set.of(
        Clone.INSTANCE,
        Eq.INSTANCE,
        PartialEq.INSTANCE,
        Hash.INSTANCE,
        Default.INSTANCE
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
  public void setName(String name) {

  }

  @Override
  public boolean isRef() {
    // Primitive string (&str) is always ref
    return true;
  }


  @Override
  public PrimitiveValue<String> random() {
    var string = RandomStringUtils.randomAlphanumeric(0, Constants.MAX_STRING_LENGTH);
    return new StringValue(string);
  }


  @Override
  public String toString() {
    return encode();
  }
}
