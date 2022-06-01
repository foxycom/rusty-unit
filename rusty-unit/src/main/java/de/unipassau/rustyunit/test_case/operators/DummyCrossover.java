package de.unipassau.rustyunit.test_case.operators;

import de.unipassau.rustyunit.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.rustyunit.metaheuristics.operators.Crossover;
import org.javatuples.Pair;

public class DummyCrossover<C extends AbstractTestCaseChromosome<C>> implements Crossover<C> {

  @Override
  public Pair<C, C> apply(C parent1, C parent2) {
    return Pair.with(parent1, parent2);
  }
}
