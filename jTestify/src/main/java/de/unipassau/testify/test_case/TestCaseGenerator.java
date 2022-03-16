package de.unipassau.testify.test_case;

import de.unipassau.testify.generators.TestIdGenerator;
import de.unipassau.testify.hir.TyCtxt;
import de.unipassau.testify.metaheuristics.chromosome.ChromosomeGenerator;
import de.unipassau.testify.metaheuristics.operators.Crossover;
import de.unipassau.testify.metaheuristics.operators.Mutation;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class TestCaseGenerator implements ChromosomeGenerator<TestCase> {
  private static final Logger logger = LoggerFactory.getLogger(TestCaseGenerator.class);

  private TyCtxt hir;
  private Mutation<TestCase> mutation;
  private Crossover<TestCase> crossover;


  public TestCaseGenerator(TyCtxt hir, Mutation<TestCase> mutation, Crossover<TestCase> crossover) {
    this.hir = hir;
    this.mutation = mutation;
    this.crossover = crossover;
  }

  @Override
  public TestCase get() {
    var testCase = new TestCase(TestIdGenerator.get(), hir, mutation, crossover);
    while (testCase.size() < 5) {
      testCase.insertRandomStmt();
    }

    return testCase;
  }
}
