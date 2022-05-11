package de.unipassau.rustyunit.test_case.visitor;

import de.unipassau.rustyunit.test_case.TestCase;
import de.unipassau.rustyunit.test_case.var.VarReference;
import de.unipassau.rustyunit.test_case.statement.Statement;

public interface Visitor {
  String visitTestCase(TestCase testCase);

  String visitStatement(Statement stmt);

  String visitVar(VarReference var);

}
