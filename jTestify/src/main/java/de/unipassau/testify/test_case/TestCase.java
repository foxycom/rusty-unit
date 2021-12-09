package de.unipassau.testify.test_case;

import de.unipassau.testify.ddg.Dependency;
import de.unipassau.testify.ddg.Node;
import de.unipassau.testify.hir.HirAnalysis;
import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.metaheuristics.operators.Crossover;
import de.unipassau.testify.metaheuristics.operators.Mutation;
import de.unipassau.testify.mir.Branch;
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
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.Set;
import java.util.stream.Collectors;
import org.javatuples.Pair;
import org.javatuples.Quartet;
import org.jgrapht.Graph;
import org.jgrapht.graph.DirectedMultigraph;

public class TestCase extends AbstractTestCaseChromosome<TestCase> {

  private int id;
  private List<Statement> statements;
  private Map<Branch, Double> coverage;
  private Set<VarReference> variables;
  private Graph<Node, Dependency> ddg;
  private HirAnalysis hirAnalysis;
  private Map<VarReference, TypeBinding> bindings;

  public TestCase(int id, HirAnalysis hirAnalysis, Mutation<TestCase> mutation,
      Crossover<TestCase> crossover) {
    super(mutation, crossover);
    this.id = id;
    this.hirAnalysis = hirAnalysis;
    this.statements = new ArrayList<>();
    this.ddg = new DirectedMultigraph<>(Dependency.class);
    this.variables = new HashSet<>();
    this.bindings = new HashMap<>();
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

  public List<Statement> getStatements() {
    return statements;
  }


  public void setStatements(List<Statement> statements) {
    this.statements = statements;
  }

  public Map<Branch, Double> getCoverage() {
    return coverage;
  }

  public void setCoverage(Map<Branch, Double> coverage) {
    this.coverage = coverage;
  }

  public Graph<Node, Dependency> getDdg() {
    return ddg;
  }

  public boolean isCutable() {
    return statements.size() > 1;
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
    } else {
      throw new RuntimeException("Not implemented");
    }
  }

  public int removeStmt(Statement stmt) {
    throw new RuntimeException("Not implemented");
  }

  public void removeStmt(int pos) {
    throw new RuntimeException("Not implemented");
  }

  public Optional<Integer> stmtPosition(Statement stmt) {
    throw new RuntimeException("Not implemented");
  }

  public Optional<Integer> varPosition(VarReference var) {
    throw new RuntimeException("Not implemented");
  }

  public String getName() {
    return String.format("testify_%d", id);
  }

  public Set<Type> instantiatedTypes() {
    return variables.stream().map(VarReference::type).collect(Collectors.toSet());
  }

  public List<VarReference> variables() {
    throw new RuntimeException("Not implemented");
  }

  public VarReference getVar(Statement stmt) {
    throw new RuntimeException("Not implemented");
  }

  public List<VarReference> unconsumedVariablesOfType(Type type) {
    return variables.stream().filter(ref -> ref.type().equals(type) && !ref.isConsumed()).toList();
  }

  public List<VarReference> variablesOfType(Type type) {
    throw new RuntimeException("Not implemented");
  }

  public List<Quartet<VarReference, Callable, Integer, Integer>> availableCallables() {
    throw new RuntimeException("Not implemented");
  }

  public boolean insertRandomStmt() {
    var callable = Rnd.element(hirAnalysis.getCallables());
    System.out.println("Selected callable: " + callable);

    return insertCallable(callable);
  }

  public boolean insertCallable(Callable callable) {
    Set<Generic> generics = callable.getParams().stream().filter(Param::isGeneric)
        .map(p -> p.getType().asGeneric()).collect(Collectors.toCollection(HashSet::new));
    if (callable.isMethod()) {
      generics.addAll(callable.getParent().generics().stream().map(Type::asGeneric)
          .collect(Collectors.toSet()));
    }
    var typeBinding = new TypeBinding(generics);

    generics.stream().map(g -> Pair.with(g, getTypeFor(g))).filter(p -> p.getValue1().isPresent())
        .forEach(p -> typeBinding.bindGeneric(p.getValue0(), p.getValue1().get()));
    if (typeBinding.hasUnboundedGeneric()) {
      System.out.println("Could not bind all generics");
      return false;
    }

    System.out.println("Generics: " + generics);

    var args = callable.getParams().stream()
        .map(p -> {
          if (p.isGeneric()) {
            var genericParam = p.getType().asGeneric();
            var boundType = typeBinding.getBindingFor(genericParam);

            return generateArg(boundType);
          }

          var type = p.getType();
          var innerGenerics = type.generics().stream()
              .filter(Type::isGeneric)
              .map(g -> typeBinding.getBindingFor(g.asGeneric()))
              .toList();
          return generateArg(p);
        })
        .filter(Optional::isPresent)
        .map(Optional::get)
        .toList();

    if (args.size() != callable.getParams().size()) {
      System.out.println("Could not generate all arguments");
      return false;
    }

    VarReference returnValue = null;
    if (callable.returnsValue()) {
      returnValue = createVariable(callable.getReturnType());
    }
    // fn a(x: A, v: Vec<A>)

    var stmt = callable.toStmt(this, args, returnValue);
    statements.add(stmt);
    return true;
  }

  public Optional<VarReference> generateArg(Param param) {
    return generateArg(param, new HashSet<>());
  }

  private Optional<VarReference> generateArg(Param param,
      Set<Type> typesToGenerate) {
    if (param.isPrimitive()) {
      var type = param.getType().asPrimitive();
      return Optional.of(generatePrimitive(type));
    } else if (param.isGeneric()) {
      return generateGeneric(param, typesToGenerate);
    } else {
      var generators = hirAnalysis.generatorsOf(param.getType());
      return generateArgFromGenerators(param.getType(), generators, typesToGenerate);
    }
  }

  /*
   * A convenience method, when we need to generate an arg for a type directly instead of a generic
   * param
   */
  private Optional<VarReference> generateArg(Type type) {
    return generateArg(type, new HashSet<>());
  }

  private Optional<VarReference> generateArg(Type type,
      Set<Type> typesToGenerate) {
    if (type.isPrim()) {
      return Optional.of(generatePrimitive(type.asPrimitive()));
    } else if (type.isComplex()) {
      var generators = hirAnalysis.generatorsOf(type);
      return generateArgFromGenerators(type, generators, typesToGenerate);
    } else {
      throw new RuntimeException("Not implemented");
    }
  }

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
      return Optional.of(Rnd.element(possiblePrimitives));
    } else {
      return Optional.empty();
    }
  }

  private Optional<Type> getComplexTypeFor(Generic generic) {
    var bounds = generic.getBounds();

    var possibleTypes = hirAnalysis.typesImplementing(bounds);
    if (possibleTypes.isEmpty()) {
      return Optional.empty();
    }

    var type = Rnd.element(possibleTypes);
    var boundedGenerics = type.generics().stream().map(g -> {
      var genericType = g.asGeneric();
      var primitive = getPrimitiveTypeFor(genericType);
      if (primitive.isPresent()) {
        return primitive.get();
      } else {
        return getComplexTypeFor(genericType).get();
      }
    }).toList();

    // TODO: 06.12.21 bind generics

    return Optional.of(type);
  }

  private List<Type> bindGenerics(Callable callable, List<VarReference> args) {
    throw new RuntimeException("Not implemented");
  }

  private Optional<VarReference> generateGeneric(Param param,
      Set<Type> typesToGenerate) {
    var genericType = param.getType().asGeneric();
    var primitiveType = getPrimitiveTypeFor(genericType);
    if (primitiveType.isPresent()) {
      // First consider a primitive if it fits
      var val = primitiveType.get().random();
      var var = createVariable(primitiveType.get());
      var stmt = new PrimitiveStmt(var, val);
      statements.add(stmt);
      return Optional.of(var);
    }

    // If not, take a complex
    var complexType = getComplexTypeFor(genericType);

    if (complexType.isPresent()) {
      var generators = hirAnalysis.generatorsOf(complexType.get());
      return generateArgFromGenerators(param.getType(), generators, typesToGenerate);
    } else {
      return Optional.empty();
    }
  }

  private VarReference generatePrimitive(Prim prim) {
    var val = prim.random();
    var var = createVariable(prim);
    var stmt = new PrimitiveStmt(var, val);
    statements.add(stmt);
    return var;
  }

  private Optional<VarReference> generateArgFromGenerators(Type type, List<Callable> generators,
      Set<Type> typesToGenerate) {
    boolean retry = true;
    Callable generator = null;
    while (retry && !generators.isEmpty()) {
      retry = false;

      var candidateGenerator = Rnd.element(generators);
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

    if (generator == null) {
      System.out.println("-- Could not generate: " + type);
      return Optional.empty();
    }

    List<VarReference> args = generator.getParams().stream().map(p -> {
          var usableVars = unconsumedVariablesOfType(p.getType());
          if (!instantiatedTypes().contains(p.getType()) || usableVars.isEmpty()) {
            var extendedTypesToGenerate = new HashSet<>(typesToGenerate);
            extendedTypesToGenerate.add(type);
            return generateArg(p, extendedTypesToGenerate);
          } else {
            // TODO check if those are used
            var var = Rnd.element(usableVars);
            return Optional.of(var);
          }
        })
        .filter(Optional::isPresent)
        .map(Optional::get)
        .toList();
    if (args.size() != generator.getParams().size()) {
      return Optional.empty();
    }

    // TODO bind generics

    VarReference returnValue = null;
    if (generator.returnsValue()) {
      returnValue = createVariable(generator.getReturnType());
    }

    var stmt = generator.toStmt(this, args, returnValue);
    addStmt(stmt);
    return Optional.ofNullable(returnValue);
  }

  private VarReference createVariable(Type type) {
    var var = new VarReference(this, type);
    variables.add(var);
    return var;
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
    throw new RuntimeException("Not implemented");
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
