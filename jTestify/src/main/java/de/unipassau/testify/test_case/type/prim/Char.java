package de.unipassau.testify.test_case.type.prim;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.test_case.Primitive;
import de.unipassau.testify.test_case.type.Trait;
import java.util.Collections;
import java.util.Set;

@JsonDeserialize(as = Char.class)
public enum Char implements Prim {
  INSTANCE;

  Set<Trait> implementedTraits() {
    return Collections.emptySet();
  }

  @Override
  public String getName() {
    return "char";
  }

  @Override
  public void setName(String name) {

  }

  @Override
  public Primitive random() {
    throw new RuntimeException("Not implemented");
  }

  @Override
  public String toString() {
    return getName();
  }
}
