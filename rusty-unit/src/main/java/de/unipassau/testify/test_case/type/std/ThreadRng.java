package de.unipassau.testify.test_case.type.std;

import de.unipassau.testify.test_case.type.AbstractStruct;
import de.unipassau.testify.test_case.type.traits.rand.Rng;
import de.unipassau.testify.test_case.type.traits.rand.RngCore;
import de.unipassau.testify.test_case.type.traits.std.clone.Clone;
import de.unipassau.testify.test_case.type.traits.std.default_.Default;
import de.unipassau.testify.test_case.type.traits.std.fmt.Debug;
import de.unipassau.testify.test_case.type.traits.std.marker.Sized;
import java.util.Collections;
import java.util.Set;

public class ThreadRng extends AbstractStruct {

  public ThreadRng() {
    super(
        "rand::rngs::ThreadRng",
        Collections.emptyList(),
        false,
        Set.of(
            Clone.getInstance(),
            Debug.getInstance(),
            Default.getInstance(),
            Rng.getInstance(),
            RngCore.getInstance(),
            Sized.getInstance()
        )
    );
  }
}
