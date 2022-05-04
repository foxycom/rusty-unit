package de.unipassau.testify.source;

import com.google.common.base.Charsets;
import com.google.common.base.Preconditions;
import com.google.common.io.FileWriteMode;
import de.unipassau.testify.Main.CLI;
import de.unipassau.testify.exec.ChromosomeExecutor;
import de.unipassau.testify.exec.LLVMCoverage;
import de.unipassau.testify.exec.TestCaseRunner;
import de.unipassau.testify.source.SourceFile.FileType;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.util.Rnd;
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
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class Crate implements ChromosomeContainer<TestCase> {

    private static final Logger logger = LoggerFactory.getLogger(ChromosomeContainer.class);

    private final Path originalRoot;
    private final Path executionRoot;
    private final List<SourceFile> sourceFiles;
    private final ChromosomeExecutor<TestCase> executor;
    private final String crateName;
    private List<TestCase> testCases;

    public static Crate load(CLI cli)
          throws IOException, InterruptedException {
        var root = Paths.get(cli.getCrateRoot());
        var executionRoot = getExecutionRoot(Paths.get(cli.getCrateRoot()));

        var sourceFiles = Files.walk(root)
              .filter(p -> !Files.isDirectory(p))
              .filter(p -> p.toString().toLowerCase(Locale.ROOT).endsWith(".rs"))
              .filter(p -> p.getParent().endsWith("src"))
              .map(p -> {
                  var relativePath = root.relativize(p);
                  var executionPath = executionRoot.resolve(relativePath);
                  if (cli.getMainFiles().stream().anyMatch(p::endsWith)) {
                      return new SourceFile(p, executionPath, FileType.MAIN);
                  } else {
                      return new SourceFile(p, executionPath, FileType.SOURCE_CODE);
                  }
              }).toList();

        return new Crate(cli.getCrateName(), root, getExecutionRoot(root), sourceFiles,
              new TestCaseRunner(cli, executionRoot.toString()));
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
        FileUtils.deleteDirectory(executionRoot.toFile());
        FileUtils.copyDirectory(originalRoot.toFile(), executionRoot.toFile());
        for (SourceFile sourceFile : sourceFiles) {
            sourceFile.onCopied();
        }

        // Add redis dependency
//    var cargoToml = findCargoToml();
//    addDependencies(cargoToml);
    }

    private void addDependencies(Path cargoToml) throws IOException {
        var sink = com.google.common.io.Files.asCharSink(cargoToml.toFile(), Charsets.UTF_8,
              FileWriteMode.APPEND);
        sink.write("\n[dependencies.redis]\nversion = \"*\"\n");
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
              .filter(f -> !f.getExecutionPath().toString().endsWith("lib.rs")
                    && !f.getExecutionPath().toString().endsWith("monitor.rs")).toList();

        testCases.forEach(testCase -> {
            var filePathBinding = testCase.getFilePathBinding()
                  .orElseGet(
                        () -> executionRoot.relativize(Rnd.choice(allowedFiles).getExecutionPath())
                              .toString());
            sorted.putIfAbsent(filePathBinding, new ArrayList<>());
            sorted.get(filePathBinding).add(testCase);
            testCase.metadata().setFilePath(filePathBinding);
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

    @Override
    public String getPath() {
        return executionRoot.toString();
    }

    @Override
    public String getName() {
        return crateName;
    }

}