package de.unipassau.testify.test_case.type.traits.std.clone;

import de.unipassau.testify.test_case.type.AssociatedType;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.traits.Trait;
import java.util.Collections;
import java.util.List;

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
