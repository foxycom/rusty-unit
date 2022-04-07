package de.unipassau.testify.test_case.type.std;

import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.type.AbstractEnum;
import de.unipassau.testify.test_case.type.Generic;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.traits.std.clone.Clone;
import de.unipassau.testify.test_case.type.traits.std.cmp.Eq;
import de.unipassau.testify.test_case.type.traits.std.cmp.Ord;
import de.unipassau.testify.test_case.type.traits.std.cmp.PartialEq;
import de.unipassau.testify.test_case.type.traits.std.cmp.PartialOrd;
import de.unipassau.testify.test_case.type.traits.std.fmt.Debug;
import de.unipassau.testify.test_case.type.traits.std.hash.Hash;
import de.unipassau.testify.test_case.type.traits.std.iter.IntoIterator;
import de.unipassau.testify.test_case.type.traits.std.marker.Copy;
import java.util.Collections;
import java.util.List;
import java.util.Set;

public class Result extends AbstractEnum {
  public static final Generic T = new Generic("T", Collections.emptyList());
  public static final Generic E = new Generic("E", Collections.emptyList());
  public static final EnumVariant OK = new TupleEnumVariant("Ok", List.of(new Param(T, false, null)));
  public static final EnumVariant ERR = new TupleEnumVariant("Err", List.of(new Param(E, false, null)));
  private static final List<Type> stdGenerics = List.of(T, E);

  public Result() {
    super(
        "std::result::Result",
        stdGenerics,
        List.of(OK, ERR),
        false,
        Set.of(
            Clone.getInstance(),
            Copy.getInstance(),
            Eq.getInstance(),
            PartialEq.getInstance(),
            Hash.getInstance(),
            Ord.getInstance(),
            IntoIterator.getInstance(),
            Debug.getInstance()
        )
    );
  }
}
