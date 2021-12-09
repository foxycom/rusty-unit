package de.unipassau.testify;

import de.unipassau.testify.hir.HirAnalysis;
import de.unipassau.testify.json.JSONParser;
import de.unipassau.testify.metaheuristics.chromosome.FixedSizePopulationGenerator;
import de.unipassau.testify.test_case.TestCaseGenerator;
import de.unipassau.testify.test_case.TestCaseVisitor;
import de.unipassau.testify.test_case.operators.BasicMutation;
import de.unipassau.testify.test_case.operators.SinglePointCrossover;
import java.io.File;
import java.io.IOException;
import java.nio.file.Files;

public class Main {

  public static void main(String[] args) throws IOException {
    var file = new File("/Users/tim/Documents/master-thesis/testify/log/hir.json");
    var json = Files.readString(file.toPath());
    var hirAnalysis = new HirAnalysis(JSONParser.parse(json));
    var mutation = new BasicMutation();
    var crossover = new SinglePointCrossover();

    var populationGenerator = new FixedSizePopulationGenerator<>(
        new TestCaseGenerator(hirAnalysis, mutation, crossover), 1);

    var population = populationGenerator.get();

    var visitor = new TestCaseVisitor();
    System.out.println(population.get(0).visit(visitor));
  }


}
