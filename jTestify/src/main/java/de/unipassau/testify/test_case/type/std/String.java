package de.unipassau.testify.test_case.type.std;

import de.unipassau.testify.test_case.type.AbstractStruct;
import de.unipassau.testify.test_case.type.traits.AbstractTrait;
import java.util.Collections;
import java.util.Set;

public class String extends AbstractStruct {

  public String() {
    super(
        "std::string::String",
        Collections.emptyList(),
        false,
        Set.of(
            new AbstractTrait("std::clone::Clone"),
            new AbstractTrait("std::marker::Copy"),
            new AbstractTrait("std::cmp::Eq"),
            new AbstractTrait("std::cmp::PartialEq"),
            new AbstractTrait("std::hash::Hash"),
            new AbstractTrait("std::cmp::Ord"),
            new AbstractTrait("std::cmp::PartialOrd"),
            new AbstractTrait("std::fmt::Debug")
        )
        );
  }
}
