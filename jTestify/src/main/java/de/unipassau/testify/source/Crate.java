package de.unipassau.testify.source;

import de.unipassau.testify.source.SourceFile.FileType;
import java.io.File;
import java.io.FileFilter;
import java.io.FilenameFilter;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
import java.util.Locale;

public class Crate {

  private Path originalRoot;
  private Path executionRoot;
  private List<SourceFile> sourceFiles;

  public static Crate parse(Path root) throws IOException {
    var executionRoot = getExecutionRoot(root);

    var sourceFiles = Files.walk(root).filter(p -> !Files.isDirectory(p))
        .filter(p -> p.toString().toLowerCase(Locale.ROOT).endsWith(".rs"))
        .map(p -> {
          var executionPath = p.relativize(root).resolve(executionRoot);
          return new SourceFile(p, executionPath, FileType.SOURCE_CODE);
        }).toList();

    return new Crate(root, getExecutionRoot(root), sourceFiles);
  }

  private static Path getExecutionRoot(Path root) {
    return Paths.get("/Users/tim/Documents/master-thesis/evaluation/current");
  }

  private Crate(Path originalRoot, Path executionRoot,
      List<SourceFile> sourceFiles) {
    this.originalRoot = originalRoot;
    this.executionRoot = executionRoot;
    this.sourceFiles = sourceFiles;
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
}
