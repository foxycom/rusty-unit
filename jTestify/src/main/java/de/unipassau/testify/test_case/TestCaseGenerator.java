package de.unipassau.testify.test_case;

import de.unipassau.testify.hir.HirAnalysis;
import de.unipassau.testify.metaheuristics.chromosome.ChromosomeGenerator;
import de.unipassau.testify.metaheuristics.operators.Crossover;
import de.unipassau.testify.metaheuristics.operators.Mutation;
import java.util.concurrent.atomic.AtomicInteger;

public class TestCaseGenerator implements ChromosomeGenerator<TestCase> {

  private AtomicInteger id;
  private HirAnalysis hirAnalysis;
  private Mutation<TestCase> mutation;
  private Crossover<TestCase> crossover;


  public TestCaseGenerator(HirAnalysis hirAnalysis, Mutation<TestCase> mutation, Crossover<TestCase> crossover) {
    this.hirAnalysis = hirAnalysis;
    this.id = new AtomicInteger(0);
    this.mutation = mutation;
    this.crossover = crossover;
  }

  @Override
  public TestCase get() {
    var testCase = new TestCase(id.getAndIncrement(), hirAnalysis, mutation, crossover);
    while (testCase.size() < 5) {
      testCase.insertRandomStmt();
    }

    return testCase;
  }
}
