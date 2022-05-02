package de.unipassau.testify.linearity;

import java.util.List;

public class Mutation implements Operator {

  public static final String NAME = "Mutation";

  private final List<Integer> parents;

  public Mutation(List<Integer> parents) {
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
