package de.unipassau.testify.hir;

import static java.util.stream.Collectors.toCollection;

import com.fasterxml.jackson.databind.ObjectMapper;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.callable.Callable;
import de.unipassau.testify.test_case.callable.EnumInit;
import de.unipassau.testify.test_case.callable.Method;
import de.unipassau.testify.test_case.callable.RefItem;
import de.unipassau.testify.test_case.callable.TupleInit;
import de.unipassau.testify.test_case.type.Trait;
import de.unipassau.testify.test_case.type.Type;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.Collection;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Map.Entry;
import java.util.Set;
import java.util.stream.Collectors;
import java.util.stream.Stream;
import org.javatuples.Pair;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class HirAnalysis {

  private static final Logger logger = LoggerFactory.getLogger(HirAnalysis.class);
  private static final String PROVIDERS_PATH = "/Users/tim/Documents/master-thesis/jTestify/providers";

  private final List<Callable> callables = loadCallableProviders();

  private final Map<Type, Set<Trait>> types = loadStdTypeProviders();

  public HirAnalysis(List<Callable> callables) throws IOException {
    this.callables.addAll(callables);
  }

  public Set<Type> getTypes() {
    return types.keySet();
  }

  public List<Type> typesImplementing(List<Trait> bounds) {
    return types.entrySet().stream().filter(entry -> entry.getValue().containsAll(bounds))
        .map(Entry::getKey).toList();
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
            && callable.getReturnType().canBeSameAs(type))
        // Unless we want the type explicitly, exclude completely generic callables like
        // Option::unwrap(Option) -> T, which would generate a wrapper just to unwrap it later
        .filter(callable -> (callable.getReturnType().getName().equals(type.getName()))
            || !callable.getReturnType().isGeneric());

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

  private static List<Callable> loadCallableProviders() throws IOException {
    var callablesPath = Paths.get(PROVIDERS_PATH, "callables");
    var mapper = new ObjectMapper();
    var javaType = mapper.getTypeFactory().constructCollectionType(List.class, Callable.class);

    List<Callable> callables;
    try (Stream<Path> walk = Files.walk(callablesPath, 1)) {
      callables = walk.filter(Files::isRegularFile).map(path -> {
            try {
              var content = Files.readString(path);
              return mapper.<List<Callable>>readValue(content, javaType);
            } catch (IOException e) {
              e.printStackTrace();
            }

            return null;
          }).flatMap(Collection::stream)
          .collect(toCollection(ArrayList::new));
    }

    var types = loadStdTypeProviders();
    var enumInits = types.keySet()
        .stream()
        .filter(Type::isEnum)
        .map(traits -> {
          var enumType = traits.asEnum();
          return enumType.getVariants().stream()
              .map(variant -> new EnumInit(enumType, variant, true))
              .toList();
        })
        .flatMap(Collection::stream)
        .toList();

    callables.addAll(enumInits);
    callables.addAll(loadArtificialCallables());
    callables.addAll(loadFunctions());

    return callables;
  }

  private static List<Callable> loadFunctions() throws IOException {
    var mapper = new ObjectMapper();
    var javaType = mapper.getTypeFactory().constructCollectionType(List.class, Callable.class);

    var functionsPath = Paths.get(PROVIDERS_PATH, "functions.json");
    var content = Files.readString(functionsPath);

    return mapper.readValue(content, javaType);
  }

  private static List<Callable> loadArtificialCallables() {

    return List.of(RefItem.MUTABLE, RefItem.IMMUTABLE, TupleInit.DEFAULT, TupleInit.SINGLE, TupleInit.PAIR,
        TupleInit.TRIPLETT);
  }

  private static Map<Type, Set<Trait>> loadStdTypeProviders() throws IOException {
    var typesPath = Paths.get(PROVIDERS_PATH, "types");

    var mapper = new ObjectMapper();
    var setType = mapper.getTypeFactory().constructCollectionType(Set.class, Trait.class);
    Map<Type, Set<Trait>> typeProviders = new HashMap<>();

    try (Stream<Path> walk = Files.walk(typesPath, 1)) {
      walk.filter(Files::isRegularFile).forEach(path -> {
        try {
          var typeContent = Files.readString(path);
          var type = mapper.readValue(typeContent, Type.class);
          var implementationPath = Paths.get(PROVIDERS_PATH, "implementations",
              type.getName() + ".json");

          var implementationsContent = Files.readString(implementationPath);
          Set<Trait> traits = mapper.readValue(implementationsContent, setType);
          typeProviders.put(type, traits);
        } catch (IOException e) {
          e.printStackTrace();
        }

      });
    }

    return typeProviders;
  }

}
