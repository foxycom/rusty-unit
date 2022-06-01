package de.unipassau.rustyunit.util;

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
  public static Random get() {
    return random;
  }

  public static char nextChar() {
    return (char) (random.nextInt(32, 128));
  }

  public static <T> T choice(List<T> list) {
    return list.get(random.nextInt(list.size()));
  }

  public static <T> T choice(Collection<T> collection) {
    var array = collection.toArray();

    return (T) array[Rnd.get().nextInt(array.length)];
  }
}
