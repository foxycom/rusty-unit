package de.unipassau.testify.exec;

public class LLVMCoverage {
  public final double lineCoverage;
  public final double regionCoverage;

  public LLVMCoverage(double lineCoverage, double regionCoverage) {
    this.lineCoverage = lineCoverage;
    this.regionCoverage = regionCoverage;
  }
}
