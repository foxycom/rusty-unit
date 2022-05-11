package de.unipassau.testify.test_case.type.traits.std.marker;

import de.unipassau.testify.test_case.type.AssociatedType;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.traits.Trait;
import java.util.Collections;
import java.util.List;

public class Sized extends Trait {
  private static final Sized instance = new Sized();

  public static Sized getInstance() {
    return instance;
  }

  private Sized() {
    super(
        "std::marker::Sized",
        Collections.emptyList(),
        Collections.emptyList()
    );
  }
}

