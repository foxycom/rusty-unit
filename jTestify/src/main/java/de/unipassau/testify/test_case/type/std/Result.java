package de.unipassau.testify.test_case.type.std;

import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.type.Enum;
import de.unipassau.testify.test_case.type.Generic;
import de.unipassau.testify.test_case.type.Type;
import java.util.Collections;
import java.util.List;

public class Result extends Enum {

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
        false
    );
  }
}
