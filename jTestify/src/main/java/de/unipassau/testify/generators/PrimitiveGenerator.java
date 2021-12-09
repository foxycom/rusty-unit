package de.unipassau.testify.generators;

import de.unipassau.testify.test_case.type.prim.Prim;

public class PrimitiveGenerator {
  public static Object get(Prim prim) {
    return prim.random();
  }
}
