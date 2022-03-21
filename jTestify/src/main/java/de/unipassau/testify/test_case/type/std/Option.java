package de.unipassau.testify.test_case.type.std;

import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.type.AbstractEnum;
import de.unipassau.testify.test_case.type.Enum;
import de.unipassau.testify.test_case.type.Generic;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.traits.std.clone.Clone;
import de.unipassau.testify.test_case.type.traits.std.cmp.Eq;
import de.unipassau.testify.test_case.type.traits.std.cmp.Ord;
import de.unipassau.testify.test_case.type.traits.std.cmp.PartialEq;
import de.unipassau.testify.test_case.type.traits.std.cmp.PartialOrd;
import de.unipassau.testify.test_case.type.traits.std.default_.Default;
import de.unipassau.testify.test_case.type.traits.std.hash.Hash;
import de.unipassau.testify.test_case.type.traits.std.iter.IntoIterator;
import de.unipassau.testify.test_case.type.traits.std.marker.Copy;
import java.util.Collections;
import java.util.List;
import java.util.Set;

public class Option extends AbstractEnum {
  public static final Type T = new Generic("T", Collections.emptyList());
  public static final List<Type> GENERICS = List.of(T);
  public static final EnumVariant NONE = new TupleEnumVariant("None", Collections.emptyList());
  public static final EnumVariant SOME = new TupleEnumVariant("Some", List.of(new Param(T, false, null)));

  private static Option instance;

  public static Option getInstance() {
    if (instance == null) {
      instance = new Option();
    }

    return instance;
  }

  Option() {
    super(
        "std::option::Option",
        GENERICS,
        List.of(
            NONE,
            SOME
        ),
        false,
        Set.of(
            Clone.INSTANCE,
            Copy.INSTANCE,
            Default.INSTANCE,
            Eq.INSTANCE,
            PartialEq.INSTANCE,
            Hash.INSTANCE,
            Ord.INSTANCE,
            PartialOrd.INSTANCE,
            IntoIterator.INSTANCE
        )
    );
  }
}
