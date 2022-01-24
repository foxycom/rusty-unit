package de.unipassau.testify.test_case.visitor;

import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.statement.Statement;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.TypeBinding;
import java.util.HashMap;
import java.util.Map;

public class TypeBindingStringVisitor implements TypeBindingVisitor {
  private static final String VAR_NAME_PATTERN = "%s_%d";
  private Map<String, Integer> varCounters;
  private Map<VarReference, String> varNames;

  public TypeBindingStringVisitor(TestCase testCase) {
    varCounters = new HashMap<>();
    varNames = new HashMap<>();

    for (Statement statement : testCase) {
      statement.returnValue().ifPresent(this::getVariableName);
    }
  }

  public String getVariableName(VarReference var) {
    if (varNames.containsKey(var)) {
      return varNames.get(var);
    }

    var typeName = var.type().varString();
    String varName;
    if (!varCounters.containsKey(typeName)) {
      varCounters.put(typeName, 0);
      varName = String.format(VAR_NAME_PATTERN, typeName, 0);
    } else {
      var counter = varCounters.compute(typeName, (key, value) -> value + 1);
      varName = String.format(VAR_NAME_PATTERN, typeName, counter);
    }

    varNames.put(var, varName);
    return varName;
  }

  private String getVariableNameRef(VarReference returnValue, VarReference referredValue) {
    if (varNames.containsKey(returnValue)) {
      return varNames.get(returnValue);
    }

    var referredVarName = getVariableName(referredValue);
    var returnVarName = String.format("%s_ref", referredVarName);
    varNames.put(returnValue, returnVarName);
    return returnVarName;
  }

  private String getTypeString(Type type) {
    return type.toString();
  }


  @Override
  public String visit(TypeBinding typeBinding) {
    return typeBinding.toString();
  }
}
