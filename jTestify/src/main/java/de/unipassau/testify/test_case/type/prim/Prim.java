package de.unipassau.testify.test_case.type.prim;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.json.PrimDeserializer;
import de.unipassau.testify.test_case.Primitive;
import de.unipassau.testify.test_case.type.Trait;
import de.unipassau.testify.test_case.type.Type;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

@JsonDeserialize(using = PrimDeserializer.class)
public interface Prim extends Type {

  List<Type> generics = Collections.emptyList();
  /*pub static ref STR_TRAITS: HashSet<Trait> = {


  pub static ref TYPES: HashMap<Arc<T>, HashSet<Trait>> = {
    let types = load_types().unwrap();

    let mut vec_traits = HashSet::new();
    vec_traits.insert(Trait::new("std::iter::IntoIterator"));
    vec_traits.insert(Trait::new("std::default::Default"));
    vec_traits.insert(Trait::new("std::cmp::Eq"));
    vec_traits.insert(Trait::new("std::cmp::PartialEq"));
    vec_traits.insert(Trait::new("std::cmp::PartialOrd"));
    vec_traits.insert(Trait::new("std::cmp::Ord"));

    let mut m = HashMap::new();
    for (ty, implementations) in types {
      m.insert(ty, implementations);
    }
    m
  };*/

  @Override
  default boolean isSameType(Type other) {
    return equals(other);
  }

  default boolean isNumeric() {
    return false;
  }

  @Override
  default String getName() {
    return null;
  }

  @Override
  default Prim asPrimitive() {
    return this;
  }

  @Override
  default boolean isPrim() {
    return true;
  }

  @Override
  default String fullName() {
    return getName();
  }

  @Override
  default String varString() {
    return getName();
  }

  @Override
  default List<Type> generics() {
    return generics;
  }

  @Override
  default void setGenerics(List<Type> generics) {
    throw new RuntimeException("Not with me");
  }

  Primitive random();

  static List<Prim> implementorsOf(Trait trait) {
    var implementors = new ArrayList<Prim>();
    if (Str.INSTANCE.implementedTraits().contains(trait)) {
      implementors.add(Str.INSTANCE);
    }

    if (Bool.INSTANCE.implementedTraits().contains(trait)) {
      implementors.add(Bool.INSTANCE);
    }

    if (Int.implementedTraits.contains(trait)) {
      implementors.addAll(Int.types);
    }

    if (UInt.implementedTraits.contains(trait)) {
      implementors.addAll(UInt.types);
    }

    if (Float.implementedTraits.contains(trait)) {
      implementors.addAll(Float.types);
    }

    return implementors;
  }

  @Override
  default Type copy() {
    return this;
  }
}
