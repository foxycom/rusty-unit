package de.unipassau.rustyunit.test_case.type.std;

import de.unipassau.rustyunit.test_case.type.Generic;
import de.unipassau.rustyunit.test_case.type.AbstractStruct;
import de.unipassau.rustyunit.test_case.type.traits.std.cmp.Eq;
import de.unipassau.rustyunit.test_case.type.traits.std.cmp.Ord;
import de.unipassau.rustyunit.test_case.type.traits.std.cmp.PartialEq;
import de.unipassau.rustyunit.test_case.type.traits.std.cmp.PartialOrd;
import de.unipassau.rustyunit.test_case.type.traits.std.default_.Default;
import de.unipassau.rustyunit.test_case.type.traits.std.iter.IntoIterator;
import java.util.Collections;
import java.util.List;
import java.util.Set;

class Vec extends AbstractStruct {
  public static final Generic T = new Generic("T", Collections.emptyList());
  public Vec() {
    super(
        "std::vec::Vec",
        List.of(T),
        false,
        Set.of(
            IntoIterator.getInstance(),
            Default.getInstance(),
            Eq.getInstance(),
            PartialEq.getInstance(),
            PartialOrd.getInstance(),
            Ord.getInstance()
        )
    );
  }
}