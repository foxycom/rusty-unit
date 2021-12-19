package de.unipassau.testify.test_case.type.prim;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.test_case.primitive.BoolValue;
import de.unipassau.testify.test_case.primitive.PrimitiveValue;
import de.unipassau.testify.test_case.type.Trait;
import de.unipassau.testify.util.Rnd;
import java.util.Set;

@JsonDeserialize(as = Bool.class)
public enum Bool implements Prim {
  INSTANCE;

  private static final Set<Trait> implementedTraits;

  static {
    implementedTraits = Set.of(
        new Trait("std::clone::Clone"),
        new Trait("std::marker::Copy"),
        new Trait("std::hash::Hash"),
        new Trait("std::cmp::Ord"),
        new Trait("std::cmp::PartialOrd"),
        new Trait("std::cmp::Eq"),
        new Trait("std::cmp::PartialEq"),
        new Trait("std::default::Default")
    );
  }

  @Override
  public void setName(String name) {

  }

  public Set<Trait> implementedTraits() {
    return implementedTraits;
  }

  @Override
  public String getName() {
    return "bool";
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
    return getName();
  }
}
