package de.unipassau.rustyunit.type.traits.std.cmp;

import de.unipassau.rustyunit.type.traits.Trait;
import java.util.Collections;

public class PartialEq extends Trait {
  private static final PartialEq instance = new PartialEq();

  public static PartialEq getInstance() {
    return instance;
  }

  private PartialEq() {
    super(
        "std::cmp::PartialEq",
        Collections.emptyList(),
        Collections.emptyList()
    );
  }
}

