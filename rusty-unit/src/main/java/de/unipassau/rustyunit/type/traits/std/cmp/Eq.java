package de.unipassau.rustyunit.type.traits.std.cmp;

import de.unipassau.rustyunit.type.traits.Trait;
import java.util.Collections;

public class Eq extends Trait {
  private static final Eq instance = new Eq();

  public static Eq getInstance() {
    return instance;
  }

  private Eq() {
    super(
        "std::cmp::Eq",
        Collections.emptyList(),
        Collections.emptyList()
    );
  }
}

