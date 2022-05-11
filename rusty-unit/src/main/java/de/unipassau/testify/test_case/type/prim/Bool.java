package de.unipassau.testify.test_case.type.prim;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.test_case.primitive.BoolValue;
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

@JsonDeserialize(as = Bool.class)
public enum Bool implements Prim {
  INSTANCE;

  private static final Set<Trait> implementedTraits;

  static {
    implementedTraits = Set.of(
        Clone.getInstance(),
        Copy.getInstance(),
        Hash.getInstance(),
        Ord.getInstance(),
        PartialOrd.getInstance(),
        Eq.getInstance(),
        PartialEq.getInstance(),
        Default.getInstance()
    );
  }

  @Override
  public void setName(String name) {

  }

  public Set<Trait> implementedTraits() {
    return implementedTraits;
  }

  @Override
  public String encode() {
    return getName();
  }

  @Override
  public String getName() {
    return "bool";
  }

  @Override
  public PrimitiveValue<?> from(String value) {
    if (value.equals("0")) {
      return new BoolValue(false);
    } else {
      return new BoolValue(true);
    }
  }

  @Override
  public PrimitiveValue<?> random() {
    var r = Rnd.get().nextDouble();
    if (r < 0.5) {
      return new BoolValue(true);
    } else {
      return new BoolValue(false);
    }
  }

  @Override
  public String toString() {
    return encode();
  }
}
