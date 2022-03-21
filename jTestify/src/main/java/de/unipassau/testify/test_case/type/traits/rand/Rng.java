package de.unipassau.testify.test_case.type.traits.rand;

import de.unipassau.testify.test_case.type.AssociatedType;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.traits.Trait;
import java.util.Collections;
import java.util.List;

public enum Rng implements Trait {
  INSTANCE;

  private static final String NAME = "rand::Rng";
  private static final List<Type> GENERICS = Collections.emptyList();
  private static final List<AssociatedType> ASSOCIATED_TYPES = Collections.emptyList();


  @Override
  public String getName() {
    return NAME;
  }

  @Override
  public List<Type> generics() {
    return GENERICS;
  }

  @Override
  public List<AssociatedType> associatedTypes() {
    return ASSOCIATED_TYPES;
  }
}
