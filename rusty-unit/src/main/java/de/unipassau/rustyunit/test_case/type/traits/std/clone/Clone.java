package de.unipassau.rustyunit.test_case.type.traits.std.clone;

import de.unipassau.rustyunit.test_case.type.traits.Trait;
import java.util.Collections;

public class Clone extends Trait {
  private static final Clone instance = new Clone();

  public static Clone getInstance() {
    return instance;
  }

  private Clone() {
    super(
        "std::clone::Clone",
        Collections.emptyList(),
        Collections.emptyList()
    );
  }
}
