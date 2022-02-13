package de.unipassau.testify.exec;

import com.jayway.jsonpath.JsonPath;
import de.unipassau.testify.server.RedisStorage;
import de.unipassau.testify.source.ChromosomeContainer;
import de.unipassau.testify.test_case.TestCase;
import java.io.BufferedReader;
import java.io.File;
import java.io.IOException;
import java.io.InputStreamReader;
import java.math.BigDecimal;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Arrays;
import java.util.Objects;
import java.util.stream.Collectors;
import org.javatuples.Pair;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class TestCaseRunner implements ChromosomeExecutor<TestCase> {

  public static final String INSTRUMENTER_PATH = "/Users/tim/Documents/master-thesis/testify/target/debug/instrumentation";

  private static final Logger logger = LoggerFactory.getLogger(TestCaseRunner.class);

  private static final Path COVERAGE_DIR = Paths.get(System.getProperty("user.dir"), "..", "tmp", "coverage");
  private static final Path LOG_PATH = Paths.get(System.getProperty("user.dir"), "..", "tmp", "jTestify",
      "tests.log");
  private static final Path ERROR_PATH = Paths.get(System.getProperty("user.dir"), "..", "tmp", "jTestify",
      "tests.error");
  private static final Path SCRIPTS_PATH = Paths.get(System.getProperty("user.dir"), "scripts");

  public TestCaseRunner() {
  }

  private void clear() {
    Arrays.stream(Objects.requireNonNull(COVERAGE_DIR.toFile().listFiles())).filter(File::isFile)
        .forEach(File::delete);
  }

  private int collectCoverageFiles(File directory) throws IOException, InterruptedException {
    var processBuilder = new ProcessBuilder("cargo", "+nightly-aarch64-apple-darwin", "test",
        "rusty_tests").directory(directory).redirectOutput(LOG_PATH.toFile())
        .redirectError(ERROR_PATH.toFile());
    var env = processBuilder.environment();
    env.put("RUSTFLAGS", "-Z instrument-coverage");
    env.put("LLVM_PROFILE_FILE",
        Paths.get(COVERAGE_DIR.toString(), "rusty-test-%m.profraw").toString());

    var process = processBuilder.start();
    return process.waitFor();
  }

  private int mergeCoverageFiles(File directory) throws IOException, InterruptedException {
    var profRawFiles = Arrays.stream(Objects.requireNonNull(COVERAGE_DIR.toFile().listFiles()))
        .filter(f -> f.getName().endsWith("profraw")).map(File::getAbsolutePath)
        .collect(Collectors.joining(","));
    var processBuilder = new ProcessBuilder("cargo", "+nightly-aarch64-apple-darwin", "profdata",
        "--", "merge", "-sparse", profRawFiles, "-o",
        Paths.get(COVERAGE_DIR.toString(), "rusty-tests.profdata").toString()).directory(directory)
        .redirectOutput(LOG_PATH.toFile()).redirectError(ERROR_PATH.toFile());
    var process = processBuilder.start();
    return process.waitFor();
  }

  private Pair<Integer, String> createCoverageReport(File directory)
      throws InterruptedException, IOException {
    var script = Paths.get(SCRIPTS_PATH.toString(), "coverage-report.sh").toString();

    var profdata = Paths.get(COVERAGE_DIR.toString(), "rusty-tests.profdata").toString();

    var processBuilder = new ProcessBuilder(script, profdata).directory(directory)
        .redirectError(ERROR_PATH.toFile());
    var process = processBuilder.start();

    var reader = new BufferedReader(new InputStreamReader(process.getInputStream()));
    var sb = new StringBuilder();
    String line = null;
    while ((line = reader.readLine()) != null) {
      sb.append(line);
      sb.append(System.getProperty("line.separator"));
    }
    var output = sb.toString();

    return Pair.with(process.waitFor(), output);
  }

  public LLVMCoverage run(String path) throws IOException, InterruptedException {
    clear();

    var directory = new File(path);
    if (collectCoverageFiles(directory) != 0) {
      logger.error("Could not run tests for some reason");
      throw new RuntimeException("Could not run tests for some reason");
    }

    if (mergeCoverageFiles(directory) != 0) {
      logger.error("Could not merge tests for some reason");
      throw new RuntimeException("Could not merge tests for some reason");
    }

    var coverageResult = createCoverageReport(directory);
    if (coverageResult.getValue0() != 0) {
      logger.error("Could not create a coverage report");
      throw new RuntimeException("Could not create a coverage report");
    }

    var lineCoverage = JsonPath.read(coverageResult.getValue1(), "$.data[0].totals.lines.percent");
    if (lineCoverage instanceof Double) {
      return new LLVMCoverage((double) lineCoverage);
    } else if (lineCoverage instanceof BigDecimal) {
      var coverage = ((BigDecimal) lineCoverage).doubleValue();
      return new LLVMCoverage(coverage);
    } else {
      throw new RuntimeException("Not implemented yet");
    }
  }

  private int executeTestsWithInstrumentation(File directory, String crateName)
      throws IOException, InterruptedException {
    var processBuilder = new ProcessBuilder("cargo", "+nightly-aarch64-apple-darwin", "test",
        "rusty_tests").directory(directory).redirectOutput(LOG_PATH.toFile())
        .redirectError(ERROR_PATH.toFile());

    var env = processBuilder.environment();
    env.put("RUSTC_WRAPPER", INSTRUMENTER_PATH);
    env.put("RUST_LOG", "info");
    env.put("RUSTY_UNIT",
        String.format("--stage=instrument --crate=%s --crate-name=%s", directory.toString(),
            crateName));
    var process = processBuilder.start();
    return process.waitFor();
  }

  public static void main(String[] args) throws IOException, InterruptedException {
    var runner = new TestCaseRunner();
    var coverage = runner.run("/Users/tim/Documents/master-thesis/evaluation/current");
    System.out.println(coverage.lineCoverage);
    /*var llvmCoverage = runner.run("/Users/tim/Documents/master-thesis/evaluation/current");
    System.out.printf("Line coverage: %.2f", llvmCoverage.lineCoverage);*/
    //runner.runWithInstrumentation("/Users/tim/Documents/master-thesis/evaluation/current", "trying");

  }

  @Override
  public LLVMCoverage run(ChromosomeContainer<TestCase> container) {
    throw new RuntimeException("Not implemented yet");
  }

  @Override
  public int runWithInstrumentation(ChromosomeContainer<TestCase> container)
      throws IOException, InterruptedException {
    RedisStorage.clear();

    var directory = new File(container.getPath());
    if (executeTestsWithInstrumentation(directory, container.getName()) != 0) {
      throw new RuntimeException("Could not execute tests with instrumentation");
    }

    var coverage = RedisStorage.requestTraces();
    if (coverage.isEmpty()) {
      throw new RuntimeException("Coverage is empty");
    }
    for (TestCase testCase : container.chromosomes()) {
      var testCoverage = coverage.get(testCase.getId());
      testCase.setCoverage(testCoverage);
    }

    return 0;
  }
}
