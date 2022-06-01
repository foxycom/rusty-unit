package de.unipassau.rustyunit.test_case.primitive;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.rustyunit.json.ConstantDeserializer;
import de.unipassau.rustyunit.type.prim.Prim;

@JsonDeserialize(using = ConstantDeserializer.class)
public interface PrimitiveValue<T> {

  default boolean isChar() {
    return false;
  }

  default boolean isString() {
    return false;
  }

  default boolean isInt() {
    return false;
  }

  default boolean isUnsinedInt() {
    return false;
  }

  default boolean isFloat() {
    return false;
  }

  default boolean isBool() {
    return false;
  }

  default CharValue asChar() {
    throw new RuntimeException("Now with me");
  }


  default StringValue asString() {
    throw new RuntimeException("Now with me");
  }

  default IntValue asInt() {
    throw new RuntimeException("Now with me");
  }

  default UIntValue asUnsignedInt() {
    throw new RuntimeException("Now with me");
  }

  default FloatValue asFloat() {
    throw new RuntimeException("Now with me");
  }

  default BoolValue asBool() {
    throw new RuntimeException("Not with me");
  }

  T get();

  Prim type();

  PrimitiveValue<T> delta();

  PrimitiveValue<?> copy();
}
