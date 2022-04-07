package de.unipassau.testify.test_case.type.traits.std.cmp;

import de.unipassau.testify.test_case.type.AssociatedType;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.traits.Trait;
import java.util.Collections;
import java.util.List;

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

