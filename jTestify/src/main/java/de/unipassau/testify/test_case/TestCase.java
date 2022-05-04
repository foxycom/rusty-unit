package de.unipassau.testify.test_case;

import static de.unipassau.testify.Constants.P_LOCAL_VARIABLES;
import static java.util.stream.Collectors.toCollection;

import com.google.common.base.Preconditions;
import de.unipassau.testify.Constants;
import de.unipassau.testify.generators.TestIdGenerator;
import de.unipassau.testify.hir.TyCtxt;
import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import de.unipassau.testify.metaheuristics.operators.Crossover;
import de.unipassau.testify.metaheuristics.operators.Mutation;
import de.unipassau.testify.mir.BasicBlock;
import de.unipassau.testify.mir.MirAnalysis;
import de.unipassau.testify.test_case.callable.ArrayInit;
import de.unipassau.testify.test_case.callable.Callable;
import de.unipassau.testify.test_case.callable.Method;
import de.unipassau.testify.test_case.callable.RefItem;
import de.unipassau.testify.test_case.callable.TupleInit;
import de.unipassau.testify.test_case.statement.PrimitiveStmt;
import de.unipassau.testify.test_case.statement.Statement;
import de.unipassau.testify.test_case.type.Array;
import de.unipassau.testify.test_case.type.Generic;
import de.unipassau.testify.test_case.type.Ref;
import de.unipassau.testify.test_case.type.Tuple;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.TypeBinding;
import de.unipassau.testify.test_case.type.prim.Int.ISize;
import de.unipassau.testify.test_case.type.prim.Prim;
import de.unipassau.testify.test_case.type.traits.Trait;
import de.unipassau.testify.test_case.type.traits.std.default_.Default;
import de.unipassau.testify.test_case.type.traits.std.marker.Copy;
import de.unipassau.testify.test_case.visitor.LineNumberDebugVisitor;
import de.unipassau.testify.test_case.visitor.TypeBindingStringVisitor;
import de.unipassau.testify.test_case.visitor.Visitor;
import de.unipassau.testify.util.Rnd;
import de.unipassau.testify.util.TypeUtil;
import java.util.ArrayList;
import java.util.BitSet;
import java.util.Collections;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.Set;
import java.util.stream.Collectors;
import java.util.stream.IntStream;
import org.javatuples.Pair;
import org.javatuples.Quartet;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class TestCase extends AbstractTestCaseChromosome<TestCase> {

  private static final Logger logger = LoggerFactory.getLogger(TestCase.class);
  private final TyCtxt tyCtxt;

  private int id;
  private List<Statement> statements;
  private Map<MinimizingFitnessFunction<TestCase>, Double> coverage;
  private MirAnalysis<TestCase> mir;
  private TestCaseMetadata metadata;

  public TestCase(int id, TyCtxt tyCtxt, Mutation<TestCase> mutation,
      Crossover<TestCase> crossover, MirAnalysis<TestCase> mir) {
    super(mutation, crossover);

    this.id = id;
    this.tyCtxt = tyCtxt;
    this.statements = new ArrayList<>();
    this.coverage = new HashMap<>();
    this.mir = mir;
    this.metadata = new TestCaseMetadata(id);
  }

  public TestCase(TestCase other) {
    super(other.getMutation(), other.getCrossover());
    this.id = TestIdGenerator.get();
    this.tyCtxt = other.tyCtxt;
    this.statements = other.statements.stream().map(s -> s.copy(this))
        .collect(toCollection(ArrayList::new));
    this.coverage = new HashMap<>();
    this.mir = other.mir;
    this.metadata = new TestCaseMetadata(id);
  }

  public TyCtxt getHirAnalysis() {
    return tyCtxt;
  }

  @Override
  public int getId() {
    return id;
  }

  public void setId(int id) {
    this.id = id;
  }

  public MirAnalysis<TestCase> mir() {
    return mir;
  }

  @Override
  public int size() {
    return statements.size();
  }

  public List<Statement> getStatements() {
    return statements;
  }

  @Override
  public TestCaseMetadata metadata() {
    return metadata;
  }

  public Optional<Statement> getLastCrateStmt() {
    return IntStream.range(0, statements.size())
        .mapToObj(i -> statements.get(statements.size() - i - 1))
        .filter(s -> s.getSrcFilePath() != null)
        .findFirst();
  }

  public Optional<String> getFilePathBinding() {
    var paths = statements.stream().filter(s -> s.getSrcFilePath() != null)
        .filter(s -> !s.isPublic())
        .map(Statement::getSrcFilePath)
        .collect(Collectors.toSet());
    Preconditions.checkState(paths.size() <= 1);

    return paths.stream().findFirst();
  }

  public boolean isValid() {
    return statements.stream().filter(s -> s.getSrcFilePath() != null)
        .filter(s -> !s.isPublic())
        .map(Statement::getSrcFilePath)
        .collect(Collectors.toSet())
        .size() <= 1;
  }

  public void setStatements(List<Statement> statements) {
    this.statements = statements;
  }

  public void setCoverage(BasicBlock branch, double distance) {
    coverage.put(branch, distance);
  }

  public void setCoverage(Map<MinimizingFitnessFunction<TestCase>, Double> coverage) {
    if (coverage == null) {
      return;
    }
    this.coverage = coverage;
  }

  public boolean isCutable() {
    return statements.size() > 1;
  }

  public Optional<VarReference> satisfyParameter(int pos, Type parameter) {
    List<VarReference> usableVariables;
    if (parameter.isRef()) {
      usableVariables = borrowableVariablesOfType(parameter, pos);
    } else {
      usableVariables = consumableVariablesOfType(parameter, pos);
    }

    VarReference var;
    if (!usableVariables.isEmpty()) {
      var = Rnd.choice(usableVariables);
    } else {
      var generatedArg = generateArg(parameter);
      if (generatedArg.isPresent()) {
        var = generatedArg.get();
      } else {
        logger.warn("Could not generate any argument for " + parameter);
        return Optional.empty();
      }
    }
    return Optional.of(var);
  }

  /**
   * We assume that all the variables used in the statement do not exist in this test case, because
   * the stmt comes from another one.
   */
  public List<VarReference> satisfyParameters(int pos, Statement stmt) {
    var paramTypes = stmt.actualParamTypes();

    List<VarReference> variables = new ArrayList<>(paramTypes.size());
    for (Type paramType : paramTypes) {
      var var = satisfyParameter(pos, paramType);

      // Don't use the same variable twice in an invocation
      if (var.isPresent() && !variables.contains(var.get())) {
        variables.add(var.get());
      }
    }

    return variables;
  }

  public void insertStmt(int pos, Statement stmt) {
    statements.add(pos, stmt);
  }

  public void addStmt(Statement stmt) {
    int insertPosition = 0;
    if (stmt.args().isEmpty()) {
      // Insert position is 0
      insertStmt(0, stmt);
    } else {
      insertPosition = stmt.args().stream().map(VarReference::position)
          .reduce(0, Integer::max);
      insertStmt(Integer.min(size(), insertPosition + 1), stmt);
    }
  }

  public void appendStmt(Statement stmt) {
    statements.add(stmt);
  }

  public void removeAllStmts() {
    statements.clear();
  }

  public int removeStmt(Statement stmt) {
    if (!stmt.returnsValue()) {
      statements.remove(stmt);
      return 0;
    }

    var returnValue = stmt.returnValue().orElseThrow();
    var usingStmts = returnValue.usedAt()
        .stream()
        .map(this::stmtAt)
        .map(Optional::orElseThrow)
        .collect(toCollection(ArrayList::new));
    Collections.reverse(usingStmts);
    usingStmts.forEach(this::removeStmt);

    statements.remove(stmt);
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
      logger.warn("({}) Could not find position of a statement in test", id);
    }
    return Optional.of(pos);
  }

  public Optional<Integer> varPosition(VarReference var) {
    throw new RuntimeException("Not implemented");
  }

  public String getName() {
    return String.format("%s_%d", Constants.TEST_PREFIX, id);
  }

  public VarReference referenceVariable(VarReference variable, boolean mutable) {
    if (variable.testCase() != this) {
      throw new IllegalStateException("The test does not contain this variable");
    }

    if (variable.type().isRef()) {
      throw new RuntimeException("Referencing variable cannot be referenced");
    }

    RefItem refItem;
    if (mutable) {
      refItem = RefItem.MUTABLE;
    } else {
      refItem = RefItem.IMMUTABLE;
    }

    var typeBinding = new TypeBinding();
    typeBinding.bindGeneric(RefItem.T, variable.type());

    var returnType = new Ref(variable.type(), mutable);
    var returnValue = createVariable(returnType);
    returnValue.setBinding(typeBinding);

    var stmt = refItem.toStmt(this, Collections.singletonList(variable), returnValue);
    addStmt(stmt);
    return returnValue;
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
        .collect(toCollection(ArrayList::new));
  }

  public List<VarReference> borrowableVariablesOfType(Type type, int untilPos) {
    return IntStream.range(0, untilPos)
        .mapToObj(statements::get)
        .map(Statement::returnValue)
        .filter(Optional::isPresent)
        .map(Optional::get)
        .filter(var -> var.type().equals(type) && var.isBorrowableAt(untilPos))
        .collect(toCollection(ArrayList::new));
  }

  public List<VarReference> consumableVariablesOfType(Type type, int untilPos) {
    return IntStream.range(0, untilPos)
        .mapToObj(statements::get)
        .map(Statement::returnValue)
        .filter(Optional::isPresent)
        .map(Optional::get)
        .filter(var -> var.type().equals(type) && var.isConsumableAt(untilPos))
        .collect(toCollection(ArrayList::new));
  }

  public List<VarReference> unconsumedVariablesOfType(Type type) {
    return statements.stream()
        .map(Statement::returnValue)
        .filter(Optional::isPresent)
        .map(Optional::get)
        .filter(var -> var.type().equals(type) && !var.isConsumed())
        .collect(toCollection(ArrayList::new));
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
        .collect(toCollection(ArrayList::new));
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
        .collect(toCollection(ArrayList::new));
  }

  public List<Quartet<VarReference, Callable, Integer, Integer>> availableCallables() {
    throw new RuntimeException("Not implemented");
  }

  public Optional<VarReference> insertRandomStmt() {
    var filePathBinding = getFilePathBinding();
    Callable callable;

    var possiblemMethods = tyCtxt.methodsOf(variables());
    if (Rnd.get().nextDouble() < P_LOCAL_VARIABLES && !possiblemMethods.isEmpty()) {
      var variableAndMethod = Rnd.choice(possiblemMethods);
      return insertMethodOnExistingVariable(variableAndMethod.getValue0(),
          variableAndMethod.getValue1());
    } else if (filePathBinding.isPresent()) {
      callable = Rnd.choice(tyCtxt.getCallables(filePathBinding.get(), true));
    } else {
      callable = Rnd.choice(tyCtxt.getCallables(null, true));
    }

    logger.info("({}) Inserting random stmt. Selected callable: {}", id, callable);

    return insertCallable(callable);
  }

  private Optional<VarReference> insertMethodOnExistingVariable(VarReference owner, Method method) {
    logger.info("({}) Inserting a method on existing variable {}", id, owner);
    var args = new ArrayList<VarReference>(method.getParams().size());

    LinkedHashSet<Generic> generics = method.getParams().stream()
        .map(Param::getType)
        .map(TypeUtil::getDeepGenerics)
        .collect(LinkedHashSet::new, LinkedHashSet::addAll, LinkedHashSet::addAll);

    generics.addAll(
        method.getParent().generics().stream().filter(Type::isGeneric).map(Type::asGeneric).collect(Collectors.toSet())
    );

    if (method.returnsValue()) {
      generics.addAll(TypeUtil.getDeepGenerics(method.getReturnType()));
    }

    var ownerTypeBinding = TypeBinding.fromTypes(method.getSelfParam().getType(), owner.type());
    var genericsTypeBinding = new TypeBinding(generics);
    var typeBinding = ownerTypeBinding.leftOuterMerge(genericsTypeBinding);

    typeBinding.getUnboundGenerics().stream().map(g -> Pair.with(g, getTypeFor(g)))
        .filter(p -> p.getValue1().isPresent())
        .forEach(p -> typeBinding.bindGeneric(p.getValue0(), p.getValue1().get()));

    VarReference selfArgument = owner;
    if (method.getSelfParam().isByReference() && !owner.type().isRef()) {
      var type = method.getSelfParam().getType().bindGenerics(typeBinding);
      selfArgument = createVariable(type);
      Statement refStmt = (method.getSelfParam().getType().asRef().isMutable())
          ? RefItem.MUTABLE.toStmt(this, List.of(owner), selfArgument)
          : RefItem.IMMUTABLE.toStmt(this, List.of(owner), selfArgument);

      var refTypeBinding = TypeBinding.fromTypes(type, owner.type());
      selfArgument.setBinding(refTypeBinding);
      statements.add(refStmt);
    }

    args.add(selfArgument);

    if (method.getSelfParam().isGeneric()) {
      // We know the concrete type of the owner variable at this point, so bind it
      // We have to set all other bindings, as there might be some
      typeBinding.bindGeneric(method.getSelfParam().getType().asGeneric(), owner.type());
    }

    typeBinding.getUnboundGenerics().stream()
        .map(g -> Pair.with(g, getTypeFor(g)))
        .filter(p -> p.getValue1().isPresent())
        .forEach(p -> typeBinding.bindGeneric(p.getValue0(), p.getValue1().get()));

    if (typeBinding.hasUnboundedGeneric()) {
      logger.warn("({}) Could not bind all generics: {}", id, typeBinding.getUnboundGenerics());
      return Optional.empty();
    }

    for (int i = 1; i < method.getParams().size(); i++) {
      var param = method.getParams().get(i);
      var boundParam = param.bindGenerics(typeBinding);
      var arg = generateArg(boundParam);
      arg.ifPresent(args::add);
    }

    if (args.size() != method.getParams().size()) {
      return Optional.empty();
    }

    VarReference returnValue = null;
    if (method.returnsValue()) {
      // TODO: 14.02.22 there probably will be some troubles with type binding
      returnValue = createVariable(method.getReturnType().bindGenerics(typeBinding));
      returnValue.setBinding(typeBinding);
    }

    var stmt = method.toStmt(this, args, returnValue);
    statements.add(stmt);
    return Optional.ofNullable(returnValue);
  }

  public Optional<VarReference> insertCallable(Callable callable) {
    logger.debug("({}) Inserting callable {}", id, callable);

    LinkedHashSet<Generic> generics = callable.getParams().stream()
        .map(Param::getType)
        .map(TypeUtil::getDeepGenerics)
        .collect(LinkedHashSet::new, LinkedHashSet::addAll, LinkedHashSet::addAll);
    if (callable.isMethod()) {
      generics.addAll(callable.getParent().generics().stream().map(Type::asGeneric)
          .collect(Collectors.toSet()));
    }

    if (callable.returnsValue()) {
      generics.addAll(TypeUtil.getDeepGenerics(callable.getReturnType()));
    }

    logger.debug("({}) It's generics are: {}", id, generics);

    var typeBinding = new TypeBinding(generics);

    generics.stream().map(g -> Pair.with(g, getTypeFor(g))).filter(p -> p.getValue1().isPresent())
        .forEach(p -> typeBinding.bindGeneric(p.getValue0(), p.getValue1().get()));
    if (typeBinding.hasUnboundedGeneric()) {
      logger.warn("({}) Could not bind all generics: {}", id, typeBinding.getUnboundGenerics());
      return Optional.empty();
    }

    var args = callable.getParams().stream()
        .map(p -> {
          Type typeToGenerate = p.getType().bindGenerics(typeBinding);
          logger.debug("({}) Bounded param {} to {}", id, p, typeToGenerate);

          return generateArg(typeToGenerate);
        })
        .filter(Optional::isPresent)
        .map(Optional::get)
        .collect(toCollection(ArrayList::new));

    if (args.size() != callable.getParams().size()) {
      logger.warn("({}) Could not generate all arguments", id);
      return Optional.empty();
    }

    VarReference returnValue = null;
    if (callable.returnsValue()) {
      returnValue = createVariable(callable.getReturnType().bindGenerics(typeBinding));
      returnValue.setBinding(typeBinding);
    }

    var stmt = callable.toStmt(this, args, returnValue);
    logger.info("({}) Pushing " + stmt + " at the end of the test", id);
    statements.add(stmt);
    return Optional.ofNullable(returnValue);
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

  /**
   * Either looks for an existing usable variable or creates a new variable to use as argument
   * for the given type.
   *
   * @param type The type to get an argument for.
   * @param usableBeforeLine The line number the possible argument shall be usable until (exclusively).
   * @return Argument if possible.
   */
  public Optional<VarReference> getArg(Type type, int usableBeforeLine) {
    Optional<VarReference> arg = Optional.empty();
    if (type.isRef()) {
      var borrowableVariables = borrowableVariablesOfType(type, usableBeforeLine);
      if (!borrowableVariables.isEmpty()) {
        var rawArg = Rnd.choice(borrowableVariables);
        if (rawArg.type().isRef()) {
          // Don't reference a reference, just use it directly
          arg = Optional.of(rawArg);
        } else {
          arg = Optional.of(referenceVariable(rawArg, true));
        }
      }
    } else {
      var consumableVariables = consumableVariablesOfType(type, usableBeforeLine);
      if (!consumableVariables.isEmpty()) {
        arg = Optional.of(Rnd.choice(consumableVariables));
      }
    }

    if (arg.isPresent()) {
      return arg;
    } else {
      return generateArg(type);
    }
  }

  /**
   * Creates a new variable as argument for the given parameter.
   *
   * @param param The parameter to create an argument for.
   * @return Generated argument if successful.
   */
  public Optional<VarReference> generateArg(Param param) {
    return generateArg(Objects.requireNonNull(param), new HashSet<>());
  }

  /**
   * Creates a new variable as argument for the given parameter. Also considers types that are
   * being initialized recursively, such that we avoid infinite loops while creating dependencies.
   *
   * @param param The parameter to create an argument for.
   * @param typesToGenerate Types that are already being generated.
   * @return Generated argument if possible.
   */
  private Optional<VarReference> generateArg(Param param,
      Set<Type> typesToGenerate) {
    logger.debug("({}) Starting to generate an argument for param {}", id, param);

    if (param.isPrimitive()) {
      var type = param.getType().asPrimitive();
      return Optional.of(generatePrimitive(type));
    } else if (param.isGeneric()) {
      throw new RuntimeException("Not allowed");
    } else {
      var generators = tyCtxt.generatorsOf(param.getType(), getFilePathBinding().orElse(null));
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
    logger.debug("({}) Starting to generate an argument for type {}", id, type);
    if (type.isPrim()) {
      return Optional.of(generatePrimitive(type.asPrimitive()));
    } else if (type.isStruct() || type.isEnum()) {
      var generators = tyCtxt.generatorsOf(type, getFilePathBinding().orElse(null));
      logger.debug("({}) Found " + generators.size() + " generators", id);
      return generateArgFromGenerators(type, generators, typesToGenerate);
    } else if (type.isRef()) {
      //var generators = tyCtxt.generatorsOf(type, getFilePathBinding().orElse(null));
      //logger.debug("({}) Found " + generators.size() + " generators", id);
      var reference = type.asRef();
      return generateReference(reference, typesToGenerate);
//      return generateArgFromGenerators(type, generators, typesToGenerate);
    } else if (type.isArray()) {
      return generateArray(type.asArray(), typesToGenerate);
    } else if (type.isTuple()) {
      var tuple = type.asTuple();
      return generateTuple(tuple, typesToGenerate);
    } else {
      throw new RuntimeException("Not implemented: " + type);
    }
  }

  private Optional<VarReference> generateReference(Ref ref, Set<Type> typesToGenerate) {
    RefItem refItem;
    if (ref.isMutable()) {
      refItem = RefItem.MUTABLE;
    } else {
      refItem = RefItem.IMMUTABLE;
    }

    var generics = TypeUtil.getDeepGenerics(ref);
    var typeBinding = new TypeBinding((LinkedHashSet<Generic>) generics);
    // Set for all generics some appropriate random type that complies with all constraints
    // and type bounds
    generics.stream().map(g -> Pair.with(g, getTypeFor(g))).filter(p -> p.getValue1().isPresent())
        .forEach(p -> typeBinding.bindGeneric(p.getValue0(), p.getValue1().get()));
    if (typeBinding.hasUnboundedGeneric()) {
      logger.warn("({}) Could not bind all generics: {}", id, typeBinding.getUnboundGenerics());
      return Optional.empty();
    }

    var refType = ref.getInnerType().bindGenerics(typeBinding);
    var extendedTypesToGenerate = new HashSet<>(typesToGenerate);
    extendedTypesToGenerate.add(ref);
    var arg = generateArg(refType, extendedTypesToGenerate);
    if (arg.isEmpty()) {
      return Optional.empty();
    }

    var returnValue = createVariable(ref.bindGenerics(typeBinding));
    returnValue.setBinding(typeBinding);

    var stmt = refItem.toStmt(this, Collections.singletonList(arg.get()), returnValue);
    addStmt(stmt);
    return Optional.of(returnValue);
  }

  private Optional<VarReference> generateTuple(Tuple tuple, Set<Type> typesToGenerate) {
    var params = tuple.getTypes().stream().map(t -> new Param(t, false, null)).toList();
    var tupleInit = new TupleInit(params);
    Set<Generic> generics = TypeUtil.getDeepGenerics(tuple);
    var typeBinding = new TypeBinding((LinkedHashSet<Generic>) generics);
    // Set for all generics some appropriate random type that complies with all constraints
    // and type bounds
    generics.stream().map(g -> Pair.with(g, getTypeFor(g))).filter(p -> p.getValue1().isPresent())
        .forEach(p -> typeBinding.bindGeneric(p.getValue0(), p.getValue1().get()));
    if (typeBinding.hasUnboundedGeneric()) {
      logger.warn("({}) Could not bind all generics: {}", id, typeBinding.getUnboundGenerics());
      return Optional.empty();
    }

    var extendedTypesToGenerate = new HashSet<>(typesToGenerate);
    extendedTypesToGenerate.add(tuple);
    var tupleTypes = tuple.getTypes().stream().map(t -> t.bindGenerics(typeBinding)).toList();
    var args = tupleTypes.stream()
        .map(innerType -> generateArg(innerType, extendedTypesToGenerate))
        .filter(Optional::isPresent)
        .map(Optional::get)
        .toList();

    if (args.size() == tuple.getTypes().size()) {
      var returnValue = createVariable(tuple.bindGenerics(typeBinding));
      returnValue.setBinding(typeBinding);
      var stmt = tupleInit.toStmt(this, args, returnValue);
      addStmt(stmt);
      return Optional.of(returnValue);
    } else {
      return Optional.empty();
    }
  }

  private Optional<VarReference> generateArray(Array array, Set<Type> typesToGenerate) {
    // TODO: 27.02.22 1) [T; N] where T: Default (and N <= 32)
    // TODO: 27.02.22  [T; N] where T: Copy
    // TODO: 27.02.22 literal array init
    if (array.implementedTraits().contains(Default.getInstance())) {
      throw new RuntimeException("Not implemented");
    } else if (array.implementedTraits().contains(Copy.getInstance())) {
      throw new RuntimeException("Not implemented");
    } else {
      var arrayInit = new ArrayInit(array);

      Set<Generic> generics = TypeUtil.getDeepGenerics(array);
      var typeBinding = new TypeBinding((LinkedHashSet<Generic>) generics);

      // Set for all generics some appropriate random type that complies with all constraints
      // and type bounds
      generics.stream().map(g -> Pair.with(g, getTypeFor(g))).filter(p -> p.getValue1().isPresent())
          .forEach(p -> typeBinding.bindGeneric(p.getValue0(), p.getValue1().get()));
      if (typeBinding.hasUnboundedGeneric()) {
        logger.warn("({}) Could not bind all generics: {}", id, typeBinding.getUnboundGenerics());
        return Optional.empty();
      }

      var extendedTypesToGenerate = new HashSet<>(typesToGenerate);
      extendedTypesToGenerate.add(array);
      var actualElementsType = array.type().bindGenerics(typeBinding);
      var elements = IntStream.range(0, array.length())
          .mapToObj(i -> generateArg(actualElementsType, extendedTypesToGenerate))
          .filter(Optional::isPresent)
          .map(Optional::get)
          .collect(toCollection(ArrayList::new));

      if (elements.size() != array.length()) {
        logger.warn("Could not generate all elements for {}", array);
        return Optional.empty();
      }

//      if (generator.returnsValue()) {
//        if (type.isRef() && !generator.getReturnType().isRef()) {
//          // Unwrap the type
//          var innerType = type.asRef().getInnerType();
//          returnValue = createVariable(innerType);
//        } else if (!type.isRef() && generator.getReturnType().isRef()) {
//          throw new RuntimeException("Not implemented");
//        } else {
//          returnValue = createVariable(type);
//        }
//
//        returnValue.setBinding(typeBinding);
//      }

      var returnValue = createVariable(array.bindGenerics(typeBinding));
      returnValue.setBinding(typeBinding);

      var stmt = arrayInit.toStmt(this, elements, returnValue);
      addStmt(stmt);
      return Optional.of(returnValue);
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

    var possibleTypes = tyCtxt.typesImplementing(bounds);
    if (possibleTypes.isEmpty()) {
      return Optional.empty();
    }

    var type = Rnd.choice(possibleTypes);
    return Optional.of(type);
  }

  private VarReference generatePrimitive(Prim prim) {
    logger.debug("({}) Starting to generate a primitive", id);
    var val = prim.random();
    var var = createVariable(prim);
    var stmt = new PrimitiveStmt(this, var, val);
    statements.add(0, stmt);
    return var;
  }

  /**
   * Unwraps till we get the required type.
   *
   * @param var The variable to unwrap
   * @param requiredType The inner type we look for.
   * @return Unwrapped variable
   */
  private Optional<VarReference> unwrapVariable(VarReference var, Type requiredType) {
    var method = var.type().unwrapMethod();
    var unwrappedVar = insertMethodOnExistingVariable(var, method);
    if (unwrappedVar.isPresent() && !unwrappedVar.get().type().equals(requiredType)) {
      return unwrapVariable(unwrappedVar.get(), requiredType);
    }

    return unwrappedVar;
  }

  private Optional<VarReference> generateArgFromGenerators(Type type, List<Callable> generators,
      Set<Type> typesToGenerate) {

    logger.debug("({}) Starting to generate a {} with {} generator options", id, type,
        generators.size());
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
        logger.debug("({}) Removing candidate generator since we already try to generate it", id);
        generators.remove(candidateGenerator);
        retry = true;
      } else {
        generator = candidateGenerator;
      }
    }

    // fn foo<A>(x: A, v: Vec<A>) -> Option<A>;

    if (generator == null) {
      logger.warn("({}) Could not find appropriate generator for {}", id, type);
      return Optional.empty();
    }

    logger.debug("({}) Selected generator: {} (Total: {})", id, generator, generators.size());
    if (generator.getReturnType().wraps(type)) {
      var wrappedValue = insertCallable(generator);
      return wrappedValue.flatMap(varReference -> unwrapVariable(varReference, type));
    }

    TypeBinding typeBinding = TypeUtil.getNecessaryBindings(generator.getReturnType(), type);
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

    // Before bounding randomly other generics, look for generators that already can
    // satisfy our partially bounded type

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

    var args = generator.getParams().stream()
        .map(p -> {
          var extendedTypesToGenerate = new HashSet<>(typesToGenerate);
          extendedTypesToGenerate.add(type);
          return generateArg(p.getType().bindGenerics(typeBinding), extendedTypesToGenerate);
        })
        .filter(Optional::isPresent)
        .map(Optional::get)
        .collect(toCollection(ArrayList::new));
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

      returnValue.setBinding(typeBinding);
    }

    var stmt = generator.toStmt(this, args, returnValue);
    addStmt(stmt);
    return Optional.ofNullable(returnValue);
  }

  private VarReference createVariable(Type type) {
    logger.debug("({}) Created variable of type {}", id, type);
    return new VarReference(this, type);
  }

  public Map<MinimizingFitnessFunction<TestCase>, Double> branchDistance() {
    return coverage;
  }

  public String getTypeBindingsString() {
    var sb = new StringBuilder();
    var visitor = new TypeBindingStringVisitor(this);
    /*typeBindings.forEach((key, value) -> sb.append(visitor.getVariableName(key)).append(": ")
        .append(visitor.visit(value)));*/
    return sb.toString();
  }

  public String visit(Visitor visitor) {
    return visitor.visitTestCase(this);
  }

  @Override
  public String toString() {
    var visitor = new LineNumberDebugVisitor();
    return visit(visitor);
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

}
