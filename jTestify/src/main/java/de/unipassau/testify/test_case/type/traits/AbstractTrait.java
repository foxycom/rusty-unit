package de.unipassau.testify.test_case.type.traits;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.test_case.type.AssociatedType;
import de.unipassau.testify.test_case.type.Type;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Objects;

@JsonDeserialize(as = AbstractTrait.class)
public class AbstractTrait implements Trait {

  private String name;
  private List<Type> generics;

  @JsonProperty("associated_types")
  private List<AssociatedType> associatedTypes;

  public AbstractTrait() {
  }

  public AbstractTrait(String name) {
    this.name = name;
    this.generics = Collections.emptyList();
    this.associatedTypes = Collections.emptyList();
  }

  public AbstractTrait(String name, List<Type> generics, List<AssociatedType> associatedTypes) {
    this.name = name;
    this.generics = generics;
    this.associatedTypes = associatedTypes;
  }

  public AbstractTrait(AbstractTrait other) {
    this.name = other.name;
    this.generics = new ArrayList<>(other.generics);
    this.associatedTypes = new ArrayList<>(other.associatedTypes);
  }

  @Override
  public String getName() {
    return name;
  }

  public void setName(String name) {
    this.name = name;
  }

  @Override
  public List<Type> generics() {
    return generics;
  }

  public void setGenerics(List<Type> generics) {
    this.generics = generics;
  }

  @Override
  public List<AssociatedType> associatedTypes() {
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
    if (!(o instanceof AbstractTrait)) {
      return false;
    }
    AbstractTrait trait = (AbstractTrait) o;
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

  public AbstractTrait copy() {
    return new AbstractTrait(this);
  }
}