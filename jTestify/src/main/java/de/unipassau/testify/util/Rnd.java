package de.unipassau.testify.util;

import java.util.Collection;
import java.util.List;
import java.util.Random;

public final class Rnd {
  /**
   * Internal source of randomness.
   */
  private static final Random random = new Random();

  private Rnd() {
    // private constructor to prevent instantiation
  }

  /**
   * Returns the source of randomness.
   *
   * @return randomness
   */
  public static Random random() {
    return random;
  }

  public static <T> T element(List<T> list) {
    return list.get(random.nextInt(list.size()));
  }
}
