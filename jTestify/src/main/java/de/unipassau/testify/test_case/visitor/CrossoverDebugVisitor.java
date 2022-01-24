package de.unipassau.testify.test_case.visitor;

import de.unipassau.testify.test_case.TestCase;
import java.util.ArrayList;
import java.util.Collections;

public class CrossoverDebugVisitor extends TestCaseVisitor {

  private int cutPoint;

  public CrossoverDebugVisitor(int cutPoint) {
    this.cutPoint = cutPoint;
  }

  @Override
  public String visitTestCase(TestCase testCase) {
    var sb = new StringBuilder("#[test]\n");
    sb.append("fn ").append(testCase.getName()).append("() {\n");

    var statements = new ArrayList<String>(testCase.size());
    for (var statement : testCase) {
      statements.add(visitStatement(statement));
    }

    int maxLength = statements.stream().map(String::length).reduce(Integer::max).orElse(0);

    for (int i = 0; i < statements.size(); i++) {
      if (i == cutPoint) {
        sb.append("    ").append(String.join("", Collections.nCopies(maxLength, "-"))).append("\n");
      }

      sb.append("    ").append(statements.get(i)).append("\n");
    }

    sb.append("}");
    clear();
    return sb.toString();
  }

}
