package de.unipassau.testify.test_case.type.traits.std.cmp;

import de.unipassau.testify.test_case.type.AssociatedType;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.traits.Trait;
import java.util.Collections;
import java.util.List;

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
