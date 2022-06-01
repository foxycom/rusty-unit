package de.unipassau.rustyunit.exec;

import de.unipassau.rustyunit.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.rustyunit.source.ChromosomeContainer;
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
