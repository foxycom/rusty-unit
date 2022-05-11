package de.unipassau.rustyunit.algorithm;

import static com.google.common.truth.Truth.assertThat;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

import de.unipassau.rustyunit.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import de.unipassau.rustyunit.mir.BasicBlock;
import de.unipassau.rustyunit.test_case.TestCase;
import de.unipassau.rustyunit.test_case.metadata.TestCaseMetadata;
import java.util.List;
import java.util.Set;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

@ExtendWith(MockitoExtension.class)
class DefaultArchiveTest {

  private Archive<TestCase> archive;

  private Set<MinimizingFitnessFunction<TestCase>> objectives;

  @Mock
  BasicBlock basicBlock1 = new BasicBlock("id", 1);

  @Mock
  BasicBlock basicBlock2 = new BasicBlock("id", 2);

  @Mock
  BasicBlock basicBlock3 = new BasicBlock("id", 3);

  @Mock
  TestCaseMetadata metadata;

  @BeforeEach
  void setUp() {
    objectives = Set.of(
        basicBlock1,
        basicBlock2,
        basicBlock3
    );
    archive = new DefaultArchive<>(objectives);
  }

  @Test
  void update() {
    var testCase1 = mock(TestCase.class);
    var testCase2 = mock(TestCase.class);
    var testCase3 = mock(TestCase.class);
    var testCase4 = mock(TestCase.class);

    when(testCase1.getId()).thenReturn(1);
    when(testCase2.getId()).thenReturn(2);
    when(testCase3.getId()).thenReturn(3);
    when(testCase4.getId()).thenReturn(4);

    when(testCase1.size()).thenReturn(3);
    when(testCase2.size()).thenReturn(5);
    when(testCase3.size()).thenReturn(1);
    when(testCase4.size()).thenReturn(4);

    when(metadata.fails()).thenReturn(false);

    when(testCase1.metadata()).thenReturn(metadata);
    when(testCase2.metadata()).thenReturn(metadata);
    when(testCase3.metadata()).thenReturn(metadata);
    when(testCase4.metadata()).thenReturn(metadata);

    when(basicBlock1.getFitness(testCase1)).thenReturn(2.0);
    when(basicBlock1.getFitness(testCase2)).thenReturn(0.0);
    when(basicBlock1.getFitness(testCase3)).thenReturn(0.0);
    when(basicBlock1.getFitness(testCase4)).thenReturn(Double.MAX_VALUE);

    when(basicBlock2.getFitness(testCase1)).thenReturn(Double.MAX_VALUE);
    when(basicBlock2.getFitness(testCase2)).thenReturn(10.0);
    when(basicBlock2.getFitness(testCase3)).thenReturn(5.0);
    when(basicBlock2.getFitness(testCase4)).thenReturn(Double.MAX_VALUE);

    when(basicBlock3.getFitness(testCase1)).thenReturn(0.0);
    when(basicBlock3.getFitness(testCase2)).thenReturn(0.0);
    when(basicBlock3.getFitness(testCase3)).thenReturn(5.0);
    when(basicBlock3.getFitness(testCase4)).thenReturn(0.0);

    archive.update(List.of(testCase1, testCase2));
    var p = archive.get().stream().map(TestCase::getId).toList();
    assertThat(p).hasSize(2);
    assertThat(p).containsExactly(1, 2);

    archive.update(List.of(testCase3, testCase4));

    var p2 = archive.get();
    assertThat(p2).containsExactly(testCase1, testCase3);
  }

  @Test
  void getCaseThatCovers() {
  }

  @Test
  void replaceBy() {
  }
}