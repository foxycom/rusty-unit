package de.unipassau.testify.test_case.seed;

import lombok.Builder;

@Builder
public class SeedOptions {
  private final boolean useConstantPool;
  private final boolean initialRandomPopulation;
  private final boolean useAllMethods;

  public boolean useConstantPool() {
    return useConstantPool;
  }

  public boolean initialRandomPopulation() {
    return initialRandomPopulation;
  }

  public boolean useAllMethods() {
    return useAllMethods;
  }
}
