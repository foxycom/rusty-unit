package de.unipassau.testify.test_case.type.traits.std.marker;

import de.unipassau.testify.test_case.type.AssociatedType;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.traits.Trait;
import java.util.Collections;
import java.util.List;

public enum Copy implements Trait {
  INSTANCE;

  private static final String NAME = "std::marker::Copy";
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
