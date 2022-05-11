package de.unipassau.rustyunit.test_case.type.traits.rand;

import de.unipassau.rustyunit.test_case.type.traits.Trait;
import java.util.Collections;

public class RngCore extends Trait {
  private static final RngCore instance = new RngCore();

  public static RngCore getInstance() {
    return instance;
  }

  private RngCore() {
    super(
        "rand::RngCore",
        Collections.emptyList(),
        Collections.emptyList()
    );
  }
}

