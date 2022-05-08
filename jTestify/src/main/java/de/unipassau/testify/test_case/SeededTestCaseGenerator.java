package de.unipassau.testify.test_case;

import com.google.common.base.Preconditions;
import de.unipassau.testify.Constants;
import de.unipassau.testify.generators.TestIdGenerator;
import de.unipassau.testify.hir.TyCtxt;
import de.unipassau.testify.metaheuristics.chromosome.ChromosomeGenerator;
import de.unipassau.testify.metaheuristics.operators.Crossover;
import de.unipassau.testify.metaheuristics.operators.Mutation;
import de.unipassau.testify.mir.MirAnalysis;

public class SeededTestCaseGenerator implements ChromosomeGenerator<TestCase> {

    private final MirAnalysis<TestCase> mir;

    private final TyCtxt hir;

    private final Mutation<TestCase> mutation;

    private final Crossover<TestCase> crossover;

    private final CallableSelector callableSelector;

    private int current;

    public SeededTestCaseGenerator(TyCtxt hir, MirAnalysis<TestCase> mir,
          Mutation<TestCase> mutation, Crossover<TestCase> crossover,
          CallableSelector callableSelector) {
        this.mir = mir;
        this.hir = hir;
        this.mutation = mutation;
        this.crossover = crossover;
        this.callableSelector = callableSelector;
        this.current = 0;

        Preconditions.checkState(!hir.getCallables().isEmpty());
    }

    @Override
    public TestCase get() {
        var callable = hir.getCallables().get(current);
        var testCase = new TestCase(TestIdGenerator.get(), hir, mutation, crossover, mir, callableSelector);
        while (testCase.size() < Constants.INITIAL_CHROMOSOME_LENGTH) {
            testCase.insertCallable(callable);
        }

        current = (current + 1) % hir.getCallables().size();

        return testCase;
    }
}
