package de.unipassau.rustyunit.type.traits.std.hash;

import de.unipassau.rustyunit.type.traits.Trait;
import java.util.Collections;

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
