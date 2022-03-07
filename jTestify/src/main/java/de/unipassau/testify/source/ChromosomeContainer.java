package de.unipassau.testify.source;

import de.unipassau.testify.exec.LLVMCoverage;
import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import java.io.IOException;
import java.util.List;

public interface ChromosomeContainer<C extends AbstractTestCaseChromosome<C>> {
  void addAll(List<C> chromosomes);
  List<C> chromosomes();
  void executeWithInstrumentation();
  LLVMCoverage executeWithLlvmCoverage() throws IOException, InterruptedException;
  String getPath();
  String getName();
}
