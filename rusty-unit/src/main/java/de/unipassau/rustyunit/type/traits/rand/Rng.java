package de.unipassau.rustyunit.type.traits.rand;

import de.unipassau.rustyunit.type.traits.Trait;
import java.util.Collections;

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

