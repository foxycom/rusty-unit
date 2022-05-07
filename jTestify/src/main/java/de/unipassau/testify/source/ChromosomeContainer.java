package de.unipassau.testify.source;

import de.unipassau.testify.exec.ChromosomeExecutor.Status;
import de.unipassau.testify.exec.LLVMCoverage;
import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import java.io.IOException;
import java.util.List;

public interface ChromosomeContainer<C extends AbstractTestCaseChromosome<C>> {
  void addAll(List<C> chromosomes);
  void refresh();
  List<C> chromosomes();

  /**
   * Execute the tests within the container with instrumentation.
   *
   * @return status of the execution
   */
  Status execute();
  LLVMCoverage executeWithLlvmCoverage() throws IOException, InterruptedException;
  String getPath();
  String getName();

  C chromosomeAt(String path, int line);
}
