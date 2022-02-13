package de.unipassau.testify.source;

import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import java.util.List;

public interface ChromosomeContainer<C extends AbstractTestCaseChromosome<C>> {
  void addAll(List<C> chromosomes);
  List<C> chromosomes();
  void executeWithInstrumentation();
  String getPath();
  String getName();
}
