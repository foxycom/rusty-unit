package de.unipassau.testify.test_case.operators;

import de.unipassau.testify.Constants;
import de.unipassau.testify.metaheuristics.operators.Mutation;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.visitor.TestCaseVisitor;
import de.unipassau.testify.util.Rnd;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class BasicMutation implements Mutation<TestCase> {

  private static Logger logger = LoggerFactory.getLogger(BasicMutation.class);

  @Override
  public TestCase apply(TestCase testCase) {
    var visitor = new TestCaseVisitor();
    logger.info("Starting mutation on testcase:\n{}\n{}", testCase.visit(visitor), testCase.getTypeBindingsString());
    var copy = testCase.copy();

    if (Rnd.get().nextDouble() <= Constants.P_TEST_DELETE) {
      // delete some statemsnts
      mutationDelete(copy);
    }

    if (Rnd.get().nextDouble() <= Constants.P_TEST_CHANGE) {
      // Change some statements
      mutationChange(copy);
    }

    if (Rnd.get().nextDouble() <= Constants.P_TEST_INSERT) {
      // Insert some statements
      mutationInsert(copy);
    }

    return copy;
  }

  private boolean mutationInsert(TestCase testCase) {
    logger.info("Starting insert mutation");

    boolean changed = true;
    final double alpha = Constants.P_STMT_INSERT;
    int count = 0;

    while (Rnd.get().nextDouble() <= Math.pow(alpha, count)
        && testCase.size() < Constants.CHROMOSOME_LENGTH) {
      count++;

      if (testCase.insertRandomStmt()) {
        changed = true;
      }
    }

    return changed;
  }

  private boolean mutationChange(TestCase testCase) {
    var p = 1d / testCase.size();

    var changed = false;
    for (int position = 0; position < testCase.size(); position++) {
      if (Rnd.get().nextDouble() <= p) {
        var stmt = testCase.stmtAt(position).orElseThrow();
        if (stmt.mutate(testCase)) {
          changed = true;
        }

        position = stmt.position();
      }
    }

    return changed;
  }

  private boolean mutationDelete(TestCase testCase) {
    if (testCase.isEmpty()) {
      return false;
    }

    logger.info("Starting delete mutation");

    boolean changed = false;

    var p = 1d / testCase.size();
    for (int pos = testCase.size() - 1; pos >= 0; pos--) {
      if (pos >= testCase.size()) {
        // In case we removed more than one statement before
        continue;
      }

      logger.info("Deleting statement at position {}", pos);

      if (Rnd.get().nextDouble() <= p) {
        changed |= deleteStatement(testCase, pos);
      }
    }

    return changed;
  }

  private boolean deleteStatement(TestCase testCase, int pos) {
    var stmt = testCase.stmtAt(pos).orElseThrow();
    var var = stmt.returnValue();

    var changed = false;
    if (var.isPresent()) {
      var returnValue = var.get();
      var alternatives = testCase.variablesOfType(returnValue.type(), pos);
      alternatives.remove(returnValue);

      if (!alternatives.isEmpty()) {
        // Replace all usages of var to something else of the same type
        for (int i = pos + 1; i < testCase.size(); i++) {
          var s = testCase.stmtAt(i).orElseThrow();
          if (s.uses(returnValue)) {
            var replacement = Rnd.choice(alternatives);
            logger.info("Replacing {} by {} in statement {}", returnValue, replacement, s);
            s.replace(returnValue, replacement);
            changed = true;
          }
        }
      }
    }

    var removed = testCase.removeStmt(testCase.stmtAt(pos).orElseThrow());
    return changed;
  }
}
