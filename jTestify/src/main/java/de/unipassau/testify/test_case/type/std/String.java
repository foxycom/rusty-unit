package de.unipassau.testify.test_case.type.std;

import de.unipassau.testify.test_case.type.AbstractStruct;
import de.unipassau.testify.test_case.type.Trait;
import java.util.Collections;
import java.util.Set;

public class String extends AbstractStruct {

  public String() {
    super(
        "std::string::String",
        Collections.emptyList(),
        false,
        Set.of(
            new Trait("std::clone::Clone"),
            new Trait("std::marker::Copy"),
            new Trait("std::cmp::Eq"),
            new Trait("std::cmp::PartialEq"),
            new Trait("std::hash::Hash"),
            new Trait("std::cmp::Ord"),
            new Trait("std::cmp::PartialOrd"),
            new Trait("std::fmt::Debug")
        )
        );
  }
}
