package de.unipassau.testify.test_case.type.prim;


import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.test_case.Primitive;
import de.unipassau.testify.test_case.type.Trait;
import java.util.Set;

@JsonDeserialize(as = Str.class)
public enum Str implements Prim {
  INSTANCE;

  private static final Set<Trait> implementedTraits;

  static {
    implementedTraits = Set.of(
        new Trait("std::clone::Clone"),
        new Trait("std::cmp::Eq"),
        new Trait("std::cmp::PartialEq"),
        new Trait("std::hash::Hash"),
        new Trait("std::default::Default")
    );
  }

  public Set<Trait> implementedTraits() {
    return implementedTraits;
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
  public Primitive random() {
    throw new RuntimeException("Not implemented");
  }


  @Override
  public String toString() {
    return getName();
  }
}
