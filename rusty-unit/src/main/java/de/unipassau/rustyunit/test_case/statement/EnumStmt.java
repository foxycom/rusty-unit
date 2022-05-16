package de.unipassau.rustyunit.test_case.statement;

import static java.util.stream.Collectors.toCollection;

import de.unipassau.rustyunit.test_case.Param;
import de.unipassau.rustyunit.test_case.TestCase;
import de.unipassau.rustyunit.test_case.var.VarReference;
import de.unipassau.rustyunit.test_case.callable.Callable;
import de.unipassau.rustyunit.test_case.callable.EnumInit;
import de.unipassau.rustyunit.type.AbstractEnum.EnumVariant;
import de.unipassau.rustyunit.type.Enum;
import de.unipassau.rustyunit.type.Ref;
import de.unipassau.rustyunit.type.Type;
import de.unipassau.rustyunit.util.Rnd;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.UUID;
import java.util.stream.Collectors;
import java.util.stream.IntStream;

public class EnumStmt implements Statement {

  private final UUID id;
  private VarReference returnValue;
  private List<VarReference> args;
  private TestCase testCase;
  private EnumInit enumInit;

  public EnumStmt(TestCase testCase, List<VarReference> args, VarReference returnValue,
      EnumInit enumInit) {
    this.id = UUID.randomUUID();
    this.enumInit = enumInit;
    this.testCase = testCase;
    this.returnValue = returnValue;
    this.args = args;
  }

  public VarReference getReturnValue() {
    return returnValue;
  }

  public Enum getType() {
    return enumInit.getReturnType().asEnum();
  }

  public EnumVariant getVariant() {
    return enumInit.getVariant();
  }

  public List<VarReference> getArgs() {
    return args;
  }

  public EnumInit getEnumInit() {
    return enumInit;
  }

  @Override
  public UUID id() {
    return id;
  }

  @Override
  public Optional<Type> returnType() {
    return Optional.of(enumInit.getReturnType());
  }

  @Override
  public Optional<VarReference> returnValue() {
    return Optional.of(returnValue);
  }

  @Override
  public boolean returnsValue() {
    return true;
  }

  @Override
  public List<VarReference> args() {
    return args;
  }

  @Override
  public void setArgs(List<VarReference> args) {
    if (args.size() != params().size()) {
      throw new RuntimeException("Unequal number of args and params");
    }

    this.args = args;
  }

  @Override
  public void setArg(int pos, VarReference var) {
    args.set(pos, var);
  }

  @Override
  public List<Param> params() {
    return enumInit.getVariant().getParams();
  }

  @Override
  public List<Type> actualParamTypes() {
    return args.stream().peek(Objects::requireNonNull).map(VarReference::type).toList();
  }

  @Override
  public TestCase testCase() {
    return testCase;
  }

  @Override
  public String getSrcFilePath() {
    return enumInit.getSrcFilePath();
  }

  @Override
  public boolean isPublic() {
    return enumInit.isPublic();
  }

  @Override
  public boolean isEnumStmt() {
    return true;
  }

  @Override
  public EnumStmt asEnumStmt() {
    return this;
  }

  @Override
  public Callable getCallable() {
    return enumInit;
  }

  @Override
  public boolean consumes(VarReference var) {
    if (var.type().isRef()) {
      var stmt = var.definedBy();
      if (stmt.isRefStmt()) {
        var referencedVar = var.definedBy().asRefStmt().arg();
        return args.contains(referencedVar);
      } else if (stmt.isTupleStmt() || stmt.isArrayStmt()) {
        throw new RuntimeException("Not implemented");
      } else {
        return false;
      }
    } else {
      return args.contains(var);
    }
  }

  @Override
  public boolean borrows(VarReference var) {
    if (var.type().isRef()) {
      return args.contains(var);
    } else {
      var referencedVars = args.stream().filter(a -> a.type().isRef())
          .map(v -> {
            var s = v.definedBy();
            return s.asRefStmt().arg();
          }).toList();
      return referencedVars.contains(var);
    }
  }

  @Override
  public boolean mutates(VarReference var) {
    throw new RuntimeException("mutates is not implemented");
  }

  @Override
  public boolean uses(VarReference var) {
    return args.stream().anyMatch(a -> a.equals(var));
  }

  @Override
  public boolean mutate(TestCase testCase) {
    var variants = testCase.getHirAnalysis()
        .generatorsOf(enumInit.getReturnType(), enumInit.getSrcFilePath(), EnumInit.class);
    var mutatedEnumInit = Rnd.choice(variants);

    if (mutatedEnumInit.getParams().isEmpty()) {
      args.clear();
      var changed = !mutatedEnumInit.equals(enumInit);
      enumInit = (EnumInit) mutatedEnumInit;
      return changed;
    } else if (mutatedEnumInit.equals(enumInit)) {
      var pChangeParam = 1d / params().size();
      var changed = false;
      for (int iParam = 0; iParam < params().size(); iParam++) {
        if (Rnd.get().nextDouble() < pChangeParam) {
          if (mutateParameter(iParam)) {
            changed = true;
          }
        }
      }

      return changed;
    } else {
      boolean changed = false;

      List<VarReference> args = new ArrayList<>(mutatedEnumInit.getParams().size());
      for (int iParam = 0; iParam < mutatedEnumInit.getParams().size(); iParam++) {
        var param = mutatedEnumInit.getParams().get(iParam);
        var boundedParam = param.bindGenerics(returnValue.getBinding());
        var newVariable = testCase.getArg(boundedParam.getType(), position());
        newVariable.ifPresent(args::add);
      }

      if (args.size() == mutatedEnumInit.getParams().size()) {
        enumInit = (EnumInit) mutatedEnumInit;

        setArgs(args);
        changed = true;
      }

      return changed;
    }
  }

  @Override
  public void replace(VarReference oldVar, VarReference newVar) {
    if (!args.contains(oldVar)) {
      throw new RuntimeException("There's something wrong");
    }

    args.replaceAll(a -> a.equals(oldVar) ? newVar : a);
  }

  @Override
  public void replaceAt(int pos, VarReference var) {
    args.set(pos, var);
  }

  @Override
  public Statement copy(TestCase testCase) {
    var argsCopy = args.stream()
        .map(a -> a.copy(testCase))
        .collect(toCollection(ArrayList::new));
    var returnValueCopy = returnValue.copy(testCase);
    return new EnumStmt(testCase, argsCopy, returnValueCopy, enumInit);
  }

  @Override
  public int position() {
    return testCase.stmtPosition(this).orElseThrow();
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof EnumStmt)) {
      return false;
    }
    EnumStmt enumStmt = (EnumStmt) o;
    return id.equals(enumStmt.id) && Objects.equals(returnValue, enumStmt.returnValue)
        && args.equals(enumStmt.args) && enumInit.equals(enumStmt.enumInit);
  }

  @Override
  public int hashCode() {
    return Objects.hash(id, returnValue, args, enumInit);
  }
}
