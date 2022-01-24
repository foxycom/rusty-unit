package de.unipassau.testify.source;


import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.visitor.TestCaseVisitor;
import java.io.BufferedWriter;
import java.io.FileWriter;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
import java.util.stream.Collectors;

public class SourceFile {

  private static final Path LOG_PATH = Paths.get(System.getProperty("user.dir"), "logs",
      "serialization.log");
  private static final Path ERROR_PATH = Paths.get(System.getProperty("user.dir"), "logs",
      "serialization.error");
  private static final String TEST_WRITER = "/Users/tim/Documents/master-thesis/testify/target/debug/writer";
  private Path originalPath;
  private Path executionPath;
  private FileType type;

  public enum FileType {
    MAIN, SOURCE_CODE;
  }

  public SourceFile(Path originalPath, Path copyPath,
      FileType type) {
    this.originalPath = originalPath;
    this.executionPath = copyPath;
    this.type = type;
  }

  public Path getOriginalPath() {
    return originalPath;
  }

  public void setOriginalPath(Path originalPath) {
    this.originalPath = originalPath;
  }

  public Path getExecutionPath() {
    return executionPath;
  }

  public void setExecutionPath(Path executionPath) {
    this.executionPath = executionPath;
  }

  public FileType getType() {
    return type;
  }

  public void setType(FileType type) {
    this.type = type;
  }

  public void onCopied() throws IOException {
    if (type == FileType.MAIN) {

      try (var out = new BufferedWriter(new FileWriter(executionPath.toFile()))) {
        out.write("pub mod testify_monitor;\n");
        var content = Files.readString(originalPath);
        out.write(content);
      }
    }
  }

  public void addTests(List<TestCase> tests) throws IOException, InterruptedException {
    var visitor = new TestCaseVisitor();

    try (var out = new BufferedWriter(new FileWriter(executionPath.toFile()))) {
      if (type == FileType.MAIN) {
        out.write("pub mod testify_monitor;\n");
      }

      var content = Files.readString(originalPath);
      out.write(content);
      out.write("\n");

      out.write("#[cfg(test)]\n");
      out.write("mod testify_tests {\n");

      var testCode = tests.stream()
          .map(testCase -> testCase.visit(visitor))
          .collect(Collectors.joining("\n\n"));

      out.write(testCode);
      out.write("\n}");
    }
  }
}
