package de.unipassau.rustyunit.algorithm.dynamosa;

import de.unipassau.rustyunit.Listener;
import de.unipassau.rustyunit.algorithm.Archive;
import de.unipassau.rustyunit.algorithm.PreferenceSorter;
import de.unipassau.rustyunit.algorithm.SVD;
import de.unipassau.rustyunit.exec.ChromosomeExecutor.Status;
import de.unipassau.rustyunit.exec.Output;
import de.unipassau.rustyunit.generator.OffspringGenerator;
import de.unipassau.rustyunit.metaheuristics.algorithm.GeneticAlgorithm;
import de.unipassau.rustyunit.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.rustyunit.metaheuristics.chromosome.FixedSizePopulationGenerator;
import de.unipassau.rustyunit.mir.MirAnalysis;
import de.unipassau.rustyunit.source.ChromosomeContainer;
import de.unipassau.rustyunit.test_case.CallableSelector;
import java.util.ArrayList;
import java.util.List;
import lombok.Builder;
import me.tongfei.progressbar.ProgressBar;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Builder
public class DynaMOSA<C extends AbstractTestCaseChromosome<C>> implements GeneticAlgorithm<C> {

  private static final Logger logger = LoggerFactory.getLogger(DynaMOSA.class);

  private final int maxGenerations;
  private final int populationSize;
  private final FixedSizePopulationGenerator<C> populationGenerator;
  private final OffspringGenerator<C> offspringGenerator;
  private final Archive<C> archive;
  private final PreferenceSorter<C> preferenceSorter;
  private final SVD<C> svd;
  private final ChromosomeContainer<C> container;
  private final MirAnalysis<C> mir;
  private final List<C> initialPopulation;

  private List<Listener<C>> listeners;

  @Override
  public List<C> findSolution() {
    var targets = mir.independentTargets();
    logger.info(">> Independent targets: {}", targets.size());

    List<C> population = initialPopulation;
    try (ProgressBar pb = new ProgressBar("DynaMOSA", maxGenerations)) {
      Status status;
      pb.setExtraMessage(barStatus("cargo test"));
      do {
        container.addAll(population);
        status = container.execute();
        switch (status) {
          case COMPILATION_ERROR -> throw new RuntimeException("Not implemented");
          default -> {
          }
        }
      } while (status != Status.OK);

      pb.setExtraMessage(barStatus("Archive"));

      archive.update(population);
      onExecuted(0);
      CallableSelector.setCurrentPopulation(archive.get());
      targets = mir.updateTargets(targets, population);
      pb.step();

      int gen = 1;
      while (gen < maxGenerations) {
        pb.setExtraMessage(barStatus("Offspring"));
        var offspring = offspringGenerator.get(population);

        container.addAll(offspring);
        onPopulation(gen, offspring);

        pb.setExtraMessage(barStatus("cargo test"));
        status = container.execute();
        if (status == Status.COMPILATION_ERROR) {
          System.out.println("Broken tests...");
          pb.setExtraMessage(barStatus("Broken, recovering..."));
          continue;
        }

        pb.setExtraMessage(barStatus("Archive"));
        archive.update(offspring);
        onExecuted(gen);
        if (archive.coverage().coverage() == 100) {
          pb.step();
          pb.maxHint(gen);
          // Early return if we reached 100% coverage
          break;
        }

        CallableSelector.setCurrentPopulation(archive.get());

        var combined = new ArrayList<C>(population.size() + offspring.size());
        combined.addAll(population);
        combined.addAll(offspring);
        targets = mir.updateTargets(targets, combined);

        var fronts = preferenceSorter.sort(combined, targets);
        population.clear();

        for (int i = 0; i < fronts.size(); i++) {
          var front = fronts.get(i);
          svd.compute(front, targets);
          for (var t : front) {
            population.add(t);
            if (population.size() == populationSize) {
              break;
            }
          }

          if (population.size() == populationSize) {
            break;
          }
        }
        gen++;
        pb.step();
      }
      onPopulation(gen, archive.get());
    }

    var averageLength =
        ((double) archive.get().stream().map(AbstractTestCaseChromosome::size).reduce(Integer::sum)
            .get()) / archive.size();

    System.out.printf("Number of tests: %d, average length: %.2f%n", archive.size(), averageLength);

    return archive.get();
  }

  private void onExecuted(int gen) {
    var coverage = archive.coverage();
    var status = Listener.Status.builder()
        .coveredTargets(coverage.coveredTargets())
        .coverage(coverage.coverage())
        .avgLength(((double) archive.get().stream().map(C::size).reduce(Integer::sum).get())
            / archive.size())
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
    return String.format("Cov: (%.2f%% - %d/%d) | Tests: %d | %s",
        coverage.coverage(),
        coverage.coveredTargets(), archive.numberOfObjectives(), archive.size(),
        msg);
  }

}
