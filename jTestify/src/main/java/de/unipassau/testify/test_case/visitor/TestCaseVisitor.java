package de.unipassau.testify.test_case.visitor;

import com.google.common.collect.Streams;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.statement.Statement;
import de.unipassau.testify.test_case.type.Type;
import java.util.HashMap;
import java.util.Map;
import java.util.stream.Collectors;
import org.javatuples.Pair;

public class TestCaseVisitor implements Visitor {

  private static final String VAR_NAME_PATTERN = "%s_%d";
  private Map<String, Integer> varCounters;
  private Map<VarReference, String> varNames;

  public TestCaseVisitor() {
    varCounters = new HashMap<>();
    varNames = new HashMap<>();
  }

  private String getVariableName(VarReference var) {
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
  public String visitTestCase(TestCase testCase) {
    if (testCase.getId() == 13) {
      System.out.println();
    }
    var sb = new StringBuilder("#[test]\n");
    sb.append("fn ").append(testCase.getName()).append("() {\n");

    sb.append("    rusty_monitor::set_test_id(").append(testCase.getId()).append(");\n");

    for (Statement statement : testCase) {
      sb.append("    ").append(visitStatement(statement)).append("\n");
    }

    sb.append("}");

    clear();
    return sb.toString();
  }

  @Override
  public String visitStatement(Statement stmt) {
    var sb = new StringBuilder();
    if (stmt.isPrimitiveStmt()) {
      var primitiveStmt = stmt.asPrimitiveStmt();
      var returnValue = primitiveStmt.returnValue().get();
      sb.append("let mut ")
          .append(getVariableName(returnValue))
          .append(": ")
          .append(getTypeString(returnValue.type()))
          .append(" = ")
          .append(primitiveStmt.getValue())
          .append(";");
    } else if (stmt.isCallableStmt()) {
      var callableStmt = stmt.asCallableStmt();
      if (callableStmt.returnsValue()) {
        var returnValue = callableStmt.returnValue().get();
        var actualType = returnValue.type();
        sb.append("let mut ").append(getVariableName(returnValue))
            .append(": ")
            .append(getTypeString(actualType))
            .append(" = ");
      }

      if (callableStmt.parent().isPresent()) {
        var parentType = callableStmt.parent().get();
        sb.append(parentType.fullName()).append("::");
      }

      var argsString = callableStmt.args()
          .stream()
          .map(this::getVariableName)
          .collect(Collectors.joining(", "));

      sb.append(callableStmt.name()).append("(").append(argsString).append(");");
    } else if (stmt.isStructInitStmt()) {
      var structInitStmt = stmt.asStructInitStmt();
      var returnValue = structInitStmt.returnValue().get();
      var returnType = returnValue.type();
      sb.append("let mut ").append(getVariableName(returnValue))
          .append(": ")
          .append(getTypeString(returnType))
          .append(" = ")
          .append(returnType.fullName()).append(" {");

      var argsStr = Streams.zip(structInitStmt.params().stream(), structInitStmt.args().stream(),
              Pair::with)
          .map(pair -> {
            var value = getVariableName(pair.getValue1());
            /*if (structInitStmt.borrows(pair.getValue1())) {
              value = String.format("&%s", value);
            }*/

            return String.format("%s: %s", pair.getValue0().getName(), value);
          })
          .collect(Collectors.joining(", "));
      sb.append(argsStr).append("};");
    } else if (stmt.isEnumStmt()) {
      var enumStmt = stmt.asEnumStmt();
      var returnValue = enumStmt.getReturnValue();
      var returnType = returnValue.type();
      var variant = enumStmt.getVariant();
      sb.append("let mut ").append(getVariableName(returnValue))
          .append(": ")
          .append(getTypeString(returnType))
          .append(" = ")
          .append(returnType.fullName())
          .append("::")
          .append(variant.getName());

      if (!enumStmt.getArgs().isEmpty()) {
        /*if (enumStmt.borrows(a)) {
                value = String.format("&%s", value);
              }*/
        var argsStr = enumStmt.getArgs().stream()
            .map(this::getVariableName)
            .collect(Collectors.joining(", "));
        sb.append("(").append(argsStr).append(")");
      }

      sb.append(";");
    } else if (stmt.isRefStmt()) {
      var refStmt = stmt.asRefStmt();
      var returnValue = refStmt.returnValue().get();
      var returnType = returnValue.type();
      sb.append("let mut ").append(getVariableNameRef(returnValue, refStmt.arg()))
          .append(": ")
          .append(getTypeString(returnType))
          .append(" = &mut ")
          .append(getVariableName(refStmt.arg()))
          .append(";");
    } else if (stmt.isTupleStmt()) {
      var tupleStmt = stmt.asTupleStmt();
      var returnValue = tupleStmt.returnValue().get();
      var returnType = returnValue.type();
      var args = tupleStmt.args().stream().map(this::getVariableName)
          .collect(Collectors.joining(", "));
      sb.append("let mut ").append(getVariableName(returnValue))
          .append(": ")
          .append(getTypeString(returnType))
          .append(" = (")
          .append(args)
          .append(");");
    } else if (stmt.isArrayStmt()) {
      var arrayStmt = stmt.asArrayStmt();
      var returnValue = arrayStmt.returnValue().get();
      var returnType = returnValue.type();
      var args = arrayStmt.args().stream().map(this::getVariableName)
          .collect(Collectors.joining(", "));
      sb.append("let mut ").append(getVariableName(returnValue))
          .append(": ")
          .append(getTypeString(returnType))
          .append(" = [")
          .append(args)
          .append("];");
    } else {
      throw new RuntimeException("Huh?");
    }

    return sb.toString();
  }

  public void clear() {
    varCounters.clear();
    varNames.clear();
  }

  @Override
  public String visitVar(VarReference var) {
    throw new RuntimeException("");
  }
}
