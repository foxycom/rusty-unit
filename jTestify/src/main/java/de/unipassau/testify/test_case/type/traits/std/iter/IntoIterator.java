package de.unipassau.testify.test_case.type.traits.std.iter;

import de.unipassau.testify.test_case.type.AssociatedType;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.traits.Trait;
import java.util.Collections;
import java.util.List;

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
