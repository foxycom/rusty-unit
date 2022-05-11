package de.unipassau.rustyunit.type.traits.std.marker;

import de.unipassau.rustyunit.type.traits.Trait;
import java.util.Collections;

public class Copy extends Trait {

  private static final Copy instance = new Copy();

  public static Copy getInstance() {
    return instance;
  }

  private Copy() {
    super(
        "std::marker::Copy",
        Collections.emptyList(),
        Collections.emptyList()
    );
  }
}
