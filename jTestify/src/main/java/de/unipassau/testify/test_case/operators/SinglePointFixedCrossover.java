package de.unipassau.testify.test_case.operators;

import de.unipassau.testify.exception.NoAvailableArgException;
import de.unipassau.testify.metaheuristics.operators.Crossover;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.visitor.CrossoverDebugVisitor;
import de.unipassau.testify.test_case.visitor.TestCaseVisitor;
import de.unipassau.testify.util.Rnd;
import java.util.Objects;
import java.util.Optional;
import org.javatuples.Pair;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class SinglePointFixedCrossover implements Crossover<TestCase> {

  private static final Logger logger = LoggerFactory.getLogger(SinglePointFixedCrossover.class);

  @Override
  public Pair<TestCase, TestCase> apply(TestCase parent1, TestCase parent2) {
    if (parent1.size() < 2 || parent2.size() < 2) {
      return Pair.with(parent1, parent2);
    }

    int point = Rnd.get().nextInt(Math.min(parent1.size(), parent2.size()) - 1) + 1;

    var debugVisitor = new CrossoverDebugVisitor(point);

    logger.info("Starting crossover at point = {}", point);
    logger.info("Parent 1:\n{}", parent1.visit(debugVisitor));
    logger.info("Parent 2:\n{}", parent2.visit(debugVisitor));

    // If a crossover fails then take the  first parent again
    var child1 = crossOver(parent1, parent2, point).orElse(parent1);
    var child2 = crossOver(parent2, parent1, point).orElse(parent2);

    logger.info("Child 1:\n{}", child1.visit(debugVisitor));
    logger.info("Child 2:\n{}", child2.visit(debugVisitor));

    return Pair.with(child1, child2);
  }

  Optional<TestCase> crossOver(TestCase t1, TestCase t2, int pos) {
    var offspring = t1.copy();
    offspring.removeAllStmts();

    for (int i = 0; i < pos; i++) {
      offspring.appendStmt(t1.stmtAt(i).orElseThrow().copy(offspring));
    }

    // When composing two test cases, there might be cases when two private methods from
    // different files collide. That is, parent 1 uses a private method from file A (and thus,
    // the test must be defined in file A), but parent 2 uses another private method from file B.
    // In such case, we try to throw away the problematic statement from parent 2. If it's
    // return value won't be used in any place, then we can just skip it, otherwise, we have to
    // abort the crossover.
    int i = pos;
    while (i < t2.size()) {
      var stmt = t2.stmtAt(i).orElseThrow();
      var variables = offspring.satisfyParameters(offspring.size() - 1, stmt);
      var newStmt = stmt.copy(offspring);
      if (variables.size() != newStmt.params().size()) {
        // We could not generate all arguments
        if (stmt.returnValue().isPresent()) {
          var returnValue = stmt.returnValue().get();
          if (returnValue.usedAt().isEmpty()) {
            // The return value won't be used at any point, so skip the problematic stmt
            logger.debug("Skipping statement");
            i++;
            continue;
          } else {
            // The return value will be used later, so the whole test case must be thrown away
            logger.debug("Unusable test, throwing away");
            return Optional.empty();
          }
        } else {
          // Just skip the statement since it does not return anything that can be used later
          logger.debug("Skipping statement");
          i++;
          continue;
        }
      }

      if (offspring.getFilePathBinding().isPresent()) {
        if (newStmt.getSrcFilePath() != null && !Objects.equals(newStmt.getSrcFilePath(),
            offspring.getFilePathBinding().get())) {
          if (stmt.returnValue().isPresent()) {
            var returnValue = stmt.returnValue().get();
            if (returnValue.usedAt().isEmpty()) {
              // The return value won't be used at any point, so skip the problematic stmt
              logger.debug("Skipping statement");
              i++;
              continue;
            } else {
              // The return value will be used later, so the whole test case must be thrown away
              logger.debug("Unusable test, throwing away");
              return Optional.empty();
            }
          } else {
            // Just skip the statement since it does not return anything that can be used later
            logger.debug("Skipping statement");
            i++;
            continue;
          }
        }
      }

      newStmt.setArgs(variables);
      offspring.appendStmt(newStmt);
      i++;
    }


    // TODO update type bindings

    return Optional.of(offspring);
  }
}
