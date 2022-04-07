package de.unipassau.testify.test_case.type.traits.std.cmp;

import de.unipassau.testify.test_case.type.AssociatedType;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.traits.Trait;
import java.util.Collections;
import java.util.List;

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

