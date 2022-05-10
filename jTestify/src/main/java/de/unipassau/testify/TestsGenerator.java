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
import de.unipassau.testify.exec.Timer;
import de.unipassau.testify.generator.OffspringGeneratorImpl;
import de.unipassau.testify.hir.TyCtxt;
import de.unipassau.testify.json.JSONParser;
import de.unipassau.testify.metaheuristics.chromosome.ChromosomeGenerator;
import de.unipassau.testify.metaheuristics.chromosome.FixedSizePopulationGenerator;
import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import de.unipassau.testify.mir.MirAnalysis;
import de.unipassau.testify.source.Crate;
import de.unipassau.testify.test_case.gen.AllMethodTestCaseGenerator;
import de.unipassau.testify.test_case.gen.SeededTestCaseGenerator;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.gen.RandomTestCaseGenerator;
import de.unipassau.testify.test_case.UncoveredObjectives;
import de.unipassau.testify.test_case.operators.DefaultMutation;
import de.unipassau.testify.test_case.operators.RankSelection;
import de.unipassau.testify.test_case.operators.SinglePointFixedCrossover;
import de.unipassau.testify.test_case.seed.SeedOptions;
import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.List;
import java.util.Set;
import java.util.concurrent.TimeUnit;
import java.util.stream.Collectors;
import java.util.stream.IntStream;
import java.util.stream.Stream;

public class TestsGenerator {

  public static void runMOSA(CLI cli) throws IOException, InterruptedException {
    var crate = Crate.load(cli);

    var hirLog = new File(cli.getHirPath());
    var hirJson = Files.readString(hirLog.toPath());
    var hir = new TyCtxt(JSONParser.parse(hirJson));
    var mir = new MirAnalysis<TestCase>(cli.getMirPath());

    Set<MinimizingFitnessFunction<TestCase>> objectives = mir.targets();

    var seedOptions = SeedOptions.builder()
        .useConstantPool(cli.seedConstantPool())
        .initialRandomPopulation(cli.seedRandomPopulation())
        .useAllMethods(cli.seedMethods())
        .build();
    var svd = new SVDImpl<>(objectives);
    var pareto = new Pareto<TestCase>();
    var fnds = new FNDSImpl<>(pareto);
    var preferenceSorter = new PreferenceSorterImpl<>(objectives, fnds);

    var mutation = new DefaultMutation();
    var crossover = new SinglePointFixedCrossover();
    var selection = new RankSelection<>(objectives, svd, preferenceSorter);

    ChromosomeGenerator<TestCase> chromosomeGenerator;
    if (seedOptions.useAllMethods()) {
      chromosomeGenerator = new SeededTestCaseGenerator(hir, mir, mutation, crossover,
          seedOptions);
    } else {
      chromosomeGenerator = new RandomTestCaseGenerator(hir, mir, mutation, crossover);
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

    var seedOptions = SeedOptions.builder()
        .useConstantPool(cli.seedConstantPool())
        .initialRandomPopulation(cli.seedRandomPopulation())
        .useAllMethods(cli.seedMethods())
        .build();
    var svd = new SVDImpl<>(objectives);
    var pareto = new Pareto<TestCase>();
    var fnds = new FNDSImpl<>(pareto);
    var preferenceSorter = new PreferenceSorterImpl<>(objectives, fnds);

    var mutation = new DefaultMutation();
    var crossover = new SinglePointFixedCrossover();
    var selection = new RankSelection<>(objectives, svd, preferenceSorter);
//    ChromosomeGenerator<TestCase> chromosomeGenerator = new SeededTestCaseGenerator(hir, mir,
//        mutation, crossover, seedOptions);
    var chromosomeGenerator = new RandomTestCaseGenerator(hir, mir, mutation, crossover);
    var populationGenerator = new FixedSizePopulationGenerator<>(
        chromosomeGenerator, POPULATION_SIZE);

    var uncoveredObjectives = new UncoveredObjectives<>(objectives);
    var offspringGenerator = new OffspringGeneratorImpl(selection, uncoveredObjectives);

    var archive = new DefaultArchive<>(objectives);
    var output = new Output<TestCase>(cli.getCrateName(), cli.getCrateRoot());

    var timer = new Timer();
    timer.start();
    List<TestCase> initialPopulation = populationGenerator.get();
    if (seedOptions.initialRandomPopulation()) {
      initialPopulation = new ArrayList<>();
      int randomGenerations = Math.max((int) (GENERATIONS * 0.2), 2);
      var randomTestCaseGenerator = new RandomTestCaseGenerator(hir, mir, mutation, crossover);
      IntStream.range(0, randomGenerations).mapToObj(i -> randomTestCaseGenerator.get())
          .forEach(initialPopulation::add);
    }

    if (seedOptions.useAllMethods()) {
      var methodsGenerator = new AllMethodTestCaseGenerator(hir, mir, mutation, crossover, seedOptions);
      var additionalPopulation = Stream.generate(methodsGenerator).limit(hir.getCallables().size()).collect(Collectors.toList());
      initialPopulation.addAll(additionalPopulation);
    }

    System.out.printf("-- Initial population has been generated. Took %ds%n", TimeUnit.MILLISECONDS.toSeconds(timer.end()));

    var mosa = DynaMOSA.<TestCase>builder().maxGenerations(GENERATIONS)
        .populationSize(POPULATION_SIZE)
        .populationGenerator(populationGenerator)
        .offspringGenerator(offspringGenerator)
        .preferenceSorter(preferenceSorter)
        .archive(archive)
        .svd(svd)
        .container(crate)
        .mir(mir)
        .initialPopulation(initialPopulation)
        .output(output).build();

    var solutions = mosa.findSolution();

    crate.addAll(solutions);
  }

  public static void runRandomSearch(CLI cli) throws IOException, InterruptedException {
    var crate = Crate.load(cli);

    var hirLog = new File(cli.getHirPath());
    var hirJson = Files.readString(hirLog.toPath());
    var hir = new TyCtxt(JSONParser.parse(hirJson));
    var mir = new MirAnalysis<TestCase>(cli.getMirPath());

    var mutation = new DefaultMutation();
    var crossover = new SinglePointFixedCrossover();
    var chromosomeGenerator = new RandomTestCaseGenerator(hir, mir, mutation, crossover);

    Set<MinimizingFitnessFunction<TestCase>> objectives = mir.targets();

    var archive = new DefaultArchive<>(objectives);
    var rs = new RandomSearch<>(GENERATIONS * POPULATION_SIZE, chromosomeGenerator, archive, crate);
    var solutions = rs.findSolution();
    crate.addAll(solutions);
  }
}
