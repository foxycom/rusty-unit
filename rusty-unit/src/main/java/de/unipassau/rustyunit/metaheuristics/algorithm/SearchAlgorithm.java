package de.unipassau.rustyunit.metaheuristics.algorithm;

public interface SearchAlgorithm<C> {

  /**
   * <p>
   * Runs the search algorithm and returns a possible admissible solution of the encoded problem.
   * </p>
   * <p>
   * Note: every run must perform a new search and must be independent of the previous one. In
   * particular, it must be possible to call this method multiple times in a row. Implementors
   * must ensure multiple runs do not interfere each other.
   * </p>
   *
   * @return a solution
   */
  C findSolution();
}