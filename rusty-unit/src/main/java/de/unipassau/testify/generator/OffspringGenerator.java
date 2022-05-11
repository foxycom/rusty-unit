package de.unipassau.testify.generator;

import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import java.util.List;

public interface OffspringGenerator<C extends AbstractTestCaseChromosome<C>> {
  List<C> get(List<C> population);
}
