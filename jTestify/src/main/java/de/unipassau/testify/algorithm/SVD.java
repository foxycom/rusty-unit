package de.unipassau.testify.algorithm;

import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import java.util.List;

public interface SVD<C extends AbstractTestCaseChromosome<C>> {
  void compute(List<C> population);
}
