package de.unipassau.testify.test_case.operators;

import de.unipassau.testify.exception.NoAvailableArgException;
import de.unipassau.testify.metaheuristics.operators.Crossover;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.TestCaseVisitor;
import de.unipassau.testify.test_case.statement.Statement;
import de.unipassau.testify.util.Rnd;
import java.util.List;
import org.javatuples.Pair;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class SinglePointFixedCrossover implements Crossover<TestCase> {
  private static Logger logger = LoggerFactory.getLogger(SinglePointFixedCrossover.class);

  @Override
  public Pair<TestCase, TestCase> apply(TestCase parent1, TestCase parent2) {
    if (parent1.size() < 2 || parent2.size() < 2) {
      return Pair.with(parent1, parent2);
    }

    var visitor = new TestCaseVisitor();


    int point = Rnd.get().nextInt(Math.min(parent1.size(), parent2.size()) - 1) + 1;

    logger.info("Starting crossover at point = {}", point);
    logger.info("Parent 1:\n{}", parent1.visit(visitor));
    logger.info("Parent 2:\n{}", parent2.visit(visitor));

    var child1 = crossOver(parent1, parent2, point);
    var child2 = crossOver(parent2, parent1, point);

    logger.info("Child 1:\n{}", child1.visit(visitor));
    logger.info("Child 2:\n{}", child2.visit(visitor));

    return Pair.with(child1, child2);
  }

  private TestCase crossOver(TestCase t1, TestCase t2, int pos) {
    var offspring = t1.copy();
    offspring.removeAllStmts();

    for (int i = 0; i < pos; i++) {
      offspring.appendStmt(t1.stmtAt(i).orElseThrow().copy(offspring));
    }

    for (int i = pos; i < t2.size(); i++) {
      var stmt = t2.stmtAt(i).orElseThrow();
      try {
        var variables = offspring.satisfyParameters(i, stmt);
        var newStmt = stmt.copy(offspring);
        newStmt.setArgs(variables);
        offspring.appendStmt(newStmt);
      } catch (NoAvailableArgException e) {
        throw new RuntimeException("Could not satisfy an argument for some reason", e);
      }
    }

    // TODO update type bindings

    return offspring;
  }
}
