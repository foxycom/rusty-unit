package de.unipassau.testify;

import de.unipassau.testify.algorithm.ArchiveImpl;
import de.unipassau.testify.algorithm.FNDSImpl;
import de.unipassau.testify.algorithm.MOSA;
import de.unipassau.testify.algorithm.Pareto;
import de.unipassau.testify.algorithm.PreferenceSorterImpl;
import de.unipassau.testify.algorithm.SVDImpl;
import de.unipassau.testify.generator.OffspringGeneratorImpl;
import de.unipassau.testify.hir.HirAnalysis;
import de.unipassau.testify.json.JSONParser;
import de.unipassau.testify.metaheuristics.chromosome.FixedSizePopulationGenerator;
import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import de.unipassau.testify.mir.Branch;
import de.unipassau.testify.mir.MirAnalysis;
import de.unipassau.testify.test_case.Fitness;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.TestCaseGenerator;
import de.unipassau.testify.test_case.TestCaseVisitor;
import de.unipassau.testify.test_case.UncoveredObjectives;
import de.unipassau.testify.test_case.operators.BasicMutation;
import de.unipassau.testify.test_case.operators.RankSelection;
import de.unipassau.testify.test_case.operators.SinglePointFixedCrossover;
import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.util.List;
import java.util.stream.Collectors;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class Main {

  private static final Logger logger = LoggerFactory.getLogger(Main.class);

  public static void mains(String[] args) throws IOException {
    var file = new File("/Users/tim/Documents/master-thesis/testify/log/hir.json");
    var json = Files.readString(file.toPath());
    var hirAnalysis = new HirAnalysis(JSONParser.parse(json));
    var mutation = new BasicMutation();
    var crossover = new SinglePointFixedCrossover();

    var populationGenerator = new FixedSizePopulationGenerator<>(
        new TestCaseGenerator(hirAnalysis, mutation, crossover), 1);

    var population = populationGenerator.get();

    var visitor = new TestCaseVisitor();
    System.out.println(population.get(0).visit(visitor));
  }

  public static void main(String[] args) throws IOException {
    var file = new File("/Users/tim/Documents/master-thesis/testify/log/hir.json");
    var json = Files.readString(file.toPath());
    var hirAnalysis = new HirAnalysis(JSONParser.parse(json));

    List<MinimizingFitnessFunction<TestCase>> objectives = MirAnalysis.getBranches().stream()
        .map(Branch::getId).map(Fitness::new).collect(Collectors.toList());

    var svd = new SVDImpl<>(objectives);
    var pareto = new Pareto<TestCase>();
    var fnds = new FNDSImpl<>(pareto);
    var preferenceSorter = new PreferenceSorterImpl<>(objectives, fnds);

    var mutation = new BasicMutation();
    var crossover = new SinglePointFixedCrossover();
    var selection = new RankSelection<>(objectives, svd, preferenceSorter);
    var populationGenerator = new FixedSizePopulationGenerator<>(
        new TestCaseGenerator(hirAnalysis, mutation, crossover), 20);

    var uncoveredObjectives = new UncoveredObjectives<>(objectives);
    var offspringGenerator = new OffspringGeneratorImpl(selection, uncoveredObjectives);

    var archive = new ArchiveImpl<>(objectives);

    var mosa = new MOSA<>(
        10,
        20,
        populationGenerator,
        offspringGenerator,
        preferenceSorter,
        archive,
        svd
    );

    var solutions = mosa.findSolution();
  }


}
