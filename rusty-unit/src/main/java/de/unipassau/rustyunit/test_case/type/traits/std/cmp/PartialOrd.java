package de.unipassau.rustyunit.test_case.type.traits.std.cmp;

import de.unipassau.rustyunit.test_case.type.traits.Trait;
import java.util.Collections;

public class PartialOrd extends Trait {
  private static final PartialOrd instance = new PartialOrd();

  public static PartialOrd getInstance() {
    return instance;
  }

  private PartialOrd() {
    super(
        "std::cmp::PartialOrd",
        Collections.emptyList(),
        Collections.emptyList()
    );
  }
}
