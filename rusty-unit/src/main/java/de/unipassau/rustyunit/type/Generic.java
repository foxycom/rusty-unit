package de.unipassau.rustyunit.type;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.rustyunit.type.traits.Trait;
import de.unipassau.rustyunit.type.traits.std.marker.Sized;
import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.Set;
import java.util.stream.Collectors;

@JsonDeserialize(as = Generic.class)
public class Generic implements Type {

  private String scope;
  private String name;
  private List<Trait> bounds;

  public Generic() {
  }

  public Generic(String name, List<Trait> bounds) {
    this.name = name;
    this.bounds = bounds.stream().filter(bound -> !bound.equals(Sized.getInstance()))
        .toList();
  }

  @Override
  public String getName() {
    return name;
  }

  @Override
  public String fullName() {
    return null;
  }

  @Override
  public String varString() {
    return null;
  }

  @Override
  public boolean canBeSameAs(Type other) {
    /*if (other.isRef()) {
      var ref = other.asRef();
      return canBeSameAs(ref.getInnerType());
    } else if (other.isGeneric()) {
      return equals(other);
    } else {
      return false;
    }*/

    return true;
  }

  @Override
  public boolean canBeIndirectlySameAs(Type other) {
    return canBeSameAs(other);
  }

  @Override
  public List<Type> generics() {
    return Collections.singletonList(this);
  }

  @Override
  public Set<Trait> implementedTraits() {
    throw new RuntimeException("implementedTraits is not implemented");
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
    return binding.getBindingFor(this);
  }

  /*
   * Generics don't change, so there's no point in copying them
   */
  @Override
  public Type copy() {
    return this;
  }

  @Override
  public String encode() {
    var sb = new StringBuilder(name);
    if (!bounds.isEmpty()) {
      sb.append(": ");
      var traits = bounds.stream().map(Trait::toString).collect(Collectors.joining(" + "));
      sb.append(traits);
    }

    return sb.toString();
  }

  @Override
  public boolean isGeneric() {
    return true;
  }

  @Override
  public Generic asGeneric() {
    return this;
  }

  public String getScope() {
    return scope;
  }

  public void setScope(String scope) {
    this.scope = scope;
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    Generic generic = (Generic) o;
    return name.equals(generic.name);
  }

  @Override
  public int hashCode() {
    return Objects.hash(name);
  }

  @Override
  public String toString() {
    return encode();
  }

  public List<Trait> getBounds() {
    return bounds;
  }

  public void setName(String name) {
    this.name = name;
  }

  public void setBounds(List<Trait> bounds) {
    this.bounds = bounds.stream()
        .filter(bound -> !bound.equals(Sized.getInstance())).toList();
  }
}
