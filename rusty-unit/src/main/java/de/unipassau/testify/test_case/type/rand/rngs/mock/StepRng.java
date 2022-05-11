package de.unipassau.testify.test_case.type.rand.rngs.mock;

import de.unipassau.testify.test_case.type.Struct;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.TypeBinding;
import de.unipassau.testify.test_case.type.traits.Trait;
import de.unipassau.testify.test_case.type.traits.rand.Rng;
import de.unipassau.testify.test_case.type.traits.rand.RngCore;
import java.util.Collections;
import java.util.List;
import java.util.Set;

public enum StepRng implements Struct {

  INSTANCE;

  private static final Set<Trait> implementedTraits = Set.of(
      Rng.getInstance(),
      RngCore.getInstance()
  );

  @Override
  public String getName() {
    return "rand::rngs::mock::StepRng";
  }

  @Override
  public void setName(String name) {
    throw new RuntimeException("setName is not implemented");
  }

  @Override
  public boolean isLocal() {
    return false;
  }

  @Override
  public List<Type> generics() {
    return Collections.emptyList();
  }

  @Override
  public Set<Trait> implementedTraits() {
    return implementedTraits;
  }

  @Override
  public Type bindGenerics(TypeBinding binding) {
    return INSTANCE;
  }

  @Override
  public Type copy() {
    return INSTANCE;
  }

  @Override
  public String toString() {
    return encode();
  }
}
