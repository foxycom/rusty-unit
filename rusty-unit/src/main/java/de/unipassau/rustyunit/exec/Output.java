package de.unipassau.rustyunit.exec;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.google.gson.JsonPrimitive;
import de.unipassau.rustyunit.Listener;
import de.unipassau.rustyunit.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.rustyunit.test_case.visitor.TestCaseVisitor;
import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.List;

public class Output<C extends AbstractTestCaseChromosome<C>> implements Listener<C> {
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

  public void addIntermediateResults(int generation, int coveredTargets, double ratio, int numberOfTests, double avgTestLength) {


  }

  @Override
  public void onExecuted(Status status) {
    var dir = getDirectoryForGeneration(status.generation);

    try (var writer = Files.newBufferedWriter(Paths.get(dir.getAbsolutePath(), "coverage.json"))) {
      var content = new JsonObject();
      content.add("absoluteCoverage", new JsonPrimitive(status.coveredTargets));
      content.add("relativeCoverage", new JsonPrimitive(status.coverage));
      content.add("numberOfTests", new JsonPrimitive(status.tests));
      content.add("avgTestLength", new JsonPrimitive(status.avgLength));

      writer.write(content.toString());
      writer.flush();
    } catch (IOException e) {
      throw new RuntimeException(e);
    }
  }

  @Override
  public void onPopulation(int generation, List<C> population) {
    var dir = getDirectoryForGeneration(generation);
    var created = dir.mkdirs();
    if (!dir.exists()) {
      throw new RuntimeException("Could not create directory for generation " + dir.getAbsolutePath());
    }

    for (var testCase : population) {
      try (var writer = Files.newBufferedWriter(Paths.get(dir.getAbsolutePath(), String.format("%d.json", testCase.getId())))) {
        writer.write(asJson(testCase));
        writer.flush();
      } catch (IOException e) {
        throw new RuntimeException(e);
      }
    }
  }
}
