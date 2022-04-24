package de.unipassau.testify.algorithm;

import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

import de.unipassau.testify.hir.TyCtxt;
import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import de.unipassau.testify.mir.BasicBlock;
import de.unipassau.testify.test_case.TestCase;
import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

@ExtendWith(MockitoExtension.class)
class FNDSImplTest {

  @Mock
  private DominationStrategy<TestCase> domination;

  @Mock
  private TyCtxt tyCtxt;

  private FNDS<TestCase> fnds;

  @BeforeEach
  void setUp() {
    fnds = new FNDSImpl<>(domination);
  }

  @Test
  void testSort() {
    final List<MinimizingFitnessFunction<TestCase>> objectives = List.of(
        BasicBlock.of("id", 1),
        BasicBlock.of("id", 2),
        BasicBlock.of("id", 3),
        BasicBlock.of("id", 4)
    );

    TestCase testCase1 = mock(TestCase.class);
    TestCase testCase2 = mock(TestCase.class);

    when(domination.dominates(testCase1, testCase1, objectives)).thenReturn(false, false);
    when(domination.dominates(testCase2, testCase2, objectives)).thenReturn(false, false);
    when(domination.dominates(testCase1, testCase2, objectives)).thenReturn(true);


    final var population = List.of(
        testCase1,
        testCase2
    );

    var result = fnds.sort(population, objectives);
  }
}