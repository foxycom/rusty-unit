package de.unipassau.testify.test_case.callable.rand;

import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.callable.StaticMethod;
import de.unipassau.testify.test_case.type.prim.UInt.UInt64;
import de.unipassau.testify.test_case.type.rand.rngs.mock.StepRng;
import java.util.List;

public class StepRngInit extends StaticMethod {

  public StepRngInit() {
    super(
        "new",
        List.of(
          new Param(UInt64.INSTANCE, false, null),
          new Param(UInt64.INSTANCE, false, null)
      ),
        StepRng.INSTANCE,
        StepRng.INSTANCE,
        null
    );
  }
}
