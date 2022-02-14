package de.unipassau.testify;

import java.io.FileInputStream;
import java.io.IOException;
import java.util.Properties;

public class Constants {

  private static final Properties properties = loadProperties();

  public static final int CHROMOSOME_LENGTH = getInt("chromosome-length");
  public static final double P_TEST_CHANGE = getDouble("p-test-change");
  public static final double P_TEST_INSERT = getDouble("p-test-insert");
  public static final double P_TEST_DELETE = getDouble("p-test-delete");
  public static final double P_STMT_INSERT = getDouble("p-statement-insert");
  public static final double P_CHANGE_PARAMETER = getDouble("p-change-parameter");
  public static final double P_CROSSOVER = getDouble("p-crossover");
  public static final double P_RANDOM_PERTURBATION = getDouble("p-random-perturbation");
  public static final double P_LOCAL_VARIABLES = getDouble("p-local-variables");
  public static final int POPULATION_SIZE = getInt("population-size");
  public static final int GENERATIONS = getInt("generations");
  public static final int MAX_DELTA = getInt("max-delta");
  public static final int MAX_INT = getInt("max-int");
  public static final String MIR_LOG_PATH = properties.getProperty("mir-log-path");
  public static final String HIR_LOG_PATH = properties.getProperty("hir-log-path");

  private static Properties loadProperties() {
    var properties = new Properties();
    try {
      properties.load(new FileInputStream(
          "/Users/tim/Documents/master-thesis/jTestify/src/main/resources/config.properties"));

    } catch (IOException e) {
      e.printStackTrace();
      System.exit(1);
    }

    return properties;
  }

  private static int getInt(String name) {
    return Integer.parseInt(properties.getProperty(name));
  }

  private static double getDouble(String name) {
    return Double.parseDouble(properties.getProperty(name));
  }
}
