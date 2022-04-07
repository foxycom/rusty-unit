package de.unipassau.testify.test_case.type.traits.rand;

import de.unipassau.testify.test_case.type.AssociatedType;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.traits.Trait;
import java.util.Collections;
import java.util.List;

public class Rng extends Trait {
  private static final Rng instance = new Rng();

  public static Rng getInstance() {
    return instance;
  }

  private Rng() {
    super(
        "rand::Rng",
        Collections.emptyList(),
        Collections.emptyList()
    );
  }
}

