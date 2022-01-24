package de.unipassau.testify.source;

import de.unipassau.testify.source.SourceFile.FileType;
import de.unipassau.testify.test_case.TestCase;
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

public class Crate {

  private static final String MONITOR_PATH = "/Users/tim/Documents/master-thesis/testify/instrumentation/src/monitor.rs";
  private final Path originalRoot;
  private final Path executionRoot;
  private final List<SourceFile> sourceFiles;

  public static Crate parse(Path root, List<Path> mainFiles) throws IOException {
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

    return new Crate(root, getExecutionRoot(root), sourceFiles);
  }

  private static Path getExecutionRoot(Path root) {
    return Paths.get("/Users/tim/Documents/master-thesis/evaluation/current");
  }

  private Crate(Path originalRoot, Path executionRoot,
      List<SourceFile> sourceFiles) throws IOException {
    this.originalRoot = originalRoot;
    this.executionRoot = executionRoot;
    this.sourceFiles = sourceFiles;
    copyToExecutionDir();
  }

  public SourceFile getFileByPath(String path) {
    return sourceFiles.stream().filter(s -> s.getExecutionPath().endsWith(path)).findFirst().get();
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
    var executionMonitorFile = Paths.get(executionRoot.toString(), "src", "testify_monitor.rs");
    com.google.common.io.Files.copy(monitorFile, executionMonitorFile.toFile());
    for (SourceFile sourceFile : sourceFiles) {
      sourceFile.onCopied();
    }
  }

  public void addTests(List<TestCase> testCases) {
    Map<String, List<TestCase>> sorted = new HashMap<>();
    testCases.forEach(testCase -> {
      var filePathBinding = testCase.getFilePathBinding().orElse("UNBOUND");
      sorted.putIfAbsent(filePathBinding, new ArrayList<>());
      sorted.get(filePathBinding).add(testCase);
    });

    sorted.forEach((path, tests) -> {
      if (path.equals("UNBOUND")) {
        // TODO: 23.12.21 pick a random file
        throw new RuntimeException("Not implemented yet");
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

}
