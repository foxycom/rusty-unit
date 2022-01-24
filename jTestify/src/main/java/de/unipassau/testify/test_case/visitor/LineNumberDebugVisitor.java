package de.unipassau.testify.test_case.visitor;

import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.statement.Statement;
import java.util.ArrayList;
import java.util.Collections;

public class LineNumberDebugVisitor extends TestCaseVisitor {

  @Override
  public String visitTestCase(TestCase testCase) {
    var sb = new StringBuilder("#[test]\n");
    sb.append("fn ").append(testCase.getName()).append("() {\n");

    var statements = new ArrayList<String>(testCase.size());
    for (Statement statement : testCase) {
      statements.add("    " + visitStatement(statement));
    }

    int maxLength = statements.stream().map(String::length).reduce(Integer::max).orElse(0);

    for (int i = 0; i < statements.size(); i++) {
      int offset = maxLength - statements.get(i).length() + 1;
      sb.append(statements.get(i))
          .append(String.join("", Collections.nCopies(offset, " ")))
          .append("// ").append(i).append("\n");
    }

    sb.append("}");

    clear();
    return sb.toString();
  }
}
