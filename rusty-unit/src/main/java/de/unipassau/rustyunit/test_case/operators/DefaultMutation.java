package de.unipassau.rustyunit.test_case.operators;

import de.unipassau.rustyunit.Constants;
import de.unipassau.rustyunit.metaheuristics.operators.Mutation;
import de.unipassau.rustyunit.test_case.TestCase;
import de.unipassau.rustyunit.util.Rnd;
import java.util.ArrayList;
import java.util.stream.Collectors;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class DefaultMutation implements Mutation<TestCase> {
  private static final Logger logger = LoggerFactory.getLogger(DefaultMutation.class);

  @Override
  public TestCase apply(TestCase testCase) {
    var copy = testCase.copy();
    logger.info("Starting mutation on testcase from {}:\n{}\n{}",  testCase.getId(), copy,
        testCase.getTypeBindingsString());

    if (Rnd.get().nextDouble() <= Constants.P_TEST_DELETE) {
      // delete some statemsnts
      mutationDelete(copy);
    }

    logger.info("Test case IR:\n{}\n", testCase);

    if (Rnd.get().nextDouble() <= Constants.P_TEST_CHANGE) {
      // Change some statements
      mutationChange(copy);
    }

    logger.info("Test case IR:\n{}\n", testCase);

    if (Rnd.get().nextDouble() <= Constants.P_TEST_INSERT) {
      // Insert some statements
      mutationInsert(copy);
    }

    logger.info("Mutated test:\n{}", copy);
    return copy;
  }

  private boolean mutationInsert(TestCase testCase) {
    logger.info("Starting insert mutation");

    boolean changed = false;
    final double alpha = Constants.P_STMT_INSERT;
    int count = 0;

    while (Rnd.get().nextDouble() <= Math.pow(alpha, count)
        && testCase.size() < Constants.CHROMOSOME_LENGTH) {
      count++;

      if (testCase.insertRandomStmt().isPresent()) {
        changed = true;
      }
    }

    logger.debug("Inserted {} statements", count);
    return changed;
  }

  private boolean mutationChange(TestCase testCase) {
    logger.info("Starting change mutation");
    var p = 1d / testCase.size();

    int count = 0;
    var changed = false;
    for (int position = 0; position < testCase.size(); position++) {
      if (Rnd.get().nextDouble() <= p) {
        logger.info("Mutating statement at position {}", position);
        var stmt = testCase.stmtAt(position).orElseThrow();
        if (stmt.mutate(testCase)) {
          count++;
          changed = true;
        }

        position = stmt.position();
      }
    }

    logger.debug("Changed " + count + " statements");
    return changed;
  }

  private boolean mutationDelete(TestCase testCase) {
    logger.info("Starting delete mutation");
    if (testCase.isEmpty()) {
      logger.debug("Aborting, test case is already empty");
      return false;
    }

    boolean changed = false;

    var p = 1d / testCase.size();
    for (int pos = testCase.size() - 1; pos >= 0; pos--) {
      if (pos >= testCase.size()) {
        // In case we removed more than one statement before
        continue;
      }

      if (Rnd.get().nextDouble() <= p) {
        logger.info("Deleting statement at position {}", pos);
        changed |= deleteStatement(testCase, pos);
      }
    }

    var message = changed ? "Deleted statements" : "Did not delete any statement";
    logger.debug(message);
    return changed;
  }

  private boolean deleteStatement(TestCase testCase, int pos) {
    var stmt = testCase.stmtAt(pos).orElseThrow();
    var var = stmt.returnValue();

    var changed = false;
    if (var.isPresent()) {
      var returnValue = var.get();
      var alternatives = testCase.variablesOfType(returnValue.type(), pos)
          .stream().filter(a -> {
            if (stmt.borrows(var.get())) {
              return a.isBorrowableAt(pos);
            } else {
              return a.isConsumableAt(pos);
            }
          }).collect(Collectors.toCollection(ArrayList::new));
      alternatives.remove(returnValue);

      if (!alternatives.isEmpty()) {
        for (int i = pos + 1; i < testCase.size(); i++) {
          var s = testCase.stmtAt(i).orElseThrow();
          if (s.uses(returnValue)) {
            // Replace all usages of var to something else of the same type
            var replacement = Rnd.choice(alternatives);
            logger.info("Replacing {} by {} at {} in statement {}",
                returnValue, replacement, i, s);
            s.replace(returnValue, replacement);
          }

          changed = true;
        }
      }
    }

    var removed = testCase.removeStmt(testCase.stmtAt(pos).orElseThrow());

    return changed;
  }
}
