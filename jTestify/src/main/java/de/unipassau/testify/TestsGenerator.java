package de.unipassau.testify;

import static de.unipassau.testify.Constants.GENERATIONS;
import static de.unipassau.testify.Constants.HIR_LOG_PATH;
import static de.unipassau.testify.Constants.POPULATION_SIZE;

import de.unipassau.testify.Main.CLI;
import de.unipassau.testify.algorithm.DefaultArchive;
import de.unipassau.testify.algorithm.FNDSImpl;
import de.unipassau.testify.algorithm.Pareto;
import de.unipassau.testify.algorithm.PreferenceSorterImpl;
import de.unipassau.testify.algorithm.SVDImpl;
import de.unipassau.testify.algorithm.dynamosa.DynaMOSA;
import de.unipassau.testify.algorithm.mosa.MOSA;
import de.unipassau.testify.algorithm.random.RandomSearch;
import de.unipassau.testify.exec.Output;
import de.unipassau.testify.generator.OffspringGeneratorImpl;
import de.unipassau.testify.hir.TyCtxt;
import de.unipassau.testify.json.JSONParser;
import de.unipassau.testify.metaheuristics.chromosome.FixedSizePopulationGenerator;
import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import de.unipassau.testify.mir.MirAnalysis;
import de.unipassau.testify.source.Crate;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.TestCaseGenerator;
import de.unipassau.testify.test_case.UncoveredObjectives;
import de.unipassau.testify.test_case.operators.DefaultMutation;
import de.unipassau.testify.test_case.operators.RankSelection;
import de.unipassau.testify.test_case.operators.SinglePointFixedCrossover;
import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Set;

public class TestsGenerator {
  public static void runMOSA(CLI cli) throws IOException, InterruptedException {
    var crate = Crate.parse(Paths.get(cli.getCrateRoot()),
        cli.getMainFiles().stream().map(Path::of).toList(), cli.getCrateName());

    // TODO: 12.02.22 run instrumentation of the crate

    var hirLog = new File(cli.getHirPath());
    var hirJson = Files.readString(hirLog.toPath());
    var hir = new TyCtxt(JSONParser.parse(hirJson));
    var mir = new MirAnalysis<TestCase>(cli.getMirPath());

    Set<MinimizingFitnessFunction<TestCase>> objectives = mir.targets();

    var svd = new SVDImpl<>(objectives);
    var pareto = new Pareto<TestCase>();
    var fnds = new FNDSImpl<>(pareto);
    var preferenceSorter = new PreferenceSorterImpl<>(objectives, fnds);

    var mutation = new DefaultMutation();
    var crossover = new SinglePointFixedCrossover();
    var selection = new RankSelection<>(objectives, svd, preferenceSorter);
    var populationGenerator = new FixedSizePopulationGenerator<>(
        new TestCaseGenerator(hir, mir, mutation, crossover), POPULATION_SIZE);

    var uncoveredObjectives = new UncoveredObjectives<>(objectives);
    var offspringGenerator = new OffspringGeneratorImpl(selection, uncoveredObjectives);

    var archive = new DefaultArchive<>(objectives);

    var mosa = new MOSA<>(
        GENERATIONS,
        POPULATION_SIZE,
        populationGenerator,
        offspringGenerator,
        preferenceSorter,
        archive,
        svd,
        crate
    );

    var solutions = mosa.findSolution();

    crate.addAll(solutions);
  }

  public static void runDynaMOSA(CLI cli) throws IOException, InterruptedException {
    var crate = Crate.parse(Paths.get(cli.getCrateRoot()),
        cli.getMainFiles().stream().map(Path::of).toList(), cli.getCrateName());

    // TODO: 12.02.22 run instrumentation of the crate

    var hirLog = new File(cli.getHirPath());
    var hirJson = Files.readString(hirLog.toPath());
    var hir = new TyCtxt(JSONParser.parse(hirJson));
    var mir = new MirAnalysis<TestCase>(cli.getMirPath());

    Set<MinimizingFitnessFunction<TestCase>> objectives = mir.targets();

    var svd = new SVDImpl<>(objectives);
    var pareto = new Pareto<TestCase>();
    var fnds = new FNDSImpl<>(pareto);
    var preferenceSorter = new PreferenceSorterImpl<>(objectives, fnds);

    var mutation = new DefaultMutation();
    var crossover = new SinglePointFixedCrossover();
    var selection = new RankSelection<>(objectives, svd, preferenceSorter);
    var populationGenerator = new FixedSizePopulationGenerator<>(
        new TestCaseGenerator(hir, mir, mutation, crossover), POPULATION_SIZE);

    var uncoveredObjectives = new UncoveredObjectives<>(objectives);
    var offspringGenerator = new OffspringGeneratorImpl(selection, uncoveredObjectives);

    var archive = new DefaultArchive<>(objectives);

    var output = new Output<TestCase>(cli.getCrateName());

    var mosa = new DynaMOSA<>(
        GENERATIONS,
        POPULATION_SIZE,
        populationGenerator,
        offspringGenerator,
        preferenceSorter,
        archive,
        svd,
        crate,
        mir,
        output
    );

    var solutions = mosa.findSolution();

    crate.addAll(solutions);
  }

  public static void runRandomSearch(CLI cli) throws IOException, InterruptedException {
    var crate = Crate.parse(Paths.get(cli.getCrateRoot()),
          cli.getMainFiles().stream().map(Path::of).toList(), cli.getCrateName());

    var hirLog = new File(cli.getHirPath());
    var hirJson = Files.readString(hirLog.toPath());
    var hir = new TyCtxt(JSONParser.parse(hirJson));
    var mir = new MirAnalysis<TestCase>(cli.getMirPath());

    var mutation = new DefaultMutation();
    var crossover = new SinglePointFixedCrossover();
    var chromosomeGenerator = new TestCaseGenerator(hir, mir, mutation, crossover);
    var populationGenerator = new FixedSizePopulationGenerator<>(chromosomeGenerator, POPULATION_SIZE);

    Set<MinimizingFitnessFunction<TestCase>> objectives = mir.targets();

    var archive = new DefaultArchive<>(objectives);
    var rs = new RandomSearch<>(GENERATIONS, populationGenerator, archive, crate);

    var solutions = rs.findSolution();

    crate.addAll(solutions);
  }
}
