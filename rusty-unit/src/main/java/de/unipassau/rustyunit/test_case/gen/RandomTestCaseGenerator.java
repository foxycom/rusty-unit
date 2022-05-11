package de.unipassau.rustyunit.test_case.gen;

import de.unipassau.rustyunit.Constants;
import de.unipassau.rustyunit.generators.TestIdGenerator;
import de.unipassau.rustyunit.hir.TyCtxt;
import de.unipassau.rustyunit.metaheuristics.chromosome.ChromosomeGenerator;
import de.unipassau.rustyunit.metaheuristics.operators.Crossover;
import de.unipassau.rustyunit.metaheuristics.operators.Mutation;
import de.unipassau.rustyunit.mir.MirAnalysis;
import de.unipassau.rustyunit.test_case.TestCase;
import de.unipassau.rustyunit.test_case.seed.SeedOptions;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class RandomTestCaseGenerator implements ChromosomeGenerator<TestCase> {
  private static final Logger logger = LoggerFactory.getLogger(RandomTestCaseGenerator.class);

  private final MirAnalysis<TestCase> mir;
  private final TyCtxt hir;
  private final Mutation<TestCase> mutation;
  private final Crossover<TestCase> crossover;

  private final SeedOptions seedOptions;

  public RandomTestCaseGenerator(TyCtxt hir, MirAnalysis<TestCase> mir, Mutation<TestCase> mutation, Crossover<TestCase> crossover) {
    this.hir = hir;
    this.mir = mir;
    this.mutation = mutation;
    this.crossover = crossover;
    this.seedOptions = SeedOptions.builder().build();
  }

  @Override
  public TestCase get() {
    var testCase = new TestCase(TestIdGenerator.get(), hir, mutation, crossover, mir, seedOptions);
    while (testCase.size() < Constants.INITIAL_CHROMOSOME_LENGTH) {
      testCase.insertRandomStmt();
    }

    return testCase;
  }
}
