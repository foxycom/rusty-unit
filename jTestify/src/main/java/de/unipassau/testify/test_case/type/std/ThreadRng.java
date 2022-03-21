package de.unipassau.testify.test_case.type.std;

import de.unipassau.testify.test_case.type.AbstractStruct;
import de.unipassau.testify.test_case.type.Trait;
import java.util.Collections;
import java.util.Set;

public class ThreadRng extends AbstractStruct {

  public ThreadRng() {
    super(
        "rand::rngs::ThreadRng",
        Collections.emptyList(),
        false,
        Set.of(
            new Trait("std::clone::Clone"),
            new Trait("std::fmt::Debug"),
            new Trait("core::default::Default"),
            new Trait("rand::CryptoRng"),
            new Trait("rand::Rng"),
            new Trait("rand::RngCore"),
            new Trait("std::marker::Sized")
        )
    );
  }
}
