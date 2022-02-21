package de.unipassau.testify.test_case.type.std;

import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.type.Enum;
import de.unipassau.testify.test_case.type.Generic;
import de.unipassau.testify.test_case.type.Trait;
import de.unipassau.testify.test_case.type.Type;
import java.util.Collections;
import java.util.List;
import java.util.Set;

public class Option extends Enum {

  private static final List<Type> stdGenerics = List.of(new Generic("T", Collections.emptyList()));

  public Option() {
    super(
        "std::option::Option",
        stdGenerics,
        List.of(
            new TupleEnumVariant("Some", List.of(new Param(stdGenerics.get(0), false, null))),
            new TupleEnumVariant("None", Collections.emptyList())
        ),
        false,
        Set.of(
            new Trait("std::clone::Clone"),
            new Trait("std::marker::Copy"),
            new Trait("std::default::Default"),
            new Trait("std::cmp::Eq"),
            new Trait("std::cmp::PartialEq"),
            new Trait("std::hash::Hash"),
            new Trait("std::cmp::Ord"),
            new Trait("std::cmp::PartialOrd"),
            new Trait("std::iter::IntoIterator")
        )
    );
  }
}
