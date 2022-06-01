package de.unipassau.rustyunit.type;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.rustyunit.json.TypeDeserializer;
import de.unipassau.rustyunit.test_case.callable.Callable;
import de.unipassau.rustyunit.type.prim.Prim;
import de.unipassau.rustyunit.type.traits.Trait;
import java.util.Collections;
import java.util.List;
import java.util.Optional;
import java.util.Set;

@JsonDeserialize(using = TypeDeserializer.class)
public interface Type {

  String getName();

  void setName(String name);

  String fullName();

  String varString();

  boolean canBeSameAs(Type other);

  boolean canBeIndirectlySameAs(Type other);

  default Optional<Integer> wraps(Type type) {
    return Optional.empty();
  }

  default Type unwrapType() {
    throw new RuntimeException("Not with me");
  }

  default Callable unwrapMethod(int at) {
    throw new RuntimeException("Not with me");
  }

  default boolean isPrim() {
    return false;
  }

  default boolean isStruct() {
    return false;
  }

  default boolean isTraitObj() {
    return false;
  }

  default boolean isEnum() {
    return false;
  }

  default boolean isGeneric() {
    return false;
  }

  default boolean isRef() {
    return false;
  }

  default boolean isTuple() {
    return false;
  }

  default boolean isArray() {
    return false;
  }

  default boolean isSlice() {
    return false;
  }

  default boolean isFn() {
    return false;
  }

  default Struct asStruct() {
    throw new RuntimeException("Not with me");
  }

  default TraitObj asTraitObj() {
    throw new RuntimeException("Not with me");
  }

  default Tuple asTuple() {
    throw new RuntimeException("Not with me");
  }

  default Array asArray() {
    throw new RuntimeException("Not with me");
  }

  default Slice asSlice() {
    throw new RuntimeException("Not implemented");
  }

  default Enum asEnum() {
    throw new RuntimeException("Not with me");
  }

  default Prim asPrimitive() {
    throw new RuntimeException("Not with me");
  }

  default Generic asGeneric() {
    throw new RuntimeException("Not with me");
  }

  default Ref asRef() {
    throw new RuntimeException("Not with me");
  }

  default Fn asFn() {
    throw new RuntimeException("Not with me");
  }

  default String toFullString() {
    return this.toString();
  }

  default String toGenericString() {
    return encode();
  }

  List<Type> generics();

  Set<Trait> implementedTraits();

  /**
   * Only intended for Jackson unmarshalling.
   */
  void setGenerics(List<Type> generics);

  Type replaceGenerics(List<Type> generics);

  Type bindGenerics(TypeBinding binding);

  Type copy();

  String encode();

  default List<Callable> methods() {
    return Collections.emptyList();
  }
}
