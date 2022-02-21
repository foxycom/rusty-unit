package de.unipassau.testify.test_case.type.std;

import de.unipassau.testify.test_case.type.Enum;
import de.unipassau.testify.test_case.type.Trait;
import java.util.Collections;
import java.util.Set;

public class Infallible extends Enum {

  public Infallible() {
    super(
        "std::convert::Infallible",
        Collections.emptyList(),
        Collections.emptyList(),
        false,
        Set.of(
            new Trait("std::marker::Copy"),
            new Trait("std::clone::Clone")
        )
        );
  }
}
