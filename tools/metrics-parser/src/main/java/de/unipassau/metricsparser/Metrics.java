package de.unipassau.metricsparser;

public record Metrics(String fileName, int methods, int lloc) {

  @Override
  public String toString() {
    return String.format("File: %s%nMethods: %d%nLLOC: %d%n", fileName, methods, lloc);
  }
}
