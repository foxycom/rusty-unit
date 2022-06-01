package de.unipassau.rustyunit;

import de.unipassau.rustyunit.metaheuristics.chromosome.AbstractTestCaseChromosome;
import java.util.List;
import lombok.Builder;

public interface Listener<C extends AbstractTestCaseChromosome<C>> {

  @Builder
  public class Status {
      public final int generation;
      public final int coveredTargets;
      public final double coverage;
      public final double avgLength;
      public final int tests;
  }

  void onExecuted(Status status);

  void onPopulation(int generation, List<C> population);
}
