package de.unipassau.testify.test_case.callable.std.option;


import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.callable.EnumInit;
import de.unipassau.testify.test_case.type.Enum.TupleEnumVariant;
import de.unipassau.testify.test_case.type.Generic;
import de.unipassau.testify.test_case.type.std.Option;
import java.util.Collections;
import java.util.List;

public class OptionInit {
  public static class OptionSomeInit extends EnumInit {

    public OptionSomeInit() {
      super(new Option(),
          new TupleEnumVariant("Some", List.of(
              new Param(new Generic("T", Collections.emptyList()), false, null))
          ),
          true
      );
    }
  }

  public static class OptionNoneInit extends EnumInit {
    public OptionNoneInit() {
      super(new Option(),
          new TupleEnumVariant("None", Collections.emptyList()),
          true
          );
    }
  }
}