package de.unipassau.testify.test_case.type.prim;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.test_case.Primitive;
import de.unipassau.testify.test_case.type.Trait;
import de.unipassau.testify.util.Rnd;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

@JsonDeserialize(as = UInt.class)
public interface UInt extends Prim {

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
      UInt8.INSTANCE,
      UInt16.INSTANCE,
      UInt32.INSTANCE,
      UInt64.INSTANCE,
      UInt128.INSTANCE,
      USize.INSTANCE
  );


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
    public Primitive random() {
      var value = Rnd.random().nextInt(255) + 1;
      return new Primitive(value, this);
    }

    @Override
    public void setName(String name) {

    }

    @Override
    public String toString() {
      return getName();
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
    public Primitive random() {
      var val = Rnd.random().nextLong() & Long.MAX_VALUE;
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
