package de.unipassau.testify.test_case.type.prim;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.test_case.Primitive;
import de.unipassau.testify.test_case.type.Trait;
import de.unipassau.testify.util.Rnd;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

@JsonDeserialize(as = Int.class)
public interface Int extends Prim {

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
      Int8.INSTANCE,
      Int16.INSTANCE,
      Int32.INSTANCE,
      Int64.INSTANCE,
      Int128.INSTANCE,
      ISize.INSTANCE
  );

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
    public Primitive random() {
      var val = Rnd.random().nextLong();
      return new Primitive(val, this);
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
