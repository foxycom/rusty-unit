package de.unipassau.rustyunit.type.std.vec;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.rustyunit.test_case.Param;
import de.unipassau.rustyunit.test_case.callable.Callable;
import de.unipassau.rustyunit.test_case.callable.Method;
import de.unipassau.rustyunit.test_case.callable.StaticMethod;
import de.unipassau.rustyunit.type.Dummy;
import de.unipassau.rustyunit.type.Generic;
import de.unipassau.rustyunit.type.AbstractStruct;
import de.unipassau.rustyunit.type.Ref;
import de.unipassau.rustyunit.type.Type;
import de.unipassau.rustyunit.type.std.option.Option;
import de.unipassau.rustyunit.type.traits.Trait;
import de.unipassau.rustyunit.type.traits.std.cmp.Eq;
import de.unipassau.rustyunit.type.traits.std.cmp.Ord;
import de.unipassau.rustyunit.type.traits.std.cmp.PartialEq;
import de.unipassau.rustyunit.type.traits.std.cmp.PartialOrd;
import de.unipassau.rustyunit.type.traits.std.default_.Default;
import de.unipassau.rustyunit.type.traits.std.iter.IntoIterator;
import java.util.Collections;
import java.util.List;
import java.util.Set;

@JsonDeserialize(as = Vec.class)
public
class Vec extends AbstractStruct {

  public static final Generic T = new Generic("T", Collections.emptyList());
  public static final Set<Trait> IMPLEMENTED_TRAITS = Set.of(
      Default.getInstance(),
      Eq.getInstance(),
      PartialEq.getInstance(),
      PartialOrd.getInstance(),
      Ord.getInstance(),
      IntoIterator.getInstance()
  );

  private final List<Callable> methods = List.of(
      new Method("push", Collections.emptyList(),
            List.of(new Param(new Ref(this, true), true, null), new Param(generics().isEmpty() ? new Dummy() : generics().get(0), false, null)), null, this),
      new Method("pop", Collections.emptyList(), List.of(new Param(new Ref(this, true), true, null)), new Option(generics().isEmpty() ? new Dummy() : generics().get(0)), this),
      new StaticMethod("new", Collections.emptyList(), this, this, null)
      );

  public Vec() {
    super(
        "std::vec::Vec",
        List.of(T),
        false,
        IMPLEMENTED_TRAITS
    );
  }

  public Vec(Vec other) {
    super(other);
  }

  @Override
  public List<Callable> methods() {
    return methods;
  }

  @Override
  public Type copy() {
    return new Vec(this);
  }
}