package de.unipassau.testify.test_case;

import static de.unipassau.testify.Constants.CHROMOSOME_LENGTH;

import de.unipassau.testify.algorithm.MOSA;
import de.unipassau.testify.generators.TestIdGenerator;
import de.unipassau.testify.hir.HirAnalysis;
import de.unipassau.testify.metaheuristics.chromosome.ChromosomeGenerator;
import de.unipassau.testify.metaheuristics.operators.Crossover;
import de.unipassau.testify.metaheuristics.operators.Mutation;
import java.util.concurrent.atomic.AtomicInteger;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class TestCaseGenerator implements ChromosomeGenerator<TestCase> {
  private static final Logger logger = LoggerFactory.getLogger(TestCaseGenerator.class);

  private HirAnalysis hirAnalysis;
  private Mutation<TestCase> mutation;
  private Crossover<TestCase> crossover;


  public TestCaseGenerator(HirAnalysis hirAnalysis, Mutation<TestCase> mutation, Crossover<TestCase> crossover) {
    this.hirAnalysis = hirAnalysis;
    this.mutation = mutation;
    this.crossover = crossover;
  }

  @Override
  public TestCase get() {
    var testCase = new TestCase(TestIdGenerator.get(), hirAnalysis, mutation, crossover);

    while (testCase.size() < 5) {
      testCase.insertRandomStmt();
    }

    return testCase;
  }
}
