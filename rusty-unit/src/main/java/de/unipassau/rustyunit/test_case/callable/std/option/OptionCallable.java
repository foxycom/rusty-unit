package de.unipassau.rustyunit.test_case.callable.std.option;


import de.unipassau.rustyunit.test_case.Param;
import de.unipassau.rustyunit.test_case.callable.EnumInit;
import de.unipassau.rustyunit.test_case.callable.Method;
import de.unipassau.rustyunit.type.std.option.Option;
import java.util.Collections;
import java.util.List;

public class OptionCallable {

  public static class OptionSomeInit extends EnumInit {

    public OptionSomeInit() {
      super(new Option(),
          Option.SOME,
          true
      );
    }
  }

  public static class OptionNoneInit extends EnumInit {

    public OptionNoneInit() {
      super(new Option(),
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
              new Param(new Option(), false, null)
          ),
          Option.T,
          new Option());
    }
  }
}