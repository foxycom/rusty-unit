package de.unipassau.testify.exec;

import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.source.ChromosomeContainer;
import java.io.IOException;

public interface ChromosomeExecutor<C extends AbstractTestCaseChromosome<C>> {
  enum Status {
    COMPILATION_ERROR, OK;
  }
  Status runWithInstrumentation(ChromosomeContainer<C> container)
      throws IOException, InterruptedException;

  LLVMCoverage run(ChromosomeContainer<C> container)
      throws IOException, InterruptedException;
}
