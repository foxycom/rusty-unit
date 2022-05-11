package de.unipassau.rustyunit.test_case.type.traits.std.default_;

import de.unipassau.rustyunit.test_case.type.traits.Trait;
import java.util.Collections;

public class Default extends Trait {
  private static final Default instance = new Default();

  public static Default getInstance() {
    return instance;
  }

  private Default() {
    super(
        "std::default::Default",
        Collections.emptyList(),
        Collections.emptyList()
    );
  }
}
