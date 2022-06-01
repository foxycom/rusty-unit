package de.unipassau.rustyunit.linearity;

import java.util.List;

public class Crossover implements Operator {
  public static final String NAME = "Crossover";

  private final List<Integer> parents;

  public Crossover(List<Integer> parents) {
    this.parents = parents;
  }

  @Override
  public String name() {
    return NAME;
  }

  @Override
  public List<Integer> parents() {
    return parents;
  }
}
