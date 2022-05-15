package de.unipassau.rustyunit.type.std.result;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.rustyunit.test_case.Param;
import de.unipassau.rustyunit.test_case.callable.Callable;
import de.unipassau.rustyunit.test_case.callable.EnumInit;
import de.unipassau.rustyunit.test_case.callable.Method;
import de.unipassau.rustyunit.type.AbstractEnum;
import de.unipassau.rustyunit.type.Generic;
import de.unipassau.rustyunit.type.Type;
import de.unipassau.rustyunit.type.traits.Trait;
import de.unipassau.rustyunit.type.traits.std.clone.Clone;
import de.unipassau.rustyunit.type.traits.std.cmp.Eq;
import de.unipassau.rustyunit.type.traits.std.cmp.Ord;
import de.unipassau.rustyunit.type.traits.std.cmp.PartialEq;
import de.unipassau.rustyunit.type.traits.std.fmt.Debug;
import de.unipassau.rustyunit.type.traits.std.hash.Hash;
import de.unipassau.rustyunit.type.traits.std.iter.IntoIterator;
import de.unipassau.rustyunit.type.traits.std.marker.Copy;
import java.util.Collections;
import java.util.List;
import java.util.Optional;
import java.util.Set;

@JsonDeserialize(as = Result.class)
public class Result extends AbstractEnum {

  public static final Generic T = new Generic("T", Collections.emptyList());
  public static final Generic E = new Generic("E", Collections.emptyList());

  public static final Set<Trait> implementedTraits = Set.of(
      Clone.getInstance(),
      Copy.getInstance(),
      Eq.getInstance(),
      PartialEq.getInstance(),
      Hash.getInstance(),
      Ord.getInstance(),
      IntoIterator.getInstance(),
      Debug.getInstance()
  );

  private final List<Callable> methods  = List.of(
      new Method("unwrap", Collections.emptyList(), List.of(new Param(this, false, null)), generics().get(0), this),
      new EnumInit(this, OK, true),
      new EnumInit(this, ERR, true)
  );
  public static final EnumVariant OK = new TupleEnumVariant("Ok",
      List.of(new Param(T, false, null)));
  public static final EnumVariant ERR = new TupleEnumVariant("Err",
      List.of(new Param(E, false, null)));
  private static final List<Type> stdGenerics = List.of(T, E);

  public Result() {
    super(
        "std::result::Result",
        stdGenerics,
        List.of(OK, ERR),
        false,
        implementedTraits
    );
  }

  public Result(Result other) {
    super(other);
  }

  @Override
  public List<Callable> methods() {
    return methods;
  }

  @Override
  public Type copy() {
    return new Result(this);
  }
}
