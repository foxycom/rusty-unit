package de.unipassau.rustyunit.generators;

import java.util.concurrent.atomic.AtomicInteger;

public class TestIdGenerator {
  private static final AtomicInteger id = new AtomicInteger(0);

  public static int get() {
    return id.getAndIncrement();
  }

}
