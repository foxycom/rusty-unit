package de.unipassau.testify.exec;

import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.source.ChromosomeContainer;
import java.io.IOException;

public interface ChromosomeExecutor<C extends AbstractTestCaseChromosome<C>> {
  LLVMCoverage run(ChromosomeContainer<C> container);
  int runWithInstrumentation(ChromosomeContainer<C> container)
      throws IOException, InterruptedException;
}
