package de.unipassau.testify.test_case.gen;

import com.google.common.base.Preconditions;
import de.unipassau.testify.Constants;
import de.unipassau.testify.generators.TestIdGenerator;
import de.unipassau.testify.hir.TyCtxt;
import de.unipassau.testify.metaheuristics.chromosome.ChromosomeGenerator;
import de.unipassau.testify.metaheuristics.operators.Crossover;
import de.unipassau.testify.metaheuristics.operators.Mutation;
import de.unipassau.testify.mir.MirAnalysis;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.seed.SeedOptions;
import de.unipassau.testify.test_case.var.VarReference;
import de.unipassau.testify.util.Rnd;
import java.util.function.Predicate;

public class SeededTestCaseGenerator implements ChromosomeGenerator<TestCase> {

  private final MirAnalysis<TestCase> mir;

  private final TyCtxt hir;

  private final Mutation<TestCase> mutation;

  private final Crossover<TestCase> crossover;

  private final SeedOptions seedOptions;

  public SeededTestCaseGenerator(TyCtxt hir, MirAnalysis<TestCase> mir,
      Mutation<TestCase> mutation, Crossover<TestCase> crossover,
      SeedOptions seedOptions) {
    this.mir = mir;
    this.hir = hir;
    this.mutation = mutation;
    this.crossover = crossover;
    this.seedOptions = seedOptions;

    Preconditions.checkState(!hir.getCallables().isEmpty());
  }

  @Override
  public TestCase get() {
    var testCase = new TestCase(TestIdGenerator.get(), hir, mutation, crossover, mir,
        seedOptions);
    var callable = Rnd.choice(hir.getCallables());
    testCase.insertCallable(callable);

    while (testCase.size() < Constants.CHROMOSOME_LENGTH) {
      var variables = testCase.variables();
      var interestingVariables = variables.stream().filter(filterInterestingVars()).toList();
      if (interestingVariables.isEmpty()) {
        throw new RuntimeException("Not implemented");
      }

      var selectedVariable = Rnd.choice(interestingVariables);
      var interestingCallables = hir.callablesWithParam(selectedVariable.type(),
          testCase.getFilePathBinding().orElse(null), selectedVariable.isConsumed());
      if (interestingCallables.isEmpty()) {
        testCase.insertCallable(Rnd.choice(hir.getCallables()));
      } else {
        var interestingCallable = Rnd.choice(interestingCallables);
        testCase.insertCallable(interestingCallable);
      }
    }
    return testCase;
  }

  private Predicate<VarReference> filterInterestingVars() {
    return v -> {
      if (v.type().isRef() && !v.type().asRef().getInnerType().isPrim()) {
        return true;
      }

      return v.type().isEnum() || v.type().isStruct();
    };
  }
}
