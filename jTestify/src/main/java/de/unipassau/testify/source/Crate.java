package de.unipassau.testify.source;

import static de.unipassau.testify.Constants.HIR_LOG_PATH;

import com.google.common.base.Charsets;
import com.google.common.base.Preconditions;
import com.google.common.io.FileWriteMode;
import de.unipassau.testify.Constants;
import de.unipassau.testify.exec.ChromosomeExecutor;
import de.unipassau.testify.exec.LLVMCoverage;
import de.unipassau.testify.exec.TestCaseRunner;
import de.unipassau.testify.hir.TyCtxt;
import de.unipassau.testify.json.JSONParser;
import de.unipassau.testify.source.SourceFile.FileType;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.util.Rnd;
import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.stream.Collectors;
import org.apache.commons.io.FileUtils;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class Crate implements ChromosomeContainer<TestCase> {

  private static final Logger logger = LoggerFactory.getLogger(ChromosomeContainer.class);

  private static final Path ERROR_PATH = Paths.get(System.getProperty("user.dir"), "..", "tmp",
      "jTestify",
      "tests.error");
  private static final String MONITOR_PATH = "/Users/tim/Documents/master-thesis/testify/src/monitor.rs";
  private final Path originalRoot;
  private final Path executionRoot;
  private final List<SourceFile> sourceFiles;
  private final ChromosomeExecutor<TestCase> executor;
  private final String crateName;
  private List<TestCase> testCases;

  public static Crate parse(Path root, List<Path> mainFiles, String crateName)
      throws IOException, InterruptedException {
    var executionRoot = getExecutionRoot(root);

    var sourceFiles = Files.walk(root)
        .filter(p -> !Files.isDirectory(p))
        .filter(p -> p.toString().toLowerCase(Locale.ROOT).endsWith(".rs"))
        .filter(p -> p.getParent().endsWith("src"))
        .map(p -> {
          var relativePath = root.relativize(p);
          var executionPath = executionRoot.resolve(relativePath);
          if (mainFiles.stream().anyMatch(p::endsWith)) {
            return new SourceFile(p, executionPath, FileType.MAIN);
          } else {
            return new SourceFile(p, executionPath, FileType.SOURCE_CODE);
          }
        }).toList();

    return new Crate(crateName, root, getExecutionRoot(root), sourceFiles, new TestCaseRunner());
  }

  private static Path getExecutionRoot(Path root) {
    return Paths.get("/Users/tim/Documents/master-thesis/evaluation/current");
  }

  private Crate(String crateName, Path originalRoot, Path executionRoot,
      List<SourceFile> sourceFiles, ChromosomeExecutor<TestCase> executor)
      throws IOException, InterruptedException {
    this.originalRoot = originalRoot;
    this.executionRoot = executionRoot;
    this.sourceFiles = sourceFiles;
    this.executor = executor;
    this.crateName = crateName;
    this.testCases = new ArrayList<>();
//    this.hir = analyze(originalRoot, crateName);
    copyToExecutionDir();
  }

  public SourceFile getFileByPath(String path) {
    var maybeFile = sourceFiles.stream().filter(s -> s.getExecutionPath().endsWith(path))
        .findFirst();
    if (maybeFile.isPresent()) {
      return maybeFile.get();
    } else {
      throw new RuntimeException("No file found");
    }
  }

  public Path getOriginalRoot() {
    return originalRoot;
  }

  public Path getExecutionRoot() {
    return executionRoot;
  }

  public List<SourceFile> getSourceFiles() {
    return sourceFiles;
  }

  private void copyToExecutionDir() throws IOException {
    FileUtils.copyDirectory(originalRoot.toFile(), executionRoot.toFile());

    var monitorFile = new File(MONITOR_PATH);
    var executionMonitorFile = Paths.get(executionRoot.toString(), "src", "rusty_monitor.rs");
    com.google.common.io.Files.copy(monitorFile, executionMonitorFile.toFile());
    for (SourceFile sourceFile : sourceFiles) {
      sourceFile.onCopied();
    }

    // Add redis dependency
    var cargoToml = findCargoToml();
    addDependencies(cargoToml);
  }

  private void addDependencies(Path cargoToml) throws IOException {
    var sink = com.google.common.io.Files.asCharSink(cargoToml.toFile(), Charsets.UTF_8,
        FileWriteMode.APPEND);
    sink.write("\n[dependencies.redis]\nversion = \"*\"\n");

    //[dependencies.redis]
    //version = "*"
  }

  private Path findCargoToml() {
    var tomlFiles = FileUtils.listFiles(executionRoot.toFile(), new String[]{"toml"}, true);
    if (tomlFiles.isEmpty()) {
      throw new RuntimeException(String.format("No Cargo.toml found in %s", executionRoot));
    } else if (tomlFiles.size() > 1) {
      throw new RuntimeException("Multiple Cargo.toml files not supported yet");
    }

    return tomlFiles.stream().findFirst().get().toPath();
  }

  @Override
  public void refresh() {
    Preconditions.checkState(!sourceFiles.isEmpty());
    Map<String, List<TestCase>> sorted = new HashMap<>();

    var allowedFiles = sourceFiles.stream()
        .filter(f -> !f.getExecutionPath().endsWith("lib.rs")
            && !f.getExecutionPath().endsWith("monitor.rs")).toList();

    testCases.forEach(testCase -> {
      var filePathBinding = testCase.getFilePathBinding()
          .orElseGet(() -> executionRoot.relativize(Rnd.choice(allowedFiles).getExecutionPath())
              .toString());
      sorted.putIfAbsent(filePathBinding, new ArrayList<>());
      sorted.get(filePathBinding).add(testCase);
    });

    sorted.forEach((path, tests) -> {
      var file = getFileByPath(path);
      try {
        file.addTests(tests);
      } catch (IOException | InterruptedException e) {
        throw new RuntimeException(e);
      }
    });
  }

  @Override
  public void addAll(List<TestCase> testCases) {
    this.testCases = testCases;

    refresh();
  }

  @Override
  public List<TestCase> chromosomes() {
    return testCases;
  }

  @Override
  public int executeWithInstrumentation() {
    // Write tests into the source files
    try {
      return executor.runWithInstrumentation(this);
    } catch (IOException | InterruptedException e) {
      throw new RuntimeException(e);
    }
  }

  @Override
  public LLVMCoverage executeWithLlvmCoverage() throws IOException, InterruptedException {
    return executor.run(this);
  }

  private TyCtxt analyze(Path root, String crateName) throws IOException, InterruptedException {

    var processBuilder = new ProcessBuilder().directory(root.toFile())
        .command("cargo", Constants.RUST_TOOLCHAIN, "build", "--all-features")
        .redirectError(ERROR_PATH.toFile());

    var env = processBuilder.environment();
    env.put("RUSTC_WRAPPER", Constants.ANALYSIS_BIN);
    env.put("RU_CRATE_NAME", crateName);
    env.put("RU_CRATE_ROOT", root.toFile().getCanonicalPath());
    var process = processBuilder.start();
    var result = process.waitFor();

    if (result != 0) {
      logger.error("HIR analysis failed");
      throw new RuntimeException("HIR analysis failed");
    }

    var hirLog = new File(HIR_LOG_PATH);
    var hirJson = Files.readString(hirLog.toPath());
    return new TyCtxt(JSONParser.parse(hirJson));
  }

  @Override
  public String getPath() {
    return executionRoot.toString();
  }

  @Override
  public String getName() {
    return crateName;
  }

}