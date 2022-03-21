package de.unipassau.testify.test_case.type.std;

import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.type.AbstractEnum;
import de.unipassau.testify.test_case.type.Generic;
import de.unipassau.testify.test_case.type.traits.AbstractTrait;
import de.unipassau.testify.test_case.type.Type;
import java.util.Collections;
import java.util.List;
import java.util.Set;

public class Result extends AbstractEnum {

  private static final List<Type> stdGenerics = List.of(
      new Generic("T", Collections.emptyList()),
      new Generic("E", Collections.emptyList())
  );

  public Result() {
    super(
        "std::result::Result",
        stdGenerics,
        List.of(
            new TupleEnumVariant("Ok", List.of(new Param(stdGenerics.get(0), false, null))),
            new TupleEnumVariant("Err", List.of(new Param(stdGenerics.get(1), false, null)))
        ),
        false,
        Set.of(
            new AbstractTrait("std::clone::Clone"),
            new AbstractTrait("std::marker::Copy"),
            new AbstractTrait("std::cmp::Eq"),
            new AbstractTrait("std::cmp::PartialEq"),
            new AbstractTrait("std::hash::Hash"),
            new AbstractTrait("std::cmp::Ord"),
            new AbstractTrait("std::cmp::PartialOrd"),
            new AbstractTrait("std::iter::IntoIterator"),
            new AbstractTrait("std::fmt::Debug")
        )
    );
  }
}
