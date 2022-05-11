package de.unipassau.rustyunit.type.std.hash;

import de.unipassau.rustyunit.type.Struct;
import de.unipassau.rustyunit.type.Type;
import de.unipassau.rustyunit.type.TypeBinding;
import de.unipassau.rustyunit.type.traits.Trait;
import de.unipassau.rustyunit.type.traits.std.hash.Hash;
import java.util.Collections;
import java.util.List;
import java.util.Set;

public enum Hasher implements Struct {
  INSTANCE;

  private static final Set<Trait> implementedTraits = Set.of(
      Hash.getInstance()
  );

  @Override
  public boolean isLocal() {
    return false;
  }

  @Override
  public String getName() {
    return "std::hash::Hasher";
  }

  @Override
  public void setName(String name) {
    throw new RuntimeException("setName is not implemented");
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
}
