package de.unipassau.testify.test_case.callable.std;

import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.callable.StaticMethod;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.prim.Str;
import de.unipassau.testify.test_case.type.std.String;
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
