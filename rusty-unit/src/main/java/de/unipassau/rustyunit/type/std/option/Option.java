package de.unipassau.rustyunit.type.std.option;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.rustyunit.test_case.Param;
import de.unipassau.rustyunit.test_case.callable.Callable;
import de.unipassau.rustyunit.test_case.callable.Method;
import de.unipassau.rustyunit.type.AbstractEnum;
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
import de.unipassau.rustyunit.type.traits.std.hash.Hash;
import de.unipassau.rustyunit.type.traits.std.iter.IntoIterator;
import de.unipassau.rustyunit.type.traits.std.marker.Copy;
import java.util.Collections;
import java.util.List;
import java.util.Set;

@JsonDeserialize(as = Option.class)
public class Option extends AbstractEnum {

  public static final Type T = new Generic("T", Collections.emptyList());
  public static final List<Type> GENERICS = List.of(T);
  public static final EnumVariant NONE = new TupleEnumVariant("None", Collections.emptyList());
  public static final EnumVariant SOME = new TupleEnumVariant("Some",
      List.of(new Param(T, false, null)));

  public static final Set<Trait> IMPLEMENTED_TRAITS = Set.of(
      Clone.getInstance(),
      Copy.getInstance(),
      Default.getInstance(),
      Eq.getInstance(),
      PartialEq.getInstance(),
      Hash.getInstance(),
      Ord.getInstance(),
      PartialOrd.getInstance(),
      IntoIterator.getInstance()
  );
  private final List<Callable> methods = List.of(
      new Method("unwrap", Collections.emptyList(), List.of(new Param(this, false, null)), T, this)
  );

  public Option(Type of) {
    super("std::option::Option",
        List.of(of),
        List.of(new TupleEnumVariant("Some", List.of(new Param(of, false, null))), NONE),
        false,
        IMPLEMENTED_TRAITS);
  }

  public Option() {
    super(
        "std::option::Option",
        GENERICS,
        List.of(
            NONE,
            SOME
        ),
        false,
        IMPLEMENTED_TRAITS
    );
  }

  public Option(Option other) {
    super(other);
  }

  @Override
  public Type unwrapType() {
    return generics().get(0);
  }

  @Override
  public Callable unwrapMethod(int at) {
    return new Method("unwrap", Collections.emptyList(), List.of(new Param(this, false, null)),
        generics().get(0), this);
  }

  @Override
  public List<Callable> methods() {
    return methods;
  }

  @Override
  public Type copy() {
    return new Option(this);
  }
}
