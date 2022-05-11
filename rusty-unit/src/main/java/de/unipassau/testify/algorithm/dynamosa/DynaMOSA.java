package de.unipassau.testify.algorithm.dynamosa;

import de.unipassau.testify.Main.CLI;
import de.unipassau.testify.algorithm.Archive;
import de.unipassau.testify.algorithm.PreferenceSorter;
import de.unipassau.testify.algorithm.SVD;
import de.unipassau.testify.exec.ChromosomeExecutor.Status;
import de.unipassau.testify.exec.Output;
import de.unipassau.testify.generator.OffspringGenerator;
import de.unipassau.testify.metaheuristics.algorithm.GeneticAlgorithm;
import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.metaheuristics.chromosome.FixedSizePopulationGenerator;
import de.unipassau.testify.mir.MirAnalysis;
import de.unipassau.testify.source.ChromosomeContainer;
import de.unipassau.testify.test_case.CallableSelector;
import java.util.ArrayList;
import java.util.List;
import lombok.Builder;
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
  private final Output<C> output;

  private final List<C> initialPopulation;

  @Override
  public List<C> findSolution() {
    var nOfTargets = mir.targets().size();
    var targets = mir.independentTargets();
    System.out.printf("\t>> Independent targets: %d%n", targets.size());

    List<C> population = initialPopulation;

    Status status;
    do {
      container.addAll(population);
      status = container.execute();
      switch (status) {
        case COMPILATION_ERROR -> System.out.println("\t>> Broken tests found, regenerating...");
        default -> {}
      }
    } while (status != Status.OK);

    output.addPopulation(0, population);
    archive.update(population);
    CallableSelector.setCurrentPopulation(archive.get());
    targets = mir.updateTargets(targets, population);

    output.addCoveredTargets(0, nOfTargets - targets.size(), nOfTargets);

    for (int gen = 1; gen < maxGenerations; gen++) {
      System.out.printf("-- Generation %d%n", gen);
      var offspring = offspringGenerator.get(population);

      container.addAll(offspring);
      output.addPopulation(gen, offspring);
      status = container.execute();
      if (status == Status.COMPILATION_ERROR) {
        throw new RuntimeException("Non-compilable tests");
      }

      archive.update(offspring);
      CallableSelector.setCurrentPopulation(archive.get());
      targets = mir.updateTargets(targets, population);

      output.addCoveredTargets(gen, nOfTargets - targets.size(), nOfTargets);

      var combined = new ArrayList<C>(population.size() + offspring.size());
      combined.addAll(population);
      combined.addAll(offspring);

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
    }

    return archive.get();
  }

}
