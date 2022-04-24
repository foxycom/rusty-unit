package de.unipassau.testify.mir;

import static com.google.common.truth.Truth.assertThat;
import static org.mockito.Mockito.when;

import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import de.unipassau.testify.test_case.TestCase;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

@ExtendWith(MockitoExtension.class)
public class BasicBlockTest {

  private BasicBlock basicBlock;

  @Mock
  private TestCase testCase;

  @Mock
  private MirAnalysis<TestCase> mir;

  @Mock
  private CDG<MinimizingFitnessFunction<TestCase>, TestCase> cdg;

  @BeforeEach
  public void setUp() {
    basicBlock = new BasicBlock("id", 42);
  }

  @Test
  public void testGetLocalFitness() {
    Map<MinimizingFitnessFunction<TestCase>, Double> map = new HashMap<>();
    map.put(basicBlock, 2.0);
    when(testCase.branchDistance()).thenReturn(map);

    double fitness = basicBlock.getFitness(testCase);

    assertThat(fitness).isEqualTo(2.0 / 3.0);
  }

  @Test
  public void testGetWithApproachLevel() {
    Map<MinimizingFitnessFunction<TestCase>, Double> map = new HashMap<>();
    map.put(new BasicBlock("id", 5), 10.0);

    when(testCase.branchDistance()).thenReturn(map);
    when(testCase.mir()).thenReturn(mir);
    when(mir.getCdgFor(basicBlock.globalId())).thenReturn(cdg);

    List<MinimizingFitnessFunction<TestCase>> path = List.of(new BasicBlock("id", 4), new BasicBlock("id", 5));
    when(cdg.pathTo(basicBlock)).thenReturn(path);

    double fitness = basicBlock.getFitness(testCase);

    assertThat(fitness).isEqualTo(1 + 10.0 / 11.0);
  }

  @Test
  public void testGetNoHit() {
    Map<MinimizingFitnessFunction<TestCase>, Double> map = new HashMap<>();
    when(testCase.branchDistance()).thenReturn(map);
    when(testCase.mir()).thenReturn(mir);
    when(mir.getCdgFor(basicBlock.globalId())).thenReturn(cdg);

    List<MinimizingFitnessFunction<TestCase>> path = List.of(new BasicBlock("id", 4), new BasicBlock("id", 5));
    when(cdg.pathTo(basicBlock)).thenReturn(path);

    double fitness = basicBlock.getFitness(testCase);

    assertThat(fitness).isEqualTo(Double.MAX_VALUE);
  }

  @Test
  public void testFitnessCache() {
    Map<MinimizingFitnessFunction<TestCase>, Double> map = new HashMap<>();
    map.put(new BasicBlock("id", 5), 10.0);

    when(testCase.branchDistance()).thenReturn(map);
    when(testCase.mir()).thenReturn(mir);
    when(mir.getCdgFor(basicBlock.globalId())).thenReturn(cdg);

    List<MinimizingFitnessFunction<TestCase>> path = List.of(new BasicBlock("id", 4), new BasicBlock("id", 5));
    when(cdg.pathTo(basicBlock)).thenReturn(path);

    double fitness = basicBlock.getFitness(testCase);
    double secondFitness = basicBlock.getFitness(testCase);

    assertThat(fitness).isEqualTo(1 + 10.0 / 11.0);
    assertThat(secondFitness).isEqualTo(1 + 10.0 / 11.0);
  }
}