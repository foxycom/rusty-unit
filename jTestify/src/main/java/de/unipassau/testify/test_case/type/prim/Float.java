package de.unipassau.testify.test_case.type.prim;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.test_case.Primitive;
import de.unipassau.testify.test_case.type.Trait;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

@JsonDeserialize(as = Float.class)
public interface Float extends Prim {

  Set<Trait> implementedTraits = new HashSet<>(Set.of(
      new Trait("std::marker::Copy"),
      new Trait("std::clone::Clone"),
      new Trait("std::hash::Hash"),
      new Trait("std::cmp::Ord"),
      new Trait("std::cmp::PartialOrd"),
      new Trait("std::cmp::Eq"),
      new Trait("std::cmp::PartialEq"),
      new Trait("std::default::Default")
  ));

  List<Prim> types = List.of(
      Float32.INSTANCE,
      Float64.INSTANCE
  );

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
    public Primitive random() {
      throw new RuntimeException("Not implemented");
    }

    @Override
    public void setName(String name) {

    }

    @Override
    public String toString() {
      return getName();
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
    public Primitive random() {
      throw new RuntimeException("Not implemented");
    }

    @Override
    public void setName(String name) {

    }

    @Override
    public String toString() {
      return getName();
    }
  }
}
