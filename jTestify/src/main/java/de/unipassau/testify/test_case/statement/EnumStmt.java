package de.unipassau.testify.test_case.statement;

import static java.util.stream.Collectors.toCollection;

import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.callable.EnumInit;
import de.unipassau.testify.test_case.type.Enum;
import de.unipassau.testify.test_case.type.Enum.EnumVariant;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.util.Rnd;
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
  public boolean consumes(VarReference var) {
    var typeBinding = testCase.getTypeBindingsFor(returnValue);

    var pos = IntStream.range(0, args.size()).filter(i -> args.get(i).equals(var)).findFirst();
    if (pos.isPresent()) {
      return !getVariant().getParams().get(pos.getAsInt()).bindGenerics(typeBinding)
          .isByReference();
    } else {
      return false;
    }
  }

  @Override
  public boolean borrows(VarReference var) {
    var typeBinding = testCase.getTypeBindingsFor(returnValue);

    var pos = IntStream.range(0, args.size()).filter(i -> args.get(i).equals(var)).findFirst();
    if (pos.isPresent()) {
      return getVariant().getParams().get(pos.getAsInt()).bindGenerics(typeBinding).isByReference();
    } else {
      return false;
    }
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

    if (params().isEmpty()) {
      args.clear();
      var changed = !mutatedEnumInit.equals(enumInit);
      enumInit = (EnumInit) mutatedEnumInit;
      return changed;
    }

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
  }

  @Override
  public void replace(VarReference oldVar, VarReference newVar) {
    if (!args.contains(oldVar)) {
      throw new RuntimeException("There's something wrong");
    }

    /*var typeBinding = testCase.popTypeBindingsFor(oldVar);
    testCase.setTypeBindingsFor(newVar, typeBinding);*/

    args = args.stream().map(a -> {
      if (a.equals(oldVar)) {
        return newVar;
      } else {
        return a;
      }
    }).toList();
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
}
