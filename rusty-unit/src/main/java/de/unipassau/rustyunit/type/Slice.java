package de.unipassau.rustyunit.type;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.rustyunit.type.traits.Trait;
import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.Set;

@JsonDeserialize(as = Slice.class)
public class Slice implements Type {

  private Type ty;

  public Slice() {

  }

  public Slice(Type ty) {
    this.ty = ty;
  }

  public Slice(Slice other) {
    this.ty = other.ty.copy();
  }

  public Type type() {
    return ty;
  }

  @Override
  public boolean isSlice() {
    return true;
  }

  @Override
  public Slice asSlice() {
    return this;
  }

  @Override
  public String getName() {
    throw new RuntimeException("getName is not implemented");
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
    return ty.varString() + "_slice";
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
    return new Slice(ty.bindGenerics(binding));
  }

  @Override
  public Type copy() {
    return new Slice(this);
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof Slice)) {
      return false;
    }
    Slice slice = (Slice) o;
    return ty.equals(slice.ty);
  }

  @Override
  public int hashCode() {
    return Objects.hash(ty);
  }

  @Override
  public String toString() {
    return encode();
  }

  @Override
  public String encode() {
    return String.format("[%s]", ty);
  }
}
