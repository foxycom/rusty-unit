package de.unipassau.rustyunit;

import static de.unipassau.rustyunit.Constants.GENERATIONS;
import static de.unipassau.rustyunit.Constants.POPULATION_SIZE;

import de.unipassau.rustyunit.Main.CLI;
import de.unipassau.rustyunit.algorithm.DefaultArchive;
import de.unipassau.rustyunit.algorithm.FNDSImpl;
import de.unipassau.rustyunit.algorithm.Pareto;
import de.unipassau.rustyunit.algorithm.PreferenceSorterImpl;
import de.unipassau.rustyunit.algorithm.SVDImpl;
import de.unipassau.rustyunit.algorithm.dynamosa.DynaMOSA;
import de.unipassau.rustyunit.algorithm.mosa.MOSA;
import de.unipassau.rustyunit.algorithm.random.RandomSearch;
import de.unipassau.rustyunit.exec.Output;
import de.unipassau.rustyunit.exec.Timer;
import de.unipassau.rustyunit.generator.OffspringGeneratorImpl;
import de.unipassau.rustyunit.hir.TyCtxt;
import de.unipassau.rustyunit.json.JSONParser;
import de.unipassau.rustyunit.metaheuristics.chromosome.ChromosomeGenerator;
import de.unipassau.rustyunit.metaheuristics.chromosome.FixedSizePopulationGenerator;
import de.unipassau.rustyunit.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import de.unipassau.rustyunit.mir.MirAnalysis;
import de.unipassau.rustyunit.source.Crate;
import de.unipassau.rustyunit.test_case.gen.AllMethodTestCaseGenerator;
import de.unipassau.rustyunit.test_case.gen.RandomSearchTestCaseGenerator;
import de.unipassau.rustyunit.test_case.gen.SeededTestCaseGenerator;
import de.unipassau.rustyunit.test_case.TestCase;
import de.unipassau.rustyunit.test_case.gen.RandomTestCaseGenerator;
import de.unipassau.rustyunit.test_case.UncoveredObjectives;
import de.unipassau.rustyunit.test_case.operators.DefaultMutation;
import de.unipassau.rustyunit.test_case.operators.DummyCrossover;
import de.unipassau.rustyunit.test_case.operators.DummyMutation;
import de.unipassau.rustyunit.test_case.operators.RankSelection;
import de.unipassau.rustyunit.test_case.operators.SinglePointFixedCrossover;
import de.unipassau.rustyunit.test_case.seed.SeedOptions;
import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Set;
import java.util.concurrent.TimeUnit;
import java.util.stream.IntStream;
import java.util.stream.Stream;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class TestsGenerator {

  private static final Logger logger = LoggerFactory.getLogger(TestsGenerator.class);

  public static void runMOSA(CLI cli) throws IOException, InterruptedException {
    var crate = Crate.load(cli);

    var hirLog = new File(cli.getHirPath());
    var hirJson = Files.readString(hirLog.toPath());
    var hir = new TyCtxt(JSONParser.parse(hirJson, cli.parseTraits()));
    var mir = new MirAnalysis<TestCase>(cli.getMirPath());

    Set<MinimizingFitnessFunction<TestCase>> objectives = mir.targets();

    var svd = new SVDImpl<>(objectives);
    var pareto = new Pareto<TestCase>();
    var fnds = new FNDSImpl<>(pareto);
    var preferenceSorter = new PreferenceSorterImpl<>(objectives, fnds);

    var mutation = new DefaultMutation();
    var crossover = new SinglePointFixedCrossover();
    var selection = new RankSelection<>(objectives, svd, preferenceSorter);

    ChromosomeGenerator<TestCase> chromosomeGenerator;
    if (SeedOptions.any()) {
      chromosomeGenerator = new SeededTestCaseGenerator(hir, mir, mutation, crossover);
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
    var hir = new TyCtxt(JSONParser.parse(hirJson, cli.parseTraits()));
    var mir = new MirAnalysis<TestCase>(cli.getMirPath());

    Set<MinimizingFitnessFunction<TestCase>> objectives = mir.targets();

    var svd = new SVDImpl<>(objectives);
    var pareto = new Pareto<TestCase>();
    var fnds = new FNDSImpl<>(pareto);
    var preferenceSorter = new PreferenceSorterImpl<>(objectives, fnds);

    var mutation = new DefaultMutation();
    var crossover = new SinglePointFixedCrossover();
    var selection = new RankSelection<>(objectives, svd, preferenceSorter);

    ChromosomeGenerator<TestCase> chromosomeGenerator;
//    if (SeedOptions.any()) {
//      chromosomeGenerator = new SeededTestCaseGenerator(hir, mir, mutation, crossover);
//    } else {
//      chromosomeGenerator = new RandomTestCaseGenerator(hir, mir, mutation, crossover);
//    }
    chromosomeGenerator = new RandomTestCaseGenerator(hir, mir, mutation, crossover);

    var populationGenerator = new FixedSizePopulationGenerator<>(
        chromosomeGenerator, POPULATION_SIZE);

    var uncoveredObjectives = new UncoveredObjectives<>(objectives);
    var offspringGenerator = new OffspringGeneratorImpl(selection, uncoveredObjectives);

    var archive = new DefaultArchive<>(objectives);

    var timer = new Timer();
    timer.start();
    List<TestCase> initialPopulation = populationGenerator.get();
    if (SeedOptions.initialRandomPopulation()) {
      logger.info("-- Applying the InitialRandomPopulation seed strategy");
      initialPopulation = Collections.synchronizedList(new ArrayList<>());
      int randomGenerations = Math.max(GENERATIONS, 2);
      var randomTestCaseGenerator = new RandomTestCaseGenerator(hir, mir, mutation, crossover);
      var randomPopulation = IntStream.range(0, randomGenerations).parallel()
          .mapToObj(i -> randomTestCaseGenerator.get()).toList();
      initialPopulation.addAll(randomPopulation);
    }

    if (SeedOptions.useAllMethods()) {
      logger.info("-- Applying the AllMethods seed strategy");
      var methodsGenerator = new AllMethodTestCaseGenerator(hir, mir, mutation, crossover);
      var additionalPopulation = Stream.generate(methodsGenerator).parallel()
          .limit(hir.getCallables().size())
          .toList();
      initialPopulation.addAll(additionalPopulation);
    }

    logger.info("-- Initial population has been generated. Took {}s",
        TimeUnit.MILLISECONDS.toSeconds(timer.end()));

    String algorithm = SeedOptions.any() ? "seeded_dynamosa" : "dynamosa";
    List<Listener<TestCase>> listeners = List.of(
        //new Output<>(cli.getCrateName(), cli.getCrateRoot()),
        new DB(cli.getCrateName(), algorithm, cli.getRun())
    );

    var dynamosa = DynaMOSA.<TestCase>builder().maxGenerations(GENERATIONS)
        .populationSize(POPULATION_SIZE)
        .populationGenerator(populationGenerator)
        .offspringGenerator(offspringGenerator)
        .preferenceSorter(preferenceSorter)
        .archive(archive)
        .svd(svd)
        .container(crate)
        .mir(mir)
        .initialPopulation(initialPopulation)
        .listeners(listeners).build();

    var solutions = dynamosa.findSolution();
    crate.addAll(solutions);
  }

  public static void runRandomSearch(CLI cli) throws IOException, InterruptedException {
    var crate = Crate.load(cli);

    var hirLog = new File(cli.getHirPath());
    var hirJson = Files.readString(hirLog.toPath());
    var hir = new TyCtxt(JSONParser.parse(hirJson, cli.parseTraits()));
    var mir = new MirAnalysis<TestCase>(cli.getMirPath());

    var mutation = new DummyMutation<TestCase>();
    var crossover = new DummyCrossover<TestCase>();
    var chromosomeGenerator = new RandomSearchTestCaseGenerator(mir, hir, mutation, crossover);

    Set<MinimizingFitnessFunction<TestCase>> objectives = mir.targets();
    List<TestCase> initialPopulation = Stream.generate(chromosomeGenerator)
        .limit(Constants.POPULATION_SIZE).toList();
    if (SeedOptions.initialRandomPopulation()) {
      logger.info("-- Applying the InitialRandomPopulation seed strategy");
      initialPopulation = Collections.synchronizedList(new ArrayList<>());
      int randomGenerations = Math.max(GENERATIONS, 2);
      var randomTestCaseGenerator = new RandomTestCaseGenerator(hir, mir, mutation, crossover);
      var randomPopulation = IntStream.range(0, randomGenerations).parallel()
          .mapToObj(i -> randomTestCaseGenerator.get()).toList();
      initialPopulation.addAll(randomPopulation);
    }

    if (SeedOptions.useAllMethods()) {
      logger.info("-- Applying the AllMethods seed strategy");
      var methodsGenerator = new AllMethodTestCaseGenerator(hir, mir, mutation, crossover);
      var additionalPopulation = Stream.generate(methodsGenerator).parallel()
          .limit(hir.getCallables().size())
          .toList();
      initialPopulation.addAll(additionalPopulation);
    }

    String algorithm = SeedOptions.any() ? "seeded_random" : "random";
    List<Listener<TestCase>> listeners = List.of(
        new DB(cli.getCrateName(), algorithm, cli.getRun())
    );
    var archive = new DefaultArchive<>(objectives);
    var rs = RandomSearch.<TestCase>builder().samples(GENERATIONS * POPULATION_SIZE)
        .chromosomeGenerator(chromosomeGenerator)
        .archive(archive)
        .container(crate)
        .listeners(listeners)
        .maxGenerations(GENERATIONS)
        .initialPopulation(initialPopulation)
        .build();
    var solutions = rs.findSolution();
    crate.addAll(solutions);
  }
}
