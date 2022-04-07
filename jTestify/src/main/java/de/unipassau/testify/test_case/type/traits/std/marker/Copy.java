package de.unipassau.testify.test_case.type.traits.std.marker;

import de.unipassau.testify.test_case.type.AssociatedType;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.traits.Trait;
import java.util.Collections;
import java.util.List;

public class Copy extends Trait {

  private static final Copy instance = new Copy();

  public static Copy getInstance() {
    return instance;
  }

  private Copy() {
    super(
        "std::marker::Copy",
        Collections.emptyList(),
        Collections.emptyList()
    );
  }
}
