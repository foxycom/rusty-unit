package de.unipassau.rustyunit.algorithm.dynamosa;

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
