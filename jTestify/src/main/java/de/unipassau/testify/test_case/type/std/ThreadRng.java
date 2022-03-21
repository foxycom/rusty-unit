package de.unipassau.testify.test_case.type.std;

import de.unipassau.testify.test_case.type.AbstractStruct;
import de.unipassau.testify.test_case.type.traits.AbstractTrait;
import java.util.Collections;
import java.util.Set;

public class ThreadRng extends AbstractStruct {

  public ThreadRng() {
    super(
        "rand::rngs::ThreadRng",
        Collections.emptyList(),
        false,
        Set.of(
            new AbstractTrait("std::clone::Clone"),
            new AbstractTrait("std::fmt::Debug"),
            new AbstractTrait("core::default::Default"),
            new AbstractTrait("rand::CryptoRng"),
            new AbstractTrait("rand::Rng"),
            new AbstractTrait("rand::RngCore"),
            new AbstractTrait("std::marker::Sized")
        )
    );
  }
}
