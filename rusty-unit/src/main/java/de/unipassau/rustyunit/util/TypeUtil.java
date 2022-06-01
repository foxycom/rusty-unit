package de.unipassau.rustyunit.util;

import de.unipassau.rustyunit.hir.TyCtxt;
import de.unipassau.rustyunit.test_case.Param;
import de.unipassau.rustyunit.test_case.callable.Callable;
import de.unipassau.rustyunit.type.Generic;
import de.unipassau.rustyunit.type.Type;
import de.unipassau.rustyunit.type.TypeBinding;
import de.unipassau.rustyunit.type.prim.Int.ISize;
import de.unipassau.rustyunit.type.prim.Prim;
import de.unipassau.rustyunit.type.traits.Trait;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.stream.Collectors;
import java.util.stream.IntStream;
import java.util.stream.Stream;
import org.javatuples.Pair;

public class TypeUtil {

  public static TyCtxt tyCtxt;

  public static TypeBinding typeBinding(Type type, Callable callable) {
    TypeBinding typeBinding = TypeUtil.getNecessaryBindings(callable.getReturnType(), type);
    callable.getParams().stream()
        .map(Param::type)
        .map(TypeUtil::getDeepGenerics)
        .peek(deepGenerics -> deepGenerics.removeAll(typeBinding.getGenerics()))
        .forEach(typeBinding::addGenerics);

    if (callable.isMethod()) {
      var methodGenerics = Stream.concat(callable.getParent().generics().stream(),
              callable.asMethod().generics().stream())
          .filter(Type::isGeneric)
          .map(Type::asGeneric)
          .collect(Collectors.toSet());

      methodGenerics.removeAll(typeBinding.getGenerics());
      typeBinding.addGenerics(methodGenerics);
    } else if (callable.isStaticMethod()) {
      var staticMethod = callable.asStaticMethod();

      var staticMethodGenerics = Stream.concat(callable.getParent().generics().stream(),
          staticMethod.generics().stream())
          .filter(Type::isGeneric)
          .map(Type::asGeneric)
          .collect(Collectors.toSet());

      staticMethodGenerics.removeAll(typeBinding.getGenerics());
      typeBinding.addGenerics(staticMethodGenerics);
    }

    if (callable.returnsValue()) {
      var generics = TypeUtil.getDeepGenerics(callable.getReturnType());
      generics.removeAll(typeBinding.getGenerics());
      typeBinding.addGenerics(generics);
    }

    typeBinding.getUnboundGenerics()
        .stream()
        .map(g -> Pair.with(g, getTypeFor(g)))
        .filter(p -> p.getValue1().isPresent())
        .forEach(p -> typeBinding.bindGeneric(p.getValue0(), p.getValue1().get()));

    return typeBinding;
  }

  public static LinkedHashSet<Generic> generics(Callable callable) {
    LinkedHashSet<Generic> generics = callable.getParams().stream()
        .map(Param::type)
        .map(TypeUtil::getDeepGenerics)
        .collect(LinkedHashSet::new, LinkedHashSet::addAll, LinkedHashSet::addAll);

    if (callable.getParent() != null) {
      generics.addAll(
          callable.getParent().generics().stream().filter(Type::isGeneric).map(Type::asGeneric)
              .collect(Collectors.toSet())
      );
    }

    if (callable.isMethod()) {
      generics.addAll(callable.asMethod().generics().stream().filter(Type::isGeneric).map(Type::asGeneric)
          .collect(Collectors.toSet()));
    } else if (callable.isStaticMethod()) {
      generics.addAll(callable.asMethod().generics().stream().filter(Type::isGeneric).map(Type::asGeneric)
          .collect(Collectors.toSet()));
    }

    if (callable.returnsValue()) {
      generics.addAll(TypeUtil.getDeepGenerics(callable.getReturnType()));
    }

    return generics;
  }

  public static TypeBinding typeBinding(Type type) {
    Set<Generic> generics = TypeUtil.getDeepGenerics(type);
    var typeBinding = new TypeBinding((LinkedHashSet<Generic>) generics);

    // Set for all generics some appropriate random type that complies with all constraints
    // and type bounds
    generics.stream().map(g -> Pair.with(g, getTypeFor(g)))
        .filter(p -> p.getValue1().isPresent())
        .forEach(p -> typeBinding.bindGeneric(p.getValue0(), p.getValue1().get()));

    return typeBinding;
  }

  /**
   * Recursively generate an actual type, e.g. if we generate std::vec::Vec&lt;T&gt;, then also
   * generate an actual type for T.
   *
   * @param generic Generic type to substitute.
   * @return Fully substituted real type.
   */
  public static Optional<Type> getTypeFor(Generic generic) {
    var primitive = getPrimitiveTypeFor(generic);
    if (primitive.isPresent()) {
      return primitive.map(p -> p);
    } else {
      return getComplexTypeFor(generic, tyCtxt);
    }
  }

  private static Optional<Prim> getPrimitiveTypeFor(Generic generic) {
    var bounds = generic.getBounds();

    if (bounds.isEmpty()) {
      return Optional.of(ISize.INSTANCE);
    }

    List<Prim> possiblePrimitives = null;
    for (Trait bound : bounds) {
      var implementors = Prim.implementorsOf(bound);
      if (possiblePrimitives == null) {
        possiblePrimitives = implementors;
      } else {
        possiblePrimitives.retainAll(implementors);
      }
    }

    if (possiblePrimitives != null && !possiblePrimitives.isEmpty()) {
      return Optional.of(Rnd.choice(possiblePrimitives));
    } else {
      return Optional.empty();
    }
  }

  /**
   * Generate a complex type for a generics recursively.
   *
   * @param generic Generic to substitute.
   * @return An actual type that deeply substitutes a generic param.
   */
  private static Optional<Type> getComplexTypeFor(Generic generic, TyCtxt tyCtxt) {
    var bounds = generic.getBounds();

    var possibleTypes = tyCtxt.typesImplementing(bounds);
    if (possibleTypes.isEmpty()) {
      return Optional.empty();
    }

    var type = Rnd.choice(possibleTypes);
    return Optional.of(type);
  }

  public static Set<Generic> getDeepGenerics(Type type) {
    return getDeepGenerics(type, new LinkedHashSet<>());
  }

  private static Set<Generic> getDeepGenerics(Type type, Set<Generic> generics) {
    if (type.isGeneric()) {
      generics.add(type.asGeneric());
    } else {
      for (Type generic : type.generics()) {
        generics = getDeepGenerics(generic, generics);
      }
    }
    return generics;
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
        return getNecessaryBindingsInner(generic.asRef().getInnerType(),
            concrete.asRef().getInnerType(), typeBinding);
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
