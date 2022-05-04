package de.unipassau.testify.exec;

import com.jayway.jsonpath.JsonPath;
import de.unipassau.testify.Constants;
import de.unipassau.testify.exception.TestCaseDoesNotCompileException;
import de.unipassau.testify.server.RedisStorage;
import de.unipassau.testify.source.ChromosomeContainer;
import de.unipassau.testify.test_case.TestCase;
import java.io.BufferedReader;
import java.io.File;
import java.io.IOException;
import java.io.InputStreamReader;
import java.math.BigDecimal;
import java.nio.charset.Charset;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.concurrent.TimeUnit;
import org.apache.commons.io.IOUtils;
import org.javatuples.Pair;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class TestCaseRunner implements ChromosomeExecutor<TestCase> {

  private static final Logger logger = LoggerFactory.getLogger(TestCaseRunner.class);

  private static final Path COVERAGE_DIR = Paths.get(System.getProperty("user.dir"), "..", "tmp",
      "coverage");
  private static final Path LOG_PATH = Paths.get(System.getProperty("user.dir"), "..", "tmp",
      "jTestify",
      "tests.log");
  private static final Path ERROR_PATH = Paths.get(System.getProperty("user.dir"), "..", "tmp",
      "jTestify",
      "tests.error");
  private static final Path SCRIPTS_PATH = Paths.get(System.getProperty("user.dir"), "scripts");

  public TestCaseRunner() {
  }

  private void clear() {
    Arrays.stream(Objects.requireNonNull(COVERAGE_DIR.toFile().listFiles())).filter(File::isFile)
        .forEach(File::delete);
  }

  private int collectCoverageFiles(File directory) throws IOException, InterruptedException {
    var processBuilder = new ProcessBuilder("cargo", Constants.RUST_TOOLCHAIN, "test",
        Constants.TEST_MOD_NAME).directory(directory).redirectOutput(LOG_PATH.toFile())
        .redirectError(ERROR_PATH.toFile());
    var env = processBuilder.environment();
    env.put("RUSTFLAGS", "-Z instrument-coverage");

    var profRawFileName = String.format("%s-%%m.profraw", Constants.TEST_PREFIX.replace("_", "-"));
    env.put("LLVM_PROFILE_FILE", Paths.get(COVERAGE_DIR.toString(), profRawFileName).toString());

    var process = processBuilder.start();
    return process.waitFor();
  }

  private int mergeCoverageFiles(File directory) throws IOException, InterruptedException {
//    var profRawFiles = Arrays.stream(Objects.requireNonNull(COVERAGE_DIR.toFile().listFiles()))
//        .filter(f -> f.getName().endsWith("profraw"))
//        .map(f -> {
//          try {
//            return f.getCanonicalPath();
//          } catch (IOException e) {
//            throw new RuntimeException(e);
//          }
//        })
//        .collect(Collectors.joining(" "));
    var profRawFiles = Paths.get(COVERAGE_DIR.toFile().getCanonicalPath(), "rusty-test*.profraw");
    var command = String.format(
        "cargo %s profdata -- merge -sparse %s -o %s",
        Constants.RUST_TOOLCHAIN,
        profRawFiles,
        Paths.get(COVERAGE_DIR.toFile().getCanonicalPath(), "rusty-tests.profdata"));

    var processBuilder = new ProcessBuilder("bash", "-c", command).directory(directory)
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

  @Override
  public LLVMCoverage run(ChromosomeContainer<TestCase> container)
      throws IOException, InterruptedException {
    return run(container.getPath());
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

  private Optional<List<Integer>> executeTestsWithInstrumentation(File directory, String crateName)
      throws IOException, InterruptedException, TestCaseDoesNotCompileException {
    System.out.println("\t>> cargo +nightly test");

    var timer = new Timer();
    timer.start();
    var processBuilder = new ProcessBuilder("cargo", Constants.RUST_TOOLCHAIN, "test",
        Constants.TEST_MOD_NAME)
        .directory(directory)
        .redirectError(ERROR_PATH.toFile());

    var env = processBuilder.environment();
    env.put("RUSTC_WRAPPER", Constants.INSTRUMENTATION_BIN);
    env.put("RUST_LOG", "info");
    env.put("RU_STAGE", "instrumentation");
    env.put("RU_CRATE_NAME", crateName);
    env.put("RU_CRATE_ROOT", directory.toString());
    var process = processBuilder.start();
    var output = IOUtils.toString(process.getInputStream(), Charset.defaultCharset());
    var statusCode = process.waitFor();

    var elapsedTime = timer.end();
    System.out.printf("\t>> Finished. Took %ds%n", TimeUnit.MILLISECONDS.toSeconds(elapsedTime));
    if (statusCode != 0) {
      if (output.contains("test result: FAILED.")) {
        // Some tests failed

        List<Integer> failedTests = new ArrayList<>();
        for (String line : output.split("\n")) {
          if (line.startsWith("test") && line.endsWith("FAILED")) {
            var data = line.substring(line.lastIndexOf("_") + 1, line.indexOf(" ..."));
            var testId = Integer.parseInt(data);
            failedTests.add(testId);
          }
        }
        return Optional.of(failedTests);
      } else {
        // Tests did not compile
        throw new TestCaseDoesNotCompileException();
      }
    } else {
      return Optional.empty();
    }
  }

  public static void main(String[] args) throws IOException, InterruptedException {
    var runner = new TestCaseRunner();
    var coverage = runner.run("/Users/tim/Documents/master-thesis/evaluation/current");
    System.out.println(coverage.lineCoverage);
  }


  @Override
  public int runWithInstrumentation(ChromosomeContainer<TestCase> container)
      throws IOException, InterruptedException {
    RedisStorage.clear();

    var directory = new File(container.getPath());
    Optional<List<Integer>> failedTestIds;
    try {
      failedTestIds = executeTestsWithInstrumentation(directory, container.getName());
    } catch (TestCaseDoesNotCompileException e) {
      logger.error("Tests did not compile", e);
      return 1;
    }

    failedTestIds.ifPresent(tests -> logger.info(tests.size() + " tests failed"));
    var coverage = RedisStorage.<TestCase>requestTraces();

    if (failedTestIds.isPresent()) {
      var ids = failedTestIds.get();
      var failedTests = container.chromosomes().stream().filter(t -> ids.contains(t.getId())).toList();
      container.chromosomes().removeAll(failedTests);
      failedTests.forEach(t -> t.metadata().setFails(true));
    }

    for (TestCase testCase : container.chromosomes()) {
      var testCoverage = coverage.get(testCase.getId());
      testCase.setCoverage(testCoverage);
    }

    container.refresh();

    return 0;
  }
}
