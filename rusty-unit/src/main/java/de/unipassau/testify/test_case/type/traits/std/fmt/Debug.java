package de.unipassau.testify.test_case.type.traits.std.fmt;

import de.unipassau.testify.test_case.type.traits.Trait;
import java.util.Collections;

public class Debug extends Trait {
  private static final Debug instance = new Debug();

  public static Debug getInstance() {
    return instance;
  }

  private Debug() {
    super(
        "std::fmt::Debug",
        Collections.emptyList(),
        Collections.emptyList()
    );
  }
}
