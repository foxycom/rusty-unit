package de.unipassau.testify.test_case.type;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.test_case.type.traits.rand.Rng;
import de.unipassau.testify.test_case.type.traits.rand.RngCore;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Objects;

@JsonDeserialize(as = Trait.class)
public class Trait {

  private String name;
  private List<Type> generics;

  @JsonProperty("associated_types")
  private List<AssociatedType> associatedTypes;

  public Trait() {
  }

  public Trait(String name) {
    this.name = name;
    this.generics = Collections.emptyList();
    this.associatedTypes = Collections.emptyList();
  }

  public Trait(String name, List<Type> generics, List<AssociatedType> associatedTypes) {
    this.name = name;
    this.generics = generics;
    this.associatedTypes = associatedTypes;
  }

  public Trait(Trait other) {
    this.name = other.name;
    this.generics = new ArrayList<>(other.generics);
    this.associatedTypes = new ArrayList<>(other.associatedTypes);
  }

  public String getName() {
    return name;
  }

  public void setName(String name) {
    this.name = name;
  }

  public List<Type> getGenerics() {
    return generics;
  }

  public void setGenerics(List<Type> generics) {
    this.generics = generics;
  }

  public List<AssociatedType> getAssociatedTypes() {
    return associatedTypes;
  }

  public void setAssociatedTypes(
      List<AssociatedType> associatedTypes) {
    this.associatedTypes = associatedTypes;
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof Trait)) {
      return false;
    }
    Trait trait = (Trait) o;
    return name.equals(trait.name);
  }

  @Override
  public int hashCode() {
    return Objects.hash(name);
  }

  @Override
  public String toString() {
    return name;
  }

  public Trait copy() {
    return new Trait(this);
  }
}
