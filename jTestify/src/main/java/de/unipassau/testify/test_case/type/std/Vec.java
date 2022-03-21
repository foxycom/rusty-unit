package de.unipassau.testify.test_case.type.std;

import de.unipassau.testify.test_case.type.Generic;
import de.unipassau.testify.test_case.type.AbstractStruct;
import de.unipassau.testify.test_case.type.traits.AbstractTrait;
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
            new AbstractTrait("std::iter::IntoIterator"),
            new AbstractTrait("std::default::Default"),
            new AbstractTrait("std::cmp::Eq"),
            new AbstractTrait("std::cmp::PartialEq"),
            new AbstractTrait("std::cmp::PartialOrd"),
            new AbstractTrait("std::cmp::Ord")
        )
    );
  }
}