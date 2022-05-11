package de.unipassau.rustyunit.test_case.type;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.rustyunit.test_case.type.traits.Trait;
import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.Set;

@JsonDeserialize(as = Array.class)
public class Array implements Type {
  private Type ty;
  private int length;

  public Array() {
  }

  public Array(Type ty, int length) {
    this.ty = ty;
    this.length = length;
  }

  public Array(Array other) {
    this.ty = other.ty.copy();
    this.length = other.length;
  }

  public Type type() {
    return ty;
  }

  public void setTy(Type ty) {
    this.ty = ty;
  }

  public int length() {
    return length;
  }

  public void setLength(int length) {
    this.length = length;
  }

  @Override
  public String getName() {
    throw new RuntimeException("Not implemented");
  }

  @Override
  public void setName(String name) {
    throw new RuntimeException("setName is not implemented");
  }

  @Override
  public String fullName() {
    throw new RuntimeException("fullName is not implemented");
  }

  @Override
  public String varString() {
    return ty.varString() + "_array";
  }

  @Override
  public boolean canBeSameAs(Type other) {
    throw new RuntimeException("canBeSameAs is not implemented");
  }

  @Override
  public boolean canBeIndirectlySameAs(Type other) {
    throw new RuntimeException("canBeIndirectlySameAs is not implemented");
  }

  @Override
  public List<Type> generics() {
    return ty.generics();
  }

  @Override
  public Set<Trait> implementedTraits() {
//    throw new RuntimeException("implementedTraits is not implemented");
    return Collections.emptySet();
  }

  @Override
  public void setGenerics(List<Type> generics) {
    throw new RuntimeException("setGenerics is not implemented");
  }

  @Override
  public Type replaceGenerics(List<Type> generics) {
    throw new RuntimeException("replaceGenerics is not implemented");
  }

  @Override
  public Type bindGenerics(TypeBinding binding) {
    return new Array(ty.bindGenerics(binding), length);
  }

  @Override
  public Type copy() {
    return new Array(this);
  }

  @Override
  public String encode() {
    return String.format("[%s; %d]", ty, length);
  }

  @Override
  public boolean isArray() {
    return true;
  }

  @Override
  public Array asArray() {
    return this;
  }

  @Override
  public String toString() {
    return encode();
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    Array array = (Array) o;
    return length == array.length && ty.equals(array.ty);
  }

  @Override
  public int hashCode() {
    return Objects.hash(ty, length);
  }


}
