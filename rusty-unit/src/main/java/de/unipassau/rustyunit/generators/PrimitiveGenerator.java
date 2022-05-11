package de.unipassau.rustyunit.generators;

import de.unipassau.rustyunit.type.prim.Prim;

public class PrimitiveGenerator {
  public static Object get(Prim prim) {
    return prim.random();
  }
}
