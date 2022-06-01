package de.unipassau.rustyunit.type.std.boxed;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.rustyunit.test_case.Param;
import de.unipassau.rustyunit.test_case.callable.Callable;
import de.unipassau.rustyunit.test_case.callable.Method;
import de.unipassau.rustyunit.test_case.callable.StaticMethod;
import de.unipassau.rustyunit.type.AbstractStruct;
import de.unipassau.rustyunit.type.Generic;
import de.unipassau.rustyunit.type.Ref;
import de.unipassau.rustyunit.type.Type;
import de.unipassau.rustyunit.type.traits.Trait;
import de.unipassau.rustyunit.type.traits.std.clone.Clone;
import de.unipassau.rustyunit.type.traits.std.cmp.Eq;
import de.unipassau.rustyunit.type.traits.std.cmp.Ord;
import de.unipassau.rustyunit.type.traits.std.cmp.PartialEq;
import de.unipassau.rustyunit.type.traits.std.cmp.PartialOrd;
import de.unipassau.rustyunit.type.traits.std.default_.Default;
import de.unipassau.rustyunit.type.traits.std.fmt.Debug;
import de.unipassau.rustyunit.type.traits.std.hash.Hash;
import de.unipassau.rustyunit.type.traits.std.iter.IntoIterator;
import java.util.Collections;
import java.util.List;
import java.util.Set;

@JsonDeserialize(as = Box.class)
public class Box extends AbstractStruct {

  public static final Generic T = new Generic("T", Collections.emptyList());
  public static final Set<Trait> IMPLEMENTED_TRAITS = Set.of(
      Debug.getInstance(),
      Clone.getInstance(),
      Default.getInstance(),
      Eq.getInstance(),
      PartialEq.getInstance(),
      PartialOrd.getInstance(),
      Ord.getInstance(),
      Hash.getInstance()
  );

  private final List<Callable> methods = List.of(
      new StaticMethod("new", Collections.emptyList(), this, this, null),
      new Method("as_ref", Collections.emptyList(),
          List.of(new Param(new Ref(this, false), false, null)), new Ref(generics().get(0), false),
          this),
      new Method("as_mut", Collections.emptyList(),
          List.of(new Param(new Ref(this, false), false, null)), new Ref(generics().get(0), true), this)
      );

  public Box() {
    super(
        "std::boxed::Box",
        List.of(T),
        false,
        IMPLEMENTED_TRAITS
    );
  }

  public Box(Box other) {
    super(other);
  }

  @Override
  public List<Callable> methods() {
    return methods;
  }

  @Override
  public Type copy() {
    return new Box(this);
  }
}
