package de.unipassau.rustyunit.test_case.type.prim;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.rustyunit.json.PrimDeserializer;
import de.unipassau.rustyunit.test_case.primitive.PrimitiveValue;
import de.unipassau.rustyunit.test_case.type.Type;
import de.unipassau.rustyunit.test_case.type.TypeBinding;
import de.unipassau.rustyunit.test_case.type.prim.Float.Float32;
import de.unipassau.rustyunit.test_case.type.prim.Float.Float64;
import de.unipassau.rustyunit.test_case.type.prim.Int.ISize;
import de.unipassau.rustyunit.test_case.type.prim.Int.Int128;
import de.unipassau.rustyunit.test_case.type.prim.Int.Int16;
import de.unipassau.rustyunit.test_case.type.prim.Int.Int32;
import de.unipassau.rustyunit.test_case.type.prim.Int.Int64;
import de.unipassau.rustyunit.test_case.type.prim.Int.Int8;
import de.unipassau.rustyunit.test_case.type.prim.UInt.UInt128;
import de.unipassau.rustyunit.test_case.type.prim.UInt.UInt16;
import de.unipassau.rustyunit.test_case.type.prim.UInt.UInt32;
import de.unipassau.rustyunit.test_case.type.prim.UInt.UInt64;
import de.unipassau.rustyunit.test_case.type.prim.UInt.UInt8;
import de.unipassau.rustyunit.test_case.type.prim.UInt.USize;
import de.unipassau.rustyunit.test_case.type.traits.Trait;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

@JsonDeserialize(using = PrimDeserializer.class)
public interface Prim extends Type {

  List<Type> generics = Collections.emptyList();

  @Override
  default boolean canBeSameAs(Type other) {
    return equals(other) || other.isGeneric();
  }

  @Override
  default boolean canBeIndirectlySameAs(Type other) {
    return canBeSameAs(other);
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

  default boolean isSignedInt() {
    return false;
  }

  default boolean isFloat() {
    return false;
  }

  default boolean isUnsignedInt() {
    return false;
  }

  default boolean isString() {
    return false;
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

  @Override
  default Type replaceGenerics(List<Type> generics) {
    return this;
  }

  PrimitiveValue<?> from(String value);

  PrimitiveValue<?> random();

  static List<Prim> implementorsOf(Trait trait) {
    var implementors = new ArrayList<Prim>();
    if (Str.INSTANCE.implementedTraits().contains(trait)) {
      implementors.add(Str.INSTANCE);
    }

    if (Bool.INSTANCE.implementedTraits().contains(trait)) {
      implementors.add(Bool.INSTANCE);
    }

    if (Int.implementedTraits.contains(trait)) {
      implementors.addAll(List.of(
          Int8.INSTANCE,
          Int16.INSTANCE,
          Int32.INSTANCE,
          Int64.INSTANCE,
          Int128.INSTANCE,
          ISize.INSTANCE
      ));
    }

    if (UInt.implementedTraits.contains(trait)) {
      implementors.addAll(List.of(
          UInt8.INSTANCE,
          UInt16.INSTANCE,
          UInt32.INSTANCE,
          UInt64.INSTANCE,
          UInt128.INSTANCE,
          USize.INSTANCE
      ));
    }

    if (Float.implementedTraits.contains(trait)) {
      implementors.addAll(List.of(
          Float32.INSTANCE,
          Float64.INSTANCE
      ));
    }

    return implementors;
  }

  @Override
  default Type bindGenerics(TypeBinding binding) {
    return this;
  }

  @Override
  default Type copy() {
    return this;
  }
}
