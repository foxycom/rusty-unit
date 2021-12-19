package de.unipassau.testify.test_case.statement;

import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.util.Rnd;
import java.util.List;
import java.util.Optional;
import java.util.UUID;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public interface Statement {

  Logger logger = LoggerFactory.getLogger(Statement.class);

  UUID id();

  Optional<Type> returnType();

  Optional<VarReference> returnValue();

  boolean returnsValue();

  List<VarReference> args();

  void setArgs(List<VarReference> args);

  void setArg(int pos, VarReference var);

  List<Param> params();

  List<Type> actualParamTypes();

  TestCase testCase();

  default boolean isRefStmt() {
    return false;
  }

  default boolean isPrimitiveStmt() {
    return false;
  }

  default boolean isEnumStmt() {
    return false;
  }

  default boolean isStructInitStmt() {
    return false;
  }

  default boolean isCallableStmt() {
    return false;
  }

  default boolean isStaticMethodStmt() {
    return false;
  }

  default boolean isMethodStmt() {
    return false;
  }

  default RefStmt asRefStmt() {
    throw new RuntimeException("Not with me");
  }

  default PrimitiveStmt asPrimitiveStmt() {
    throw new RuntimeException("Not with me");
  }

  default StructInitStmt asStructInitStmt() {
    throw new RuntimeException("Not with me");
  }

  default CallableStmt asCallableStmt() {
    throw new RuntimeException("Not with me");
  }

  default StaticMethodStmt asStaticMethodStmt() {
    throw new RuntimeException("Not with me");
  }

  default MethodStmt asMethodStmt() {
    throw new RuntimeException("Not with me");
  }

  default EnumStmt asEnumStmt() {
    throw new RuntimeException("Not with me");
  }

  default boolean consumes(VarReference var) {
    return false;
  }

  default boolean borrows(VarReference var) {
    return false;
  }

  default boolean mutates(VarReference var) {
    return false;
  }

  default boolean uses(VarReference var) {
    return false;
  }

  default boolean mutate(TestCase testCase) {
    if (params().isEmpty()) {
      return false;
    }

    var pChangeParam = 1d / params().size();
    boolean changed = false;
    for (int iParam = 0; iParam < params().size(); iParam++) {
      if (Rnd.get().nextDouble() < pChangeParam) {
        if (mutateParameter(iParam)) {
          changed = true;
        }
      }
    }

    return changed;
  }

  default int numParamsOfType(Type type) {
    return (int) args().stream().map(VarReference::type).filter(t -> t.equals(type)).count();
  }

  default boolean mutateParameter(int pos) {
    var param = actualParamTypes().get(pos);
    var currentVar = args().get(pos);

    List<VarReference> usableVariables;
    if (param.isRef()) {
      usableVariables = testCase().borrowableVariablesOfType(param, pos);
    } else {
      usableVariables = testCase().consumableVariablesOfType(param, pos);
    }

    usableVariables.remove(returnValue().orElseThrow());
    usableVariables.remove(currentVar);

    int numParamsOfThatType = numParamsOfType(param);
    // If there are fewer objects than parameters of that type,
    // we consider adding an instance
    while (numParamsOfThatType + 1 > usableVariables.size()) {
      logger.info("Still too few usable variables, trying to generate another one");
      var var = testCase().generateArg(param);
      var.ifPresent(usableVariables::add);
    }

    if (usableVariables.isEmpty()) {
      logger.warn("Could not mutate param #{} of type {} ", pos, param);
      return false;
    }

    var replacement = Rnd.choice(usableVariables);
    replaceAt(pos, replacement);

    return true;
  }

  void replace(VarReference oldVar, VarReference newVar);

  default void replaceAt(int pos, VarReference var) {
    var typeBinding = testCase().popTypeBindingsFor(args().get(pos));
    testCase().setTypeBindingsFor(var, typeBinding);

    setArg(pos, var);
  }

  Statement copy(TestCase testCase);

  int position();
}
