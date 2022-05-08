package de.unipassau.testify;

import static de.unipassau.testify.Constants.GENERATIONS;
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
import de.unipassau.testify.metaheuristics.chromosome.ChromosomeGenerator;
import de.unipassau.testify.metaheuristics.chromosome.FixedSizePopulationGenerator;
import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import de.unipassau.testify.mir.MirAnalysis;
import de.unipassau.testify.source.Crate;
import de.unipassau.testify.test_case.CallableSelector;
import de.unipassau.testify.test_case.SeededTestCaseGenerator;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.RandomTestCaseGenerator;
import de.unipassau.testify.test_case.UncoveredObjectives;
import de.unipassau.testify.test_case.operators.DefaultMutation;
import de.unipassau.testify.test_case.operators.RankSelection;
import de.unipassau.testify.test_case.operators.SinglePointFixedCrossover;
import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.util.Set;

public class TestsGenerator {

  public static void runMOSA(CLI cli) throws IOException, InterruptedException {
    var crate = Crate.load(cli);

    var hirLog = new File(cli.getHirPath());
    var hirJson = Files.readString(hirLog.toPath());
    var hir = new TyCtxt(JSONParser.parse(hirJson));
    var mir = new MirAnalysis<TestCase>(cli.getMirPath());

    Set<MinimizingFitnessFunction<TestCase>> objectives = mir.targets();

    var svd = new SVDImpl<>(objectives);
    var pareto = new Pareto<TestCase>();
    var fnds = new FNDSImpl<>(pareto);
    var preferenceSorter = new PreferenceSorterImpl<>(objectives, fnds);

    var callableSelector = new CallableSelector();
    var mutation = new DefaultMutation();
    var crossover = new SinglePointFixedCrossover();
    var selection = new RankSelection<>(objectives, svd, preferenceSorter);

    ChromosomeGenerator<TestCase> chromosomeGenerator;
    if (cli.seedMethods()) {
      chromosomeGenerator = new SeededTestCaseGenerator(hir, mir, mutation, crossover,
          callableSelector);
    } else {
      chromosomeGenerator = new RandomTestCaseGenerator(hir, mir, mutation, crossover,
          callableSelector);
    }
    var populationGenerator = new FixedSizePopulationGenerator<>(chromosomeGenerator,
        POPULATION_SIZE);

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
    var crate = Crate.load(cli);

    var hirLog = new File(cli.getHirPath());
    var hirJson = Files.readString(hirLog.toPath());
    var hir = new TyCtxt(JSONParser.parse(hirJson));
    var mir = new MirAnalysis<TestCase>(cli.getMirPath());

    Set<MinimizingFitnessFunction<TestCase>> objectives = mir.targets();

    var callableSelector = new CallableSelector();
    var svd = new SVDImpl<>(objectives);
    var pareto = new Pareto<TestCase>();
    var fnds = new FNDSImpl<>(pareto);
    var preferenceSorter = new PreferenceSorterImpl<>(objectives, fnds);

    var mutation = new DefaultMutation();
    var crossover = new SinglePointFixedCrossover();
    var selection = new RankSelection<>(objectives, svd, preferenceSorter);
    ChromosomeGenerator<TestCase> chromosomeGenerator;
    if (cli.seedMethods()) {
      chromosomeGenerator = new SeededTestCaseGenerator(hir, mir, mutation, crossover, callableSelector);
    } else {
      chromosomeGenerator = new RandomTestCaseGenerator(hir, mir, mutation, crossover, callableSelector);
    }
    var populationGenerator = new FixedSizePopulationGenerator<>(
        chromosomeGenerator, POPULATION_SIZE);

    var uncoveredObjectives = new UncoveredObjectives<>(objectives);
    var offspringGenerator = new OffspringGeneratorImpl(selection, uncoveredObjectives);

    var archive = new DefaultArchive<>(objectives);

    var output = new Output<TestCase>(cli.getCrateName(), cli.getCrateRoot());

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
    var crate = Crate.load(cli);

    var hirLog = new File(cli.getHirPath());
    var hirJson = Files.readString(hirLog.toPath());
    var hir = new TyCtxt(JSONParser.parse(hirJson));
    var mir = new MirAnalysis<TestCase>(cli.getMirPath());

    var callableSelector = new CallableSelector();
    var mutation = new DefaultMutation();
    var crossover = new SinglePointFixedCrossover();
    var chromosomeGenerator = new RandomTestCaseGenerator(hir, mir, mutation, crossover,
        callableSelector);
    var populationGenerator = new FixedSizePopulationGenerator<>(chromosomeGenerator,
        POPULATION_SIZE);

    Set<MinimizingFitnessFunction<TestCase>> objectives = mir.targets();

    var archive = new DefaultArchive<>(objectives);
    var rs = new RandomSearch<>(GENERATIONS, populationGenerator, archive, crate);

    var solutions = rs.findSolution();

    crate.addAll(solutions);
  }
}
