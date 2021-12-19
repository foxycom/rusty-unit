package de.unipassau.testify.source;

import de.unipassau.testify.test_case.TestCase;
import java.nio.file.Path;
import java.util.List;

public class SourceFile {
  private Path originalPath;
  private Path executionPath;
  private FileType type;

  public enum FileType {
    EXECUTABLE, LIBRARY, SOURCE_CODE;
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

  public void addTests(List<TestCase> tests) {
    throw new RuntimeException("Not implemented yet");
  }
}
