package de.unipassau.testify.algorithm;

import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import java.util.List;
import java.util.Map;

public interface PreferenceSorter<C extends AbstractTestCaseChromosome<C>> {
  Map<Integer, List<C>> sort(List<C> population);
}
