package de.unipassau.testify.test_case;

import de.unipassau.testify.generators.TestIdGenerator;
import de.unipassau.testify.hir.TyCtxt;
import de.unipassau.testify.metaheuristics.chromosome.ChromosomeGenerator;
import de.unipassau.testify.metaheuristics.operators.Crossover;
import de.unipassau.testify.metaheuristics.operators.Mutation;
import de.unipassau.testify.mir.MirAnalysis;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class TestCaseGenerator implements ChromosomeGenerator<TestCase> {
  private static final Logger logger = LoggerFactory.getLogger(TestCaseGenerator.class);

  private final MirAnalysis<TestCase> mir;
  private final TyCtxt hir;
  private final Mutation<TestCase> mutation;
  private final Crossover<TestCase> crossover;
  private final CallableSelector callableSelector;

  public TestCaseGenerator(TyCtxt hir, MirAnalysis<TestCase> mir, Mutation<TestCase> mutation, Crossover<TestCase> crossover, CallableSelector callableSelector) {
    this.hir = hir;
    this.mir = mir;
    this.mutation = mutation;
    this.crossover = crossover;
    this.callableSelector = callableSelector;
  }

  @Override
  public TestCase get() {
    var testCase = new TestCase(TestIdGenerator.get(), hir, mutation, crossover, mir, callableSelector);
    while (testCase.size() < 5) {
      testCase.insertRandomStmt();
    }

    return testCase;
  }
}
