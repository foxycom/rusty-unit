package de.unipassau.rustyunit;

import de.unipassau.rustyunit.test_case.TestCase;
import de.unipassau.rustyunit.test_case.visitor.TestCaseVisitor;
import java.sql.Connection;
import java.sql.DriverManager;
import java.sql.PreparedStatement;
import java.sql.SQLException;
import java.sql.Statement;
import java.util.List;

public class DB implements Listener<TestCase> {

  private static final String URL = String.format("jdbc:postgresql://localhost/%s",
      Constants.DB_NAME);
  private static final String METADATA_PREFIX = "experiments";
  private static final String TESTS_PREFIX = "tests";

  private final String METADATA_TABLE;
  private final String TESTS_TABLE;

  private final String crate;
  private final int run;
  private final String algorithm;

  public DB(String crate, String algorithm, int run) {
    this.crate = crate;
    this.run = run;
    this.algorithm = algorithm;
    this.METADATA_TABLE = METADATA_PREFIX + "_" + algorithm;
    this.TESTS_TABLE = TESTS_PREFIX + "_" + algorithm;
  }


  @Override
  public void onExecuted(Status status) {
    String tableStmt = "CREATE TABLE IF NOT EXISTS " + METADATA_TABLE
        + "(crate VARCHAR(40), run INT, gen INT, "
        + "mir_coverage DOUBLE PRECISION, covered_targets INT, tests INT, tests_length DOUBLE PRECISION,"
        + "PRIMARY KEY(crate, run, gen))";
    var dataStmt = "INSERT INTO " + METADATA_TABLE
        + " (crate, run, gen, mir_coverage, covered_targets, tests, tests_length)"
        + " VALUES(?, ?, ?, ?, ?, ?, ?) ON CONFLICT (crate, run, gen) DO NOTHING";

    try (Connection conn = DriverManager.getConnection(URL, Constants.DB_USER,
        Constants.DB_PASSWORD)) {
      try (Statement stmt = conn.createStatement()) {
        stmt.execute(tableStmt);
      }
      try (PreparedStatement stmt = conn.prepareStatement(dataStmt)) {
        stmt.setString(1, crate);
        stmt.setInt(2, run);
        stmt.setInt(3, status.generation);
        stmt.setDouble(4, status.coverage);
        stmt.setInt(5, status.coveredTargets);
        stmt.setInt(6, status.tests);
        stmt.setDouble(7, status.avgLength);
        stmt.executeUpdate();
      }
    } catch (SQLException e) {
      throw new RuntimeException(e);
    }
  }

  @Override
  public void onPopulation(int generation, List<TestCase> population) {
    var visitor = new TestCaseVisitor();
    String tableStmt = "CREATE TABLE IF NOT EXISTS " + TESTS_TABLE
        + "(crate VARCHAR(40), run INT, gen INT, test_id INT,"
        + "path_binding VARCHAR, test_case TEXT, "
        + "PRIMARY KEY(crate, run, gen, test_id))";
    var dataStmt = "INSERT INTO " + TESTS_TABLE
        + " (crate, run, gen, test_id, path_binding, test_case)"
        + " VALUES(?, ?, ?, ?, ?, ?) ON CONFLICT (crate, run, gen, test_id) DO NOTHING";

    try (Connection conn = DriverManager.getConnection(URL, Constants.DB_USER,
        Constants.DB_PASSWORD)) {
      try (Statement stmt = conn.createStatement()) {
        stmt.execute(tableStmt);
      }

      try (PreparedStatement stmt = conn.prepareStatement(dataStmt)) {
        for (TestCase testCase : population) {
          stmt.setString(1, crate);
          stmt.setInt(2, run);
          stmt.setInt(3, generation);
          stmt.setInt(4, testCase.getId());
          stmt.setString(5, testCase.getFilePathBinding().orElse(null));
          stmt.setString(6, testCase.visit(visitor));
          stmt.addBatch();
          stmt.clearParameters();
        }
        stmt.executeBatch();
      }


    } catch (SQLException e) {
      throw new RuntimeException(e);
    }
  }
}
