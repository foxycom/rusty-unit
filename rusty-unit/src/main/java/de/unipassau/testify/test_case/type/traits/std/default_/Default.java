package de.unipassau.testify.test_case.type.traits.std.default_;

import de.unipassau.testify.test_case.type.AssociatedType;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.traits.Trait;
import java.util.Collections;
import java.util.List;

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
