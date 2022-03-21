package de.unipassau.testify.test_case.type.rand.rngs.mock;

import de.unipassau.testify.test_case.type.AbstractStruct;
import de.unipassau.testify.test_case.type.traits.AllTraits;
import java.util.Collections;
import java.util.Set;

public class StepRng extends AbstractStruct {

  public StepRng() {
    super(
        "rand::rngs::mock::StepRng",
        Collections.emptyList(),
        false,
        Set.of(
            AllTraits.byName("rand::Rng").orElseThrow(),
            AllTraits.byName("rand::RngCore").orElseThrow()
        )
    );
  }
}
