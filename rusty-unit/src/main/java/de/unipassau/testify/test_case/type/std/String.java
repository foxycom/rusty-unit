package de.unipassau.testify.test_case.type.std;

import de.unipassau.testify.test_case.type.AbstractStruct;
import de.unipassau.testify.test_case.type.traits.std.clone.Clone;
import de.unipassau.testify.test_case.type.traits.std.cmp.Eq;
import de.unipassau.testify.test_case.type.traits.std.cmp.Ord;
import de.unipassau.testify.test_case.type.traits.std.cmp.PartialEq;
import de.unipassau.testify.test_case.type.traits.std.cmp.PartialOrd;
import de.unipassau.testify.test_case.type.traits.std.fmt.Debug;
import de.unipassau.testify.test_case.type.traits.std.hash.Hash;
import de.unipassau.testify.test_case.type.traits.std.marker.Copy;
import java.util.Collections;
import java.util.Set;

public class String extends AbstractStruct {

  public String() {
    super(
        "std::string::String",
        Collections.emptyList(),
        false,
        Set.of(
            Clone.getInstance(),
            Copy.getInstance(),
            Eq.getInstance(),
            PartialEq.getInstance(),
            Hash.getInstance(),
            Ord.getInstance(),
            PartialOrd.getInstance(),
            Debug.getInstance()
        )
        );
  }
}
