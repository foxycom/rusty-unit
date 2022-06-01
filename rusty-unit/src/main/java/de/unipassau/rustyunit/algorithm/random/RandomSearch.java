package de.unipassau.rustyunit.algorithm.random;

import de.unipassau.rustyunit.Constants;
import de.unipassau.rustyunit.Listener;
import de.unipassau.rustyunit.algorithm.Archive;
import de.unipassau.rustyunit.exec.ChromosomeExecutor.Status;
import de.unipassau.rustyunit.metaheuristics.algorithm.GeneticAlgorithm;
import de.unipassau.rustyunit.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.rustyunit.metaheuristics.chromosome.ChromosomeGenerator;
import de.unipassau.rustyunit.source.ChromosomeContainer;
import java.util.ArrayList;
import java.util.List;
import java.util.stream.IntStream;
import java.util.stream.Stream;
import me.tongfei.progressbar.ProgressBar;

public class RandomSearch<C extends AbstractTestCaseChromosome<C>> implements GeneticAlgorithm<C> {

  private final ChromosomeGenerator<C> chromosomeGenerator;
  private final Archive<C> archive;
  private final ChromosomeContainer<C> container;

  private final List<Listener<C>> listeners;

  private final int maxGenerations;

  private final int samples;

  public RandomSearch(int samples, ChromosomeGenerator<C> chromosomeGenerator,
      Archive<C> archive, ChromosomeContainer<C> container, List<Listener<C>> listeners,
      int maxGenerations) {
    this.chromosomeGenerator = chromosomeGenerator;
    this.container = container;
    this.archive = archive;
    this.samples = samples;
    this.maxGenerations = maxGenerations;
    this.listeners = listeners;
  }

  @Override
  public List<C> findSolution() {
    var status = Status.OK;
    try (ProgressBar pb = new ProgressBar("Random Search", maxGenerations)) {
      int gen = 0;
      while (gen < maxGenerations) {
        pb.setExtraMessage(barStatus("Population"));

        var population = Stream.generate(chromosomeGenerator).limit(Constants.POPULATION_SIZE).toList();

        pb.setExtraMessage(barStatus("cargo test"));
        container.addAll(population);
        status = container.execute();
        if (status == Status.COMPILATION_ERROR) {
          //throw new RuntimeException("Not implemented");
          System.out.println("Failed, restart...");
          pb.setExtraMessage(barStatus("Broken, recovering..."));
          continue;
        }

        archive.update(population);
        onExecuted(gen);

        if (archive.coverage().coverage() == 100) {
          pb.step();
          pb.maxHint(gen);
          // Early return if we reached 100% coverage
          break;
        }

        gen++;
        pb.step();
      }
      onPopulation(gen, archive.get());

    }
    return archive.get();
  }

  private void onExecuted(int gen) {
    var coverage = archive.coverage();
    var status = Listener.Status.builder()
        .coveredTargets(coverage.coveredTargets())
        .coverage(coverage.coverage())
        .avgLength(((double) archive.get().stream().map(C::size).reduce(Integer::sum).get()) / archive.size())
        .generation(gen)
        .tests(archive.size())
        .build();
    listeners.forEach(listener -> listener.onExecuted(status));
  }

  private void onPopulation(int gen, List<C> population) {
    listeners.forEach(listener -> listener.onPopulation(gen, population));
  }

  private String barStatus(String msg) {
    var coverage = archive.coverage();
    return String.format("Cov: (%.2f%% - %d/%d) | Tests: %d | %s", coverage.coverage(),
        coverage.coveredTargets(), archive.numberOfObjectives(), archive.size(), msg);
  }

}
