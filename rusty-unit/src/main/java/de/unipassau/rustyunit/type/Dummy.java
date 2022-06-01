package de.unipassau.rustyunit.type;

import de.unipassau.rustyunit.type.traits.Trait;
import java.util.Collections;
import java.util.List;
import java.util.Set;

public class Dummy implements Type {

  @Override
  public String getName() {
    return "Dummy";
  }

  @Override
  public void setName(String name) {
  }

  @Override
  public String fullName() {
    return "dummy";
  }

  @Override
  public String varString() {
    return "dummy";
  }

  @Override
  public boolean canBeSameAs(Type other) {
    return false;
  }

  @Override
  public boolean canBeIndirectlySameAs(Type other) {
    return false;
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
  }

  @Override
  public Type replaceGenerics(List<Type> generics) {
    return this;
  }

  @Override
  public Type bindGenerics(TypeBinding binding) {
    return this;
  }

  @Override
  public Type copy() {
    return this;
  }

  @Override
  public String encode() {
    return "dummy";
  }
}
