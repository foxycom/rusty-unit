package de.unipassau.testify.test_case;

import de.unipassau.testify.test_case.statement.Statement;

public interface Visitor {
  String visitTestCase(TestCase testCase);

  String visitStatement(Statement stmt);

  String visitVar(VarReference var);

}
