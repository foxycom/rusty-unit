package de.unipassau.testify.test_case.type.traits.std.hash;

import de.unipassau.testify.test_case.type.AssociatedType;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.traits.Trait;
import java.util.Collections;
import java.util.List;

public class Hash extends Trait {
  private static final Hash instance = new Hash();

  public static Hash getInstance() {
    return instance;
  }

  private Hash() {
    super(
        "std::hash::Hash",
        Collections.emptyList(),
        Collections.emptyList()
    );
  }
}
