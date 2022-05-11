package de.unipassau.testify.test_case.type.traits.rand;

import de.unipassau.testify.test_case.type.AssociatedType;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.traits.Trait;
import java.util.Collections;
import java.util.List;

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

