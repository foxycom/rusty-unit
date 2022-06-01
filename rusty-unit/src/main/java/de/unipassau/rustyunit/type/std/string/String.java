package de.unipassau.rustyunit.type.std.string;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.rustyunit.test_case.callable.Callable;
import de.unipassau.rustyunit.test_case.callable.StaticMethod;
import de.unipassau.rustyunit.type.AbstractStruct;
import de.unipassau.rustyunit.type.Type;
import de.unipassau.rustyunit.type.traits.Trait;
import de.unipassau.rustyunit.type.traits.std.clone.Clone;
import de.unipassau.rustyunit.type.traits.std.cmp.Eq;
import de.unipassau.rustyunit.type.traits.std.cmp.Ord;
import de.unipassau.rustyunit.type.traits.std.cmp.PartialEq;
import de.unipassau.rustyunit.type.traits.std.cmp.PartialOrd;
import de.unipassau.rustyunit.type.traits.std.fmt.Debug;
import de.unipassau.rustyunit.type.traits.std.hash.Hash;
import de.unipassau.rustyunit.type.traits.std.marker.Copy;
import java.util.Collections;
import java.util.List;
import java.util.Set;

@JsonDeserialize(as = String.class)
public class String extends AbstractStruct {

  public static final Set<Trait> IMPLEMENTED_TRAITS = Set.of(
      Clone.getInstance(),
      Copy.getInstance(),
      Eq.getInstance(),
      PartialEq.getInstance(),
      Hash.getInstance(),
      Ord.getInstance(),
      PartialOrd.getInstance(),
      Debug.getInstance()
  );

  private final List<Callable> methods = List.of(
  );

  public String() {
    super(
        "std::string::String",
        Collections.emptyList(),
        false,
        IMPLEMENTED_TRAITS
    );
  }

  public String(String other) {
    super(other);
  }

  @Override
  public List<Callable> methods() {
    return methods;
  }

  @Override
  public Type copy() {
    return new String(this);
  }
}
