package de.unipassau.rustyunit.test_case.type.traits.std.marker;

import de.unipassau.rustyunit.test_case.type.traits.Trait;
import java.util.Collections;

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

