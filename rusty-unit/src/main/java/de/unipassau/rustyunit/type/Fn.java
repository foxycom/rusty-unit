package de.unipassau.rustyunit.type;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.rustyunit.type.traits.Trait;
import java.util.Collections;
import java.util.List;
import java.util.Set;

@JsonDeserialize(as = Fn.class)
public class Fn implements Type {

  @Override
  public String getName() {
    return "<anonymous>";
  }

  @Override
  public void setName(String name) {
    throw new RuntimeException("setName is not implemented");
  }

  @Override
  public boolean isFn() {
    return true;
  }

  @Override
  public Fn asFn() {
    return this;
  }

  @Override
  public String fullName() {
    throw new RuntimeException("fullName is not implemented");
  }

  @Override
  public String varString() {
    return "fn";
  }

  @Override
  public boolean canBeSameAs(Type other) {
    return false;
  }

  @Override
  public boolean canBeIndirectlySameAs(Type other) {
    throw new RuntimeException("canBeIndirectlySameAs is not implemented");
  }

  @Override
  public List<Type> generics() {
    return Collections.emptyList();
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
    return this;
  }

  @Override
  public Type copy() {
    throw new RuntimeException("copy is not implemented");
  }

  @Override
  public String encode() {
    return "fn() -> None";
  }
}
