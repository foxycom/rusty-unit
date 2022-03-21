package de.unipassau.testify.test_case.type.std;

import de.unipassau.testify.test_case.type.Generic;
import de.unipassau.testify.test_case.type.AbstractStruct;
import de.unipassau.testify.test_case.type.Trait;
import java.util.Collections;
import java.util.List;
import java.util.Set;

class Vec extends AbstractStruct {

  public Vec() {
    super(
        "std::vec::Vec",
        List.of(new Generic("T", Collections.emptyList())),
        false,
        Set.of(
            new Trait("std::iter::IntoIterator"),
            new Trait("std::default::Default"),
            new Trait("std::cmp::Eq"),
            new Trait("std::cmp::PartialEq"),
            new Trait("std::cmp::PartialOrd"),
            new Trait("std::cmp::Ord")
        )
    );
  }
}