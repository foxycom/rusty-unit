package de.unipassau.testify.exec;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.google.gson.JsonPrimitive;
import de.unipassau.testify.Constants;
import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.test_case.visitor.TestCaseVisitor;
import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.List;

public class Output<C extends AbstractTestCaseChromosome<C>> {
  private final File directory;

  public Output(String crateName, String crateRoot) {
    var dirName = String.format("%s-%d", crateName, System.currentTimeMillis());
    var path = Paths.get(crateRoot, "generated-tests", dirName);
    directory = path.toFile();
    var result = directory.mkdirs();
    if (!result) {
      throw new RuntimeException("Could create output directory " + path);
    }
  }

  public void addPopulation(int generation, List<C> testCases) {
    var dir = getDirectoryForGeneration(generation);
    var created = dir.mkdirs();
    if (!dir.exists()) {
      throw new RuntimeException("Could not create directory for generation " + dir.getAbsolutePath());
    }

    for (var testCase : testCases) {
      try {
        var writer = Files.newBufferedWriter(Paths.get(dir.getAbsolutePath(), String.format("%d.json", testCase.getId())));
        writer.write(asJson(testCase));
        writer.flush();
      } catch (IOException e) {
        throw new RuntimeException(e);
      }
    }
  }

  private String asJson(C testCase) {
    var visitor = new TestCaseVisitor();

    var jo = new JsonObject();
    jo.add("id", new JsonPrimitive(testCase.getId()));
    jo.add("filePath", new JsonPrimitive(testCase.metadata().filePath()));

    var statementsArray = new JsonArray(testCase.size());
    for (var stmt : testCase.getStatements()) {
      statementsArray.add(visitor.visitStatement(stmt));
    }

    jo.add("statements", statementsArray);
    return jo.toString();
  }

  private File getDirectoryForGeneration(int generation) {
    return Paths.get(directory.getAbsolutePath(), String.format("gen-%d", generation)).toFile();
  }

  public void addCoveredTargets(int generation, int coveredTargets, int overallNumberOfTargets) {
    var dir = getDirectoryForGeneration(generation);

    try {
      var writer = Files.newBufferedWriter(Paths.get(dir.getAbsolutePath(), "coverage.json"));

      var content = new JsonObject();
      content.add("absoluteCoverage", new JsonPrimitive(coveredTargets));
      content.add("relativeCoverage", new JsonPrimitive(((double) coveredTargets) / overallNumberOfTargets));

      writer.write(content.toString());
      writer.flush();
    } catch (IOException e) {
      throw new RuntimeException(e);
    }

  }
}
