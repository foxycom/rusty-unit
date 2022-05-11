package de.unipassau.rustyunit.test_case.type;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.rustyunit.test_case.type.traits.Trait;
import java.util.Collections;
import java.util.List;
import java.util.Locale;
import java.util.Set;

@JsonDeserialize(as = TraitObj.class)
public class TraitObj implements Type {

  private String name;
  @JsonProperty("is_local")
  private boolean isLocal;

  public TraitObj() {

  }

  public TraitObj(TraitObj other) {
    this.name = other.name;
    this.isLocal = other.isLocal;
  }

  @Override
  public String getName() {
    return name;
  }

  @Override
  public void setName(String name) {
    this.name = name;
  }

  public void setIsLocal(boolean isLocal) {
    this.isLocal = isLocal;
  }

  public boolean isLocal() {
    return isLocal;
  }

  @Override
  public String fullName() {
    if (isLocal) {
      return String.format("crate::%s", getName());
    } else {
      return getName();
    }
  }

  @Override
  public String varString() {
    var segments = getName().split("::");
    return segments[segments.length - 1].toLowerCase(Locale.ROOT);
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
    return Collections.emptyList();
  }

  @Override
  public Set<Trait> implementedTraits() {
    throw new RuntimeException("implementedTraits is not implemented");
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
    return new TraitObj(this);
  }

  @Override
  public String encode() {
    return String.format("impl %s", fullName());
  }
}
