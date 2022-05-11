package de.unipassau.rustyunit.test_case.type.traits.std.iter;

import de.unipassau.rustyunit.test_case.type.traits.Trait;
import java.util.Collections;

public class IntoIterator extends Trait {
  private static final IntoIterator instance = new IntoIterator();

  public static IntoIterator getInstance() {
    return instance;
  }

  private IntoIterator() {
    super(
        "std::iter::IntoIterator",
        Collections.emptyList(),
        Collections.emptyList()
    );
  }
}
