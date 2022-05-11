package de.unipassau.testify.test_case.callable.std.option;


import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.callable.EnumInit;
import de.unipassau.testify.test_case.callable.Method;
import de.unipassau.testify.test_case.type.AbstractEnum.TupleEnumVariant;
import de.unipassau.testify.test_case.type.Generic;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.std.Option;
import java.util.Collections;
import java.util.List;

public class OptionCallable {

  public static class OptionSomeInit extends EnumInit {

    public OptionSomeInit() {
      super(Option.getInstance(),
          Option.SOME,
          true
      );
    }
  }

  public static class OptionNoneInit extends EnumInit {

    public OptionNoneInit() {
      super(Option.getInstance(),
          Option.NONE,
          true
      );
    }
  }

  public static class OptionUnwrap extends Method {

    public OptionUnwrap() {
      super("unwrap",
          Collections.emptyList(),
          List.of(
              new Param(Option.getInstance(), false, null)
          ),
          Option.T,
          Option.getInstance());
    }
  }
}