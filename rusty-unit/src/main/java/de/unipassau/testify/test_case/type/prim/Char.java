package de.unipassau.testify.test_case.type.prim;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.test_case.primitive.CharValue;
import de.unipassau.testify.test_case.primitive.PrimitiveValue;
import de.unipassau.testify.test_case.type.traits.Trait;
import de.unipassau.testify.util.Rnd;
import java.util.Collections;
import java.util.Set;

@JsonDeserialize(as = Char.class)
public enum Char implements Prim {
  INSTANCE;

  @Override
  public Set<Trait> implementedTraits() {
    return Collections.emptySet();
  }

  @Override
  public String encode() {
    return getName();
  }

  @Override
  public String getName() {
    return "char";
  }

  @Override
  public PrimitiveValue<?> from(String value) {
    throw new RuntimeException("from is not implemented");
  }

  @Override
  public void setName(String name) {

  }

  @Override
  public PrimitiveValue<?> random() {
    return new CharValue(Rnd.nextChar());
  }

  @Override
  public String toString() {
    return encode();
  }
}