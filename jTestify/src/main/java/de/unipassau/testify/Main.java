package de.unipassau.testify;

import com.lexicalscope.jewel.cli.CliFactory;
import com.lexicalscope.jewel.cli.Option;
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
import de.unipassau.testify.source.Crate;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.TestCaseGenerator;
import de.unipassau.testify.test_case.UncoveredObjectives;
import de.unipassau.testify.test_case.fitness.RandomFitness;
import de.unipassau.testify.test_case.operators.BasicMutation;
import de.unipassau.testify.test_case.operators.RankSelection;
import de.unipassau.testify.test_case.operators.SinglePointFixedCrossover;
import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Collections;
import java.util.List;
import java.util.stream.Collectors;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class Main {

  private static final Logger logger = LoggerFactory.getLogger(Main.class);

  public interface CLI {

    @Option(shortName = "c", longName = "crate")
    String getCrateRoot();

    @Option(shortName = "m")
    List<String> getMainFiles();

    @Option(shortName = "n")
    String getCrateName();
  }

  public static void main(String[] args) throws IOException, InterruptedException {
    var cli = CliFactory.parseArguments(CLI.class, args);
    var crate = Crate.parse(Paths.get(cli.getCrateRoot()),
        cli.getMainFiles().stream().map(Path::of).toList(), cli.getCrateName());

    var file = new File("/Users/tim/Documents/master-thesis/testify/log/hir.json");
    var json = Files.readString(file.toPath());
    var hirAnalysis = new HirAnalysis(JSONParser.parse(json));

    /*List<MinimizingFitnessFunction<TestCase>> objectives = MirAnalysis.getBranches().stream()
        .map(Branch::getGlobalId).map(RandomFitness::new).collect(Collectors.toList());*/

    List<MinimizingFitnessFunction<TestCase>> objectives = Collections.emptyList();

    var svd = new SVDImpl<>(objectives);
    var pareto = new Pareto<TestCase>();
    var fnds = new FNDSImpl<>(pareto);
    var preferenceSorter = new PreferenceSorterImpl<>(objectives, fnds);

    var mutation = new BasicMutation();
    var crossover = new SinglePointFixedCrossover();
    var selection = new RankSelection<>(objectives, svd, preferenceSorter);
    var populationGenerator = new FixedSizePopulationGenerator<>(
        new TestCaseGenerator(hirAnalysis, mutation, crossover), 20);

    //var population = populationGenerator.get();

    //crate.addTests(population);

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
        svd,
        crate
    );

    var solutions = mosa.findSolution();

  }


}
