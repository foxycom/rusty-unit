package de.unipassau.testify.test_case;

import de.unipassau.testify.ddg.Dependency;
import de.unipassau.testify.ddg.Node;
import de.unipassau.testify.exception.NoAvailableArgException;
import de.unipassau.testify.generators.TestIdGenerator;
import de.unipassau.testify.hir.HirAnalysis;
import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.metaheuristics.operators.Crossover;
import de.unipassau.testify.metaheuristics.operators.Mutation;
import de.unipassau.testify.test_case.callable.Callable;
import de.unipassau.testify.test_case.statement.PrimitiveStmt;
import de.unipassau.testify.test_case.statement.Statement;
import de.unipassau.testify.test_case.type.Generic;
import de.unipassau.testify.test_case.type.Trait;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.TypeBinding;
import de.unipassau.testify.test_case.type.prim.Int.ISize;
import de.unipassau.testify.test_case.type.prim.Prim;
import de.unipassau.testify.util.Rnd;
import de.unipassau.testify.util.TypeUtil;
import java.util.ArrayList;
import java.util.Collections;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.Set;
import java.util.stream.Collectors;
import java.util.stream.IntStream;
import org.javatuples.Pair;
import org.javatuples.Quartet;
import org.jgrapht.Graph;
import org.jgrapht.graph.DirectedMultigraph;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class TestCase extends AbstractTestCaseChromosome<TestCase> {

  private static Logger logger = LoggerFactory.getLogger(TestCase.class);

  private int id;
  private List<Statement> statements;
  private Map<Integer, Double> coverage;
  private Graph<Node, Dependency> ddg;
  private HirAnalysis hirAnalysis;
  private Map<VarReference, TypeBinding> typeBindings;

  public TestCase(int id, HirAnalysis hirAnalysis, Mutation<TestCase> mutation,
      Crossover<TestCase> crossover) {
    super(mutation, crossover);
    this.id = id;
    this.hirAnalysis = hirAnalysis;
    this.statements = new ArrayList<>();
    this.ddg = new DirectedMultigraph<>(Dependency.class);
    this.typeBindings = new HashMap<>();
    this.coverage = new HashMap<>();
  }

  public TestCase(TestCase other) {
    super(other.getMutation(), other.getCrossover());
    this.id = TestIdGenerator.get();
    this.hirAnalysis = other.hirAnalysis;
    this.statements = other.statements.stream().map(s -> s.copy(this))
        .collect(Collectors.toCollection(ArrayList::new));
    // TODO not really copying it
    this.ddg = new DirectedMultigraph<>(Dependency.class);

    this.typeBindings = variables().stream()
        .map(v -> Pair.with(v, other.typeBindings.get(v)))
        .collect(
            HashMap::new,
            (m, v) -> m.put(v.getValue0(), v.getValue1()),
            HashMap::putAll
        );
    this.coverage = new HashMap<>();
  }

  public int getId() {
    return id;
  }

  public void setId(int id) {
    this.id = id;
  }

  @Override
  public int size() {
    return statements.size();
  }

  public TypeBinding getTypeBindingsFor(VarReference var) {
    return typeBindings.get(var);
  }

  public TypeBinding popTypeBindingsFor(VarReference var) {
    return typeBindings.remove(var);
  }

  public void setTypeBindingsFor(VarReference var, TypeBinding binding) {
    typeBindings.put(var, binding);
  }

  public List<Statement> getStatements() {
    return statements;
  }


  public void setStatements(List<Statement> statements) {
    this.statements = statements;
  }

  public void setCoverage(Map<Integer, Double> coverage) {
    this.coverage = coverage;
  }

  public Graph<Node, Dependency> getDdg() {
    return ddg;
  }

  public boolean isCutable() {
    return statements.size() > 1;
  }

  public VarReference satisfyParameter(int pos, Type parameter) throws NoAvailableArgException {
    List<VarReference> usableVariables;
    if (parameter.isRef()) {
      usableVariables = borrowableVariablesOfType(parameter, pos);
    } else {
      usableVariables = consumableVariablesOfType(parameter, pos);
    }

    VarReference var;
    if (!usableVariables.isEmpty()) {
      // TODO Reuse a variable
      var = Rnd.choice(usableVariables);
    } else {
      var generatedArg = generateArg(parameter);
      if (generatedArg.isPresent()) {
        var = generatedArg.get();
      } else {
        throw new NoAvailableArgException("Could not generate any argument for " + parameter);
      }
    }
    return var;
  }

  /**
   * We assume that all the variables used in the statement do not exist in this test case, because
   * the stmt comes from another one.
   */
  public List<VarReference> satisfyParameters(int pos, Statement stmt)
      throws NoAvailableArgException {
    var paramTypes = stmt.actualParamTypes();

    List<VarReference> variables = new ArrayList<>(paramTypes.size());
    for (Type paramType : paramTypes) {
      var var = satisfyParameter(pos, paramType);
      variables.add(var);
    }

    return variables;
  }

  public void insertStmt(int pos, Statement stmt) {
    statements.add(pos, stmt);
  }

  public void addStmt(Statement stmt) {
    int insertPosition = 0;
    if (stmt.isPrimitiveStmt()) {
      statements.add(stmt);
    } else if (stmt.isCallableStmt()) {
      var callableStmt = stmt.asCallableStmt();
      insertPosition = callableStmt.args().stream().map(VarReference::position)
          .reduce(0, Integer::max);
      var size = size();
      insertStmt(Integer.min(size, insertPosition + 1), stmt);
    } else if (stmt.isStructInitStmt()) {
      var structInitStmt = stmt.asStructInitStmt();
      insertPosition = structInitStmt.args().stream().map(VarReference::position)
          .reduce(0, Integer::max);
      var size = size();
      insertStmt(Integer.min(size, insertPosition + 1), stmt);
    } else if (stmt.isEnumStmt()) {
      var enumStmt = stmt.asEnumStmt();
      insertPosition = enumStmt.getArgs().stream().map(VarReference::position)
          .reduce(0, Integer::max);
      insertStmt(Integer.min(size(), insertPosition + 1), stmt);
    } else if (stmt.isRefStmt()) {
      var refStmt = stmt.asRefStmt();
      insertPosition = refStmt.arg().position();
      insertStmt(Integer.min(size(), insertPosition + 1), refStmt);
    } else {
      throw new RuntimeException("Not implemented");
    }
  }

  public void appendStmt(Statement stmt) {
    statements.add(stmt);
  }

  public void removeAllStmts() {
    statements.clear();
    typeBindings.clear();
  }

  public int removeStmt(Statement stmt) {
    if (!stmt.returnsValue()) {
      statements.remove(stmt);
      return 0;
    }

    var returnValue = stmt.returnValue().orElseThrow();
    var usingStmts = returnValue.usedAt().stream()
        .map(this::stmtAt)
        .map(Optional::orElseThrow)
        .toList();
    Collections.reverse(usingStmts);
    usingStmts.forEach(this::removeStmt);
    return 0;
  }

  public Optional<Statement> stmtAt(int pos) {
    if (pos >= size() || pos < 0) {
      return Optional.empty();
    }

    return Optional.of(statements.get(pos));
  }

  public Optional<Integer> stmtPosition(Statement stmt) {
    var pos = statements.indexOf(stmt);
    if (pos < 0) {
      logger.warn("Could not find position of a statement in test");
    }
    return Optional.of(pos);
  }

  public Optional<Integer> varPosition(VarReference var) {
    throw new RuntimeException("Not implemented");
  }

  public String getName() {
    return String.format("testify_%d", id);
  }

  public Set<Type> instantiatedTypes() {
    return statements.stream()
        .map(Statement::returnValue)
        .filter(Optional::isPresent)
        .map(Optional::get)
        .map(VarReference::type)
        .collect(Collectors.toSet());
  }

  public List<VarReference> variables() {
    return statements.stream()
        .map(Statement::returnValue)
        .filter(Optional::isPresent)
        .map(Optional::get)
        .collect(Collectors.toCollection(ArrayList::new));
  }

  public List<VarReference> borrowableVariablesOfType(Type type, int untilPos) {
    return IntStream.range(0, untilPos)
        .mapToObj(statements::get)
        .map(Statement::returnValue)
        .filter(Optional::isPresent)
        .map(Optional::get)
        .filter(var -> var.type().equals(type) && !var.isBorrowableAt(untilPos))
        .collect(Collectors.toCollection(ArrayList::new));
  }

  public List<VarReference> consumableVariablesOfType(Type type, int untilPos) {
    return IntStream.range(0, untilPos)
        .mapToObj(statements::get)
        .map(Statement::returnValue)
        .filter(Optional::isPresent)
        .map(Optional::get)
        .filter(var -> var.type().equals(type) && !var.isConsumableAt(untilPos))
        .collect(Collectors.toCollection(ArrayList::new));
  }

  public List<VarReference> unconsumedVariablesOfType(Type type) {
    return statements.stream()
        .map(Statement::returnValue)
        .filter(Optional::isPresent)
        .map(Optional::get)
        .filter(var -> var.type().equals(type) && !var.isConsumed())
        .collect(Collectors.toCollection(ArrayList::new));
  }

  /**
   * Get all defined variables of a type.
   */
  public List<VarReference> variablesOfType(Type type) {
    return statements.stream()
        .map(Statement::returnValue)
        .filter(Optional::isPresent)
        .map(Optional::get)
        .filter(var -> var.type().equals(type))
        .collect(Collectors.toCollection(ArrayList::new));
  }

  /**
   * Get defined variables of a type until the given position (exclusive).
   */
  public List<VarReference> variablesOfType(Type type, int pos) {
    if (pos == 0) {
      return new ArrayList<>();
    }

    return IntStream.range(0, pos)
        .mapToObj(i -> statements.get(i).returnValue())
        .filter(Optional::isPresent)
        .map(Optional::get)
        .filter(var -> var.type().equals(type))
        .collect(Collectors.toCollection(ArrayList::new));
  }

  public List<Quartet<VarReference, Callable, Integer, Integer>> availableCallables() {
    throw new RuntimeException("Not implemented");
  }

  public boolean insertRandomStmt() {
    var callable = Rnd.choice(hirAnalysis.getCallables());
    logger.info("Selected callable: {}", callable);

    return insertCallable(callable);
  }

  public boolean insertCallable(Callable callable) {
    logger.info("Inserting callable {}", callable);

    Set<Generic> generics = callable.getParams().stream()
        .map(Param::getType)
        .map(TypeUtil::getDeepGenerics)
        .collect(HashSet::new, HashSet::addAll, HashSet::addAll);
    if (callable.isMethod()) {
      generics.addAll(callable.getParent().generics().stream().map(Type::asGeneric)
          .collect(Collectors.toSet()));
    }

    if (callable.returnsValue()) {
      generics.addAll(TypeUtil.getDeepGenerics(callable.getReturnType()));
    }

    logger.info("It's generics are: {}", generics);

    var typeBinding = new TypeBinding(generics);

    generics.stream().map(g -> Pair.with(g, getTypeFor(g))).filter(p -> p.getValue1().isPresent())
        .forEach(p -> typeBinding.bindGeneric(p.getValue0(), p.getValue1().get()));
    if (typeBinding.hasUnboundedGeneric()) {
      logger.info("Could not bind all generics: {}", typeBinding.getUnboundGenerics());
      return false;
    }

    var args = callable.getParams().stream()
        .map(p -> {
          Type typeToGenerate = p.getType().bindGenerics(typeBinding);
          logger.info("Bounded param {} to {}", p, typeToGenerate);

          return generateArg(typeToGenerate);
        })
        .filter(Optional::isPresent)
        .map(Optional::get)
        .toList();

    if (args.size() != callable.getParams().size()) {
      logger.warn("Could not generate all arguments");
      return false;
    }

    VarReference returnValue = null;
    if (callable.returnsValue()) {
      returnValue = createVariable(callable.getReturnType().bindGenerics(typeBinding));
      typeBindings.put(returnValue, typeBinding);
    }

    var stmt = callable.toStmt(this, args, returnValue);
    statements.add(stmt);
    return true;
  }

  private List<Type> substituteGenerics(List<Type> generics, TypeBinding binding) {
    return generics.stream()
        .map(g -> {
          if (g.isGeneric()) {
            return binding.getBindingFor(g.asGeneric());
          } else {
            return g;
          }
        })
        .peek(Objects::requireNonNull)
        .toList();
  }

  public Optional<VarReference> generateArg(Param param) {
    return generateArg(Objects.requireNonNull(param), new HashSet<>());
  }

  private Optional<VarReference> generateArg(Param param,
      Set<Type> typesToGenerate) {
    logger.info("Starting to generate an argument for param {}", param);
    if (param.isPrimitive()) {
      var type = param.getType().asPrimitive();
      return Optional.of(generatePrimitive(type));
    } else if (param.isGeneric()) {
      throw new RuntimeException("Not allowed");
    } else {
      var generators = hirAnalysis.generatorsOf(param.getType());
      return generateArgFromGenerators(param.getType(), generators, typesToGenerate);
    }
  }

  /*
   * A convenience method, when we need to generate an arg for a type directly instead of a generic
   * param
   */
  public Optional<VarReference> generateArg(Type type) {
    return generateArg(Objects.requireNonNull(type), new HashSet<>());
  }

  private Optional<VarReference> generateArg(Type type,
      Set<Type> typesToGenerate) {
    logger.info("Starting to generate an argument for type {}", type);
    // TODO reuse existing variables if possible, e.g., introduce a boolean flag or so
    if (type.isPrim()) {
      return Optional.of(generatePrimitive(type.asPrimitive()));
    } else if (type.isComplex() || type.isEnum()) {
      var generators = hirAnalysis.generatorsOf(type);
      return generateArgFromGenerators(type, generators, typesToGenerate);
    } else if (type.isRef()) {
      var generators = hirAnalysis.generatorsOf(type);
      return generateArgFromGenerators(type, generators, typesToGenerate);
    } else {
      throw new RuntimeException("Not implemented: " + type);
    }
  }

  /**
   * Recursively generate an actual type, e.g. if we generate std::vec::Vec&lt;T&gt;, then also
   * generate an actual type for T.
   *
   * @param generic Generic type to substitute.
   * @return Fully substituted real type.
   */
  private Optional<Type> getTypeFor(Generic generic) {
    var primitive = getPrimitiveTypeFor(generic);
    if (primitive.isPresent()) {
      return primitive.map(p -> p);
    } else {
      return getComplexTypeFor(generic);
    }
  }

  private Optional<Prim> getPrimitiveTypeFor(Generic generic) {
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
  private Optional<Type> getComplexTypeFor(Generic generic) {
    var bounds = generic.getBounds();

    var possibleTypes = hirAnalysis.typesImplementing(bounds);
    if (possibleTypes.isEmpty()) {
      return Optional.empty();
    }

    var type = Rnd.choice(possibleTypes);
    var boundedGenerics = type.generics().stream()
        .map(g -> getTypeFor(g.asGeneric()))
        .filter(Optional::isPresent)
        .map(Optional::get)
        .toList();

    if (boundedGenerics.size() != type.generics().size()) {
      return Optional.empty();
    }

    return Optional.of(type.replaceGenerics(boundedGenerics));
  }

  private List<Type> bindGenerics(Callable callable, List<VarReference> args) {
    throw new RuntimeException("Not implemented");
  }

  private VarReference generatePrimitive(Prim prim) {
    logger.info("Starting to generate a primitive");
    var val = prim.random();
    var var = createVariable(prim);
    var stmt = new PrimitiveStmt(this, var, val);
    statements.add(stmt);
    return var;
  }

  private Optional<VarReference> generateArgFromGenerators(Type type, List<Callable> generators,
      Set<Type> typesToGenerate) {

    logger.info("Starting to generate a " + type);
    boolean retry = true;
    Callable generator = null;
    while (retry && !generators.isEmpty()) {
      retry = false;

      var candidateGenerator = Rnd.choice(generators);
      var paramTypes = candidateGenerator.getParams().stream().map(Param::getType)
          .collect(Collectors.toSet());
      paramTypes.retainAll(typesToGenerate);
      if (!paramTypes.isEmpty()) {
        // We already try to generate a type which is needed as an argument for the call
        // Hence, this would probably lead to infinite recursive chain. Remove the
        // generator and retry with another one.
        generators.remove(candidateGenerator);
        retry = true;
      } else {
        generator = candidateGenerator;
      }
    }

    // fn foo<A>(x: A, v: Vec<A>) -> Option<A>;

    if (generator == null) {
      logger.warn("Could not find appropriate generator for {}", type);
      return Optional.empty();
    }

    logger.info("Selected generator: {} (Total: {})", generator, generators.size());

    var typeBinding = TypeUtil.getNecessaryBindings(generator.getReturnType(), type);

    generator.getParams().stream()
        .map(Param::getType)
        .map(TypeUtil::getDeepGenerics)
        .peek(deepGenerics -> deepGenerics.removeAll(typeBinding.getGenerics()))
        .forEach(typeBinding::addGenerics);

    if (generator.isMethod()) {
      var generics = generator.getParent().generics().stream().map(Type::asGeneric)
          .collect(Collectors.toSet());
      generics.removeAll(typeBinding.getGenerics());
      typeBinding.addGenerics(generics);
    }

    if (generator.returnsValue()) {
      var generics = TypeUtil.getDeepGenerics(generator.getReturnType());
      generics.removeAll(typeBinding.getGenerics());
      typeBinding.addGenerics(generics);
    }

    typeBinding.getUnboundGenerics()
        .stream()
        .map(g -> Pair.with(g, getTypeFor(g)))
        .filter(p -> p.getValue1().isPresent())
        .forEach(p -> typeBinding.bindGeneric(p.getValue0(), p.getValue1().get()));

    if (typeBinding.hasUnboundedGeneric()) {
      logger.warn("Could not bind all generics: {}", typeBinding.getUnboundGenerics());

      generators.remove(generator);
      return generateArgFromGenerators(type, generators, typesToGenerate);
      // instead of giving up, try another generator
      // return Optional.empty();
    }

    List<VarReference> args = generator.getParams().stream()
        .map(p -> {
          var usableVars = unconsumedVariablesOfType(p.getType());
          if (!instantiatedTypes().contains(p.getType()) || usableVars.isEmpty()) {
            var extendedTypesToGenerate = new HashSet<>(typesToGenerate);
            extendedTypesToGenerate.add(type);
            return generateArg(p.getType().bindGenerics(typeBinding), extendedTypesToGenerate);
          } else {
            // TODO check if those are used
            var var = Rnd.choice(usableVars);
            return Optional.of(var);
          }
        })
        .filter(Optional::isPresent)
        .map(Optional::get)
        .toList();
    if (args.size() != generator.getParams().size()) {

      generators.remove(generator);
      return generateArgFromGenerators(type, generators, typesToGenerate);

      // instead of giving up, try another generator
      //return Optional.empty();
    }

    VarReference returnValue = null;
    if (generator.returnsValue()) {
      if (type.isRef() && !generator.getReturnType().isRef()) {
        // Unwrap the type
        var innerType = type.asRef().getInnerType();
        returnValue = createVariable(innerType);
      } else if (!type.isRef() && generator.getReturnType().isRef()) {
        throw new RuntimeException("Not implemented");
      } else {
        returnValue = createVariable(type);
      }

      typeBindings.put(returnValue, typeBinding);
    }

    var stmt = generator.toStmt(this, args, returnValue);
    addStmt(stmt);
    return Optional.ofNullable(returnValue);
  }

  private VarReference createVariable(Type type) {
    logger.info("Created variable of type {}", type);
    return new VarReference(this, type);
  }

  public String visit(Visitor visitor) {
    return visitor.visitTestCase(this);
  }

  @Override
  public String toString() {
    var sb = new StringBuilder();
    sb.append("fn ").append(getName()).append("() {\n");
    for (Statement statement : statements) {
      sb.append("    ").append(statement).append("\n");
    }
    sb.append("}");

    return sb.toString();
  }

  @Override
  public TestCase copy() {
    return new TestCase(this);
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    TestCase testCase = (TestCase) o;
    return id == testCase.id && statements.equals(testCase.statements);
  }

  @Override
  public int hashCode() {
    return Objects.hash(id, statements);
  }

  @Override
  public TestCase self() {
    return this;
  }

  public Map<Integer, Double> getCoverage() {
    if (coverage == null) {
      throw new IllegalStateException("Branch execution info is not initialized");
    }

    return coverage;
  }
}