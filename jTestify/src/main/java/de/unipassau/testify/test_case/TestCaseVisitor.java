package de.unipassau.testify.test_case;

import com.google.common.collect.Streams;
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

  private String getTypeString(Type type) {
    return type.toString();
  }

  @Override
  public String visitTestCase(TestCase testCase) {
    var sb = new StringBuilder("#[test]\n");
    sb.append("fn ").append(testCase.getName()).append("() {\n");

    for (Statement statement : testCase) {
      sb.append("    ").append(visitStatement(statement)).append("\n");
    }

    sb.append("}");
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

      var argsString = callableStmt.args().stream().map(a -> {
            var argBuilder = new StringBuilder();
            if (callableStmt.borrows(a)) {
              argBuilder.append("&");
            }
            if (callableStmt.mutates(a)) {
              argBuilder.append("mut ");
            }

            argBuilder.append(getVariableName(a));
            return argBuilder.toString();
          })
          .collect(Collectors.joining(", "));

      sb.append(callableStmt.name()).append("(").append(argsString).append(");");
    } else if (stmt.isStructInitStmt()) {
      var structInitStmt = stmt.asStructInitStmt();
      var returnValue = structInitStmt.returnValue().get();
      var actualType = structInitStmt.returnType().get();
      sb.append("let mut ").append(getVariableName(returnValue))
          .append(": ")
          .append(getTypeString(actualType))
          .append(" = ")
          .append(actualType.fullName()).append(" {");

      var argsStr = Streams.zip(structInitStmt.params().stream(), structInitStmt.args().stream(),
              Pair::with)
          .map(pair -> String.format("%s: %s", pair.getValue0().getName(),
              getVariableName(pair.getValue1())))
          .collect(Collectors.joining(", "));
      sb.append(argsStr).append("};");
    } else if (stmt.isEnumStmt()) {
      var enumStmt = stmt.asEnumStmt();
      var returnValue = enumStmt.getReturnValue();
      var actualType = enumStmt.returnType().get();
      var variant = enumStmt.getVariant();
      sb.append("let mut ").append(getVariableName(returnValue))
          .append(": ")
          .append(getTypeString(actualType))
          .append(" = ")
          .append(actualType.fullName())
          .append("::")
          .append(variant.getName());

      if (!enumStmt.getArgs().isEmpty()) {
        var argsStr = enumStmt.getArgs().stream().map(this::getVariableName).collect(Collectors.joining(", "));
        sb.append("(").append(argsStr).append(")");
      }

      sb.append(";");
    } else {
      throw new RuntimeException("Huh?");
    }

    return sb.toString();
  }

  @Override
  public String visitVar(VarReference var) {
    throw new RuntimeException("");
  }
}
