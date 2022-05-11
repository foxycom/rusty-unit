package de.unipassau.rustyunit.source;


import de.unipassau.rustyunit.Constants;
import de.unipassau.rustyunit.test_case.TestCase;
import de.unipassau.rustyunit.test_case.visitor.TestCaseVisitor;
import java.io.BufferedWriter;
import java.io.FileWriter;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashSet;
import java.util.List;
import java.util.stream.Collectors;

public class SourceFile {

  private Path originalPath;
  private Path executionPath;
  private FileType type;

  private int size;

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
        //out.write("#![feature(no_coverage)]\n");
        var content = Files.readString(originalPath);
        var lines = content.split(System.lineSeparator());
        size = lines.length;
        out.write(content);
      }
    }
  }

  public void addTests(List<TestCase> tests) throws IOException, InterruptedException {
    var visitor = new TestCaseVisitor();

    var usedTraitNames = new HashSet<String>();
    tests.forEach(t -> usedTraitNames.addAll(t.getUsedTraitNames()));
    var imports = usedTraitNames.stream().map(tn -> String.format("\tuse %s;", tn))
        .collect(Collectors.joining("\n"));

    try (var out = new BufferedWriter(new FileWriter(executionPath.toFile()))) {
      var content = Files.readString(originalPath);
      out.write(content);
      out.write("\n");

      out.write("#[cfg(test)]\n");
      out.write(String.format("mod %s {\n", Constants.TEST_MOD_NAME));
      out.write("\tuse crate::*;\n");
      out.write(String.format("%s\n", imports));

      //out.write("\tuse ntest::timeout;\n");

      var testCode = tests.stream()
          .map(testCase -> testCase.visit(visitor))
          .collect(Collectors.joining("\n\n"));

      out.write(testCode);
      out.write("\n}");
    }
  }
}
