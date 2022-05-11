package de.unipassau.testify.test_case.type.traits.std.cmp;

import de.unipassau.testify.test_case.type.AssociatedType;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.traits.Trait;
import java.util.Collections;
import java.util.List;

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
