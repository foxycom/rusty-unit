package de.unipassau.testify.hir;

import static java.util.stream.Collectors.toCollection;

import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.callable.Callable;
import de.unipassau.testify.test_case.callable.Method;
import de.unipassau.testify.test_case.callable.base.OptionInit;
import de.unipassau.testify.test_case.callable.base.OptionInit.OptionNoneInit;
import de.unipassau.testify.test_case.callable.base.OptionInit.OptionSomeInit;
import de.unipassau.testify.test_case.callable.base.StringInit;
import de.unipassau.testify.test_case.type.Trait;
import de.unipassau.testify.test_case.type.Type;
import java.io.IOException;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.stream.Collectors;
import org.javatuples.Pair;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class TyCtxt {

  private static final Logger logger = LoggerFactory.getLogger(TyCtxt.class);

  private final List<Callable> callables = loadBaseCallables();

  private final Set<Type> types = new HashSet<>();

  public TyCtxt(List<Callable> callables) throws IOException {
    this.callables.addAll(callables);
    analysis();
  }

  private static List<Callable> loadBaseCallables() {
    var baseCallables = new ArrayList<Callable>();
    baseCallables.add(new OptionNoneInit());
    baseCallables.add(new OptionSomeInit());
    baseCallables.add(new StringInit());

    return baseCallables;
  }

  private void analysis() {
    for (Callable callable : callables) {
      if (callable.getParent() != null) {
        var parent = callable.getParent();
        addType(parent);
      }

      for (Param param : callable.getParams()) {
        addType(param.getType());
      }

      if (callable.getReturnType() != null) {
        addType(callable.getReturnType());
      }
    }
  }

  private void addType(Type type) {
    if (type.isGeneric() || type.isPrim()) {
      // Skip for now
    } else if (type.isRef()) {
      addType(type.asRef().getInnerType());
    } else if (type.isTuple()) {
      type.asTuple().getTypes().forEach(this::addType);
    } else if (type.isStruct()) {
      types.add(type);
    } else if (type.isEnum()) {
      types.add(type);
    } else if (type.isArray()) {
      addType(type.asArray().type());
    } else {
      throw new RuntimeException("Not implemented");
    }
  }

  public Set<Type> getTypes() {
    return types;
  }

  public List<Type> typesImplementing(List<Trait> bounds) {
    throw new RuntimeException("Not implemented");
  }

  public List<Callable> getCallablesOf(Type type) {
    throw new RuntimeException("Not implemented");
  }

  public List<Callable> getCallables() {
    return callables;
  }

  public List<Callable> getCallables(String filePath) {
    return callables.stream().filter(
            callable -> callable.getSrcFilePath() != null
                && callable.getSrcFilePath().equals(filePath))
        .toList();
  }

  public List<Pair<VarReference, Method>> methodsOf(List<VarReference> variables) {
    return callables.stream()
        .filter(Callable::isMethod)
        .map(callable -> (Method) callable)
        .map(method -> variables.stream()
            .filter(v -> method.getParent().canBeSameAs(v.type()))
            .filter(v -> {
              var selfParam = method.getSelfParam();
              var testCase = v.testCase();
              if (selfParam.isByReference()) {
                return v.isBorrowableAt(testCase.size());
              } else {
                return v.isConsumableAt(testCase.size());
              }
            })
            .map(v -> Pair.with(v, method))
            .toList())
        .flatMap(List::stream)
        .collect(Collectors.toList());
  }

  public List<Callable> generatorsOf(Type type, String filePath) {
    return generatorsOf(type, filePath, Callable.class);
  }

  /**
   * Returns the generators which can either generate a type that 1) is the same, e.g., u32 == u32
   * 2) is generic and can be the given type wrt the trait bounds, e.g., T: Default == u32 3) is a
   * container and some inner type can be same as given type, e.g., Vec<u32> == u32
   *
   * @param type The type to look for.
   * @return The generators of the type.
   */
  public <S extends Callable> List<Callable> generatorsOf(Type type, String filePath,
      Class<S> subClass) {
    logger.debug("Looking for generators of " + type);
    var stream = callables.stream()
        .filter(subClass::isInstance)
        .filter(callable -> callable.returnsValue()
            && callable.getReturnType().canBeSameAs(type));
        // Unless we want the type explicitly, exclude completely generic callables like
        // Option::unwrap(Option) -> T, which would generate a wrapper just to unwrap it later
//        .filter(callable -> (callable.getReturnType().getName().equals(type.getName()))
//            || !callable.getReturnType().isGeneric());

    if (filePath != null) {
      logger.debug("File path is not null, applying filtering");
      stream = stream.filter(callable -> callable.isPublic()
          || (callable.getSrcFilePath() != null && callable.getSrcFilePath().equals(filePath)));
    }

    var generators = stream.collect(toCollection(ArrayList::new));

    return generators;
  }

  public <S extends Callable> List<Callable> wrappingGeneratorsOf(Type type, String filePath) {
    return wrappingGeneratorsOf(type, filePath, Callable.class);
  }

  private <S extends Callable> List<Callable> wrappingGeneratorsOf(Type type, String filePath,
      Class<S> subClass) {
    logger.debug("Looking for wrapping generators of " + type);
    var stream = callables.stream()
        .filter(subClass::isInstance)
        .filter(callable -> callable.returnsValue()
            && callable.getReturnType().wraps(type));
    if (filePath != null) {
      logger.debug("File path is not null, applying filtering");
      stream = stream.filter(callable -> callable.isPublic()
          || (callable.getSrcFilePath() != null && callable.getSrcFilePath().equals(filePath)));
    }

    var generators = stream.collect(toCollection(ArrayList::new));
    return generators;
  }

  public <S extends Callable> List<Callable> generatorsOf(Type owner, Type type, String filePath,
      Class<S> subClass) {
    logger.debug("Looking for generators of " + type + " by " + owner);
    var stream = callables.stream()
        .filter(subClass::isInstance)
        .filter(callable -> callable.getParent() != null && callable.getParent().equals(owner))
        .filter(callable -> callable.returnsValue() && callable.getReturnType()
            .canBeIndirectlySameAs(type));
    if (filePath != null) {
      logger.debug("File path is not null, applying filtering");
      stream = stream.filter(callable -> callable.isPublic()
          || (callable.getSrcFilePath() != null && callable.getSrcFilePath().equals(filePath)));
    }

    return stream.collect(toCollection(ArrayList::new));
  }
}
