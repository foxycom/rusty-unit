package de.unipassau.testify.test_case.type;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import java.util.Objects;

@JsonDeserialize(as = Trait.class)
public class Trait {
  private String name;

  public Trait() {
  }

  public Trait(String name) {
    this.name = name;
  }

  public String getName() {
    return name;
  }

  public void setName(String name) {
    this.name = name;
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
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
}
