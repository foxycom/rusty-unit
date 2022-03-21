package de.unipassau.testify.test_case.type.std;

import de.unipassau.testify.test_case.type.AbstractEnum;
import de.unipassau.testify.test_case.type.traits.AbstractTrait;
import java.util.Collections;
import java.util.Set;

public class Infallible extends AbstractEnum {

  public Infallible() {
    super(
        "std::convert::Infallible",
        Collections.emptyList(),
        Collections.emptyList(),
        false,
        Set.of(
            new AbstractTrait("std::marker::Copy"),
            new AbstractTrait("std::clone::Clone")
        )
        );
  }
}
