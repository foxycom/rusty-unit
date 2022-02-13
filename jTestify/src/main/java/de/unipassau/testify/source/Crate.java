package de.unipassau.testify.source;

import com.google.common.base.Charsets;
import com.google.common.base.Preconditions;
import com.google.common.io.FileWriteMode;
import de.unipassau.testify.exec.ChromosomeExecutor;
import de.unipassau.testify.exec.TestCaseRunner;
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
import org.apache.commons.io.FileUtils;

public class Crate implements ChromosomeContainer<TestCase> {

  private static final String MONITOR_PATH = "/Users/tim/Documents/master-thesis/testify/instrumentation/src/monitor.rs";
  private final Path originalRoot;
  private final Path executionRoot;
  private final List<SourceFile> sourceFiles;
  private final ChromosomeExecutor<TestCase> executor;
  private final String crateName;
  private final List<TestCase> testCases;

  public static Crate parse(Path root, List<Path> mainFiles, String crateName) throws IOException {
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
      List<SourceFile> sourceFiles, ChromosomeExecutor<TestCase> executor) throws IOException {
    this.originalRoot = originalRoot;
    this.executionRoot = executionRoot;
    this.sourceFiles = sourceFiles;
    this.executor = executor;
    this.crateName = crateName;
    this.testCases = new ArrayList<>();
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
  public void addAll(List<TestCase> testCases) {
    this.testCases.clear();
    this.testCases.addAll(testCases);

    Preconditions.checkState(!sourceFiles.isEmpty());
    Map<String, List<TestCase>> sorted = new HashMap<>();
    testCases.forEach(testCase -> {
      var filePathBinding = testCase.getFilePathBinding().orElse("UNBOUND");
      sorted.putIfAbsent(filePathBinding, new ArrayList<>());
      sorted.get(filePathBinding).add(testCase);
    });

    sorted.forEach((path, tests) -> {
      if (path.equals("UNBOUND")) {
        // Take a random path
        var file = Rnd.choice(sourceFiles);
        try {
          file.addTests(tests);
        } catch (IOException | InterruptedException e) {
          throw new RuntimeException(e);
        }
      } else {
        var file = getFileByPath(path);
        try {
          file.addTests(tests);
        } catch (IOException | InterruptedException e) {
          throw new RuntimeException(e);
        }
      }
    });
  }

  @Override
  public List<TestCase> chromosomes() {
    return testCases;
  }

  @Override
  public void executeWithInstrumentation() {
    // Write tests into the source files
    try {
      var statusCode = executor.runWithInstrumentation(this);
    } catch (IOException | InterruptedException e) {
      throw new RuntimeException(e);
    }
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