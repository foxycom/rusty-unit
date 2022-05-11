package de.unipassau.rustyunit.type.traits.std.cmp;

import de.unipassau.rustyunit.type.traits.Trait;
import java.util.Collections;

public class Ord extends Trait {
  private static final Ord instance = new Ord();

  public static Ord getInstance() {
    return instance;
  }

  private Ord() {
    super(
        "std::cmp::Ord",
        Collections.emptyList(),
        Collections.emptyList()
    );
  }
}
