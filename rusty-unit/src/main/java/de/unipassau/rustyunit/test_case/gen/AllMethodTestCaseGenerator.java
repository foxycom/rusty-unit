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

public class AllMethodTestCaseGenerator implements ChromosomeGenerator<TestCase> {

  private final MirAnalysis<TestCase> mir;

  private final TyCtxt hir;

  private final Mutation<TestCase> mutation;

  private final Crossover<TestCase> crossover;

  private final SeedOptions seedOptions;

  private int current;

  public AllMethodTestCaseGenerator(TyCtxt hir, MirAnalysis<TestCase> mir, Mutation<TestCase> mutation, Crossover<TestCase> crossover, SeedOptions seedOptions) {
    this.mir = mir;
    this.hir = hir;
    this.mutation = mutation;
    this.crossover = crossover;
    this.seedOptions = seedOptions;
    this.current = 0;
  }

  @Override
  public TestCase get() {
    var testCase = new TestCase(TestIdGenerator.get(), hir, mutation, crossover, mir, seedOptions);
    var callable = hir.getCallables().get(current);

    while (testCase.size() <= Constants.CHROMOSOME_LENGTH) {
      testCase.insertCallable(callable);
    }

    return testCase;
  }
}
