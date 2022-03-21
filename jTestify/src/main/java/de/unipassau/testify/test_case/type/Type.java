package de.unipassau.testify.test_case.type;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.json.TypeDeserializer;
import de.unipassau.testify.test_case.type.prim.Prim;
import de.unipassau.testify.test_case.type.traits.Trait;
import java.util.List;
import java.util.Set;

@JsonDeserialize(using = TypeDeserializer.class)
public interface Type {

  String getName();

  void setName(String name);

  String fullName();

  String varString();

  boolean canBeSameAs(Type other);

  boolean canBeIndirectlySameAs(Type other);

  default boolean wraps(Type type) {
    return false;
  }

  default boolean isPrim() {
    return false;
  }

  default boolean isStruct() {
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

  default Struct asStruct() {
    throw new RuntimeException("Not with me");
  }

  default Tuple asTuple() {
    throw new RuntimeException("Not with me");
  }

  default Array asArray() {
    throw new RuntimeException("Not with me");
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

  default String toFullString() {
    return this.toString();
  }

  default String toGenericString() {
    return this.toString();
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
}
