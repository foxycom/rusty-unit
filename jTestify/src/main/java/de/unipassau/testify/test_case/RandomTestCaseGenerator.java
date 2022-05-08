package de.unipassau.testify.test_case;

import de.unipassau.testify.Constants;
import de.unipassau.testify.generators.TestIdGenerator;
import de.unipassau.testify.hir.TyCtxt;
import de.unipassau.testify.metaheuristics.chromosome.ChromosomeGenerator;
import de.unipassau.testify.metaheuristics.operators.Crossover;
import de.unipassau.testify.metaheuristics.operators.Mutation;
import de.unipassau.testify.mir.MirAnalysis;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class RandomTestCaseGenerator implements ChromosomeGenerator<TestCase> {
  private static final Logger logger = LoggerFactory.getLogger(RandomTestCaseGenerator.class);

  private final MirAnalysis<TestCase> mir;
  private final TyCtxt hir;
  private final Mutation<TestCase> mutation;
  private final Crossover<TestCase> crossover;
  private final CallableSelector callableSelector;

  public RandomTestCaseGenerator(TyCtxt hir, MirAnalysis<TestCase> mir, Mutation<TestCase> mutation, Crossover<TestCase> crossover, CallableSelector callableSelector) {
    this.hir = hir;
    this.mir = mir;
    this.mutation = mutation;
    this.crossover = crossover;
    this.callableSelector = callableSelector;
  }

  @Override
  public TestCase get() {
    var testCase = new TestCase(TestIdGenerator.get(), hir, mutation, crossover, mir, callableSelector);
    while (testCase.size() < Constants.INITIAL_CHROMOSOME_LENGTH) {
      testCase.insertRandomStmt();
    }

    return testCase;
  }
}
