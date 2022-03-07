package de.unipassau.testify.util;

import de.unipassau.testify.test_case.type.Generic;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.TypeBinding;
import java.util.HashSet;
import java.util.LinkedHashSet;
import java.util.Set;
import java.util.stream.IntStream;

public class TypeUtil {

  public static Set<Generic> getDeepGenerics(Type type) {
    return getDeepGenerics(type, new LinkedHashSet<>());
  }

  private static Set<Generic> getDeepGenerics(Type type, Set<Generic> generics) {
    if (type.isGeneric()) {
      generics.add(type.asGeneric());
      return generics;
    } else {
      for (Type generic : type.generics()) {
        generics = getDeepGenerics(generic, generics);
      }
      return generics;
    }
  }

  public static TypeBinding getNecessaryBindings(Type generic, Type concrete) {
    if (!generic.canBeSameAs(concrete)) {
      throw new RuntimeException("Types cannot be same");
    }

    return getNecessaryBindingsInner(generic, concrete, new TypeBinding());
  }

  private static TypeBinding getNecessaryBindingsInner(Type generic, Type concrete,
      TypeBinding typeBinding) {
    if (generic.isGeneric()) {
      typeBinding.bindGeneric(generic.asGeneric(), concrete);
    } else {
      if (generic.isRef() && concrete.isRef()) {
        return getNecessaryBindingsInner(generic.asRef().getInnerType(), concrete.asRef().getInnerType(), typeBinding);
      } else if (!generic.generics().isEmpty() && concrete.generics().isEmpty()) {
        throw new RuntimeException();
      }
      IntStream.range(0, generic.generics().size()).forEach(
          i -> getNecessaryBindingsInner(generic.generics().get(i), concrete.generics().get(i),
              typeBinding)
      );
    }

    return typeBinding;
  }
}
