package de.unipassau.rustyunit.test_case.callable.std;

import de.unipassau.rustyunit.test_case.Param;
import de.unipassau.rustyunit.test_case.callable.StaticMethod;
import de.unipassau.rustyunit.type.Type;
import de.unipassau.rustyunit.type.prim.Str;
import de.unipassau.rustyunit.type.std.string.String;
import java.util.List;

public class StringInit extends StaticMethod {

  private static final Type type = new String();

  public StringInit() {
    super("from",
        List.of(new Param(
            Str.INSTANCE,
            false,
            null
        )),
        type,
        type,
        null
        );
  }
}
