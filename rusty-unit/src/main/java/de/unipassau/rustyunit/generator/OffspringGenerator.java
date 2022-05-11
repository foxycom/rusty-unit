package de.unipassau.rustyunit.generator;

import de.unipassau.rustyunit.metaheuristics.chromosome.AbstractTestCaseChromosome;
import java.util.List;

public interface OffspringGenerator<C extends AbstractTestCaseChromosome<C>> {
  List<C> get(List<C> population);
}
