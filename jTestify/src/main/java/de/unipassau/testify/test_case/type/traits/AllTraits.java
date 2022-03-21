package de.unipassau.testify.test_case.type.traits;

import de.unipassau.testify.test_case.type.Trait;
import de.unipassau.testify.test_case.type.traits.rand.Rng;
import de.unipassau.testify.test_case.type.traits.rand.RngCore;
import java.util.List;
import java.util.Optional;

public class AllTraits {
  public static final List<Trait> PREDEFINED_TRAITS;

  static {
    PREDEFINED_TRAITS = List.of(
        new Rng(),
        new RngCore()
    );
  }

  public static Optional<Trait> byName(String name) {
    return PREDEFINED_TRAITS.stream().filter(trait -> trait.getName().equals(name)).findFirst();
  }
}
