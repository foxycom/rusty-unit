package de.unipassau.rustyunit.test_case.seed;


public class SeedOptions {

  private static boolean useConstantPool;
  private static boolean initialRandomPopulation;
  private static boolean useAllMethods;

  public static boolean useConstantPool() {
    return useConstantPool;
  }

  public static boolean initialRandomPopulation() {
    return initialRandomPopulation;
  }

  public static boolean useAllMethods() {
    return useAllMethods;
  }

  public static  boolean any() {
    return useAllMethods || initialRandomPopulation || useConstantPool;
  }

  public static void setUseConstantPool(boolean value) {
    useConstantPool = value;
  }

  public static void setInitialRandomPopulation(boolean value) {
    initialRandomPopulation = value;
  }

  public static void setUseAllMethods(boolean value) {
    useAllMethods = value;
  }
}
