package de.unipassau.testify.algorithm.dynamosa;

import de.unipassau.testify.algorithm.Archive;
import de.unipassau.testify.algorithm.PreferenceSorter;
import de.unipassau.testify.algorithm.SVD;
import de.unipassau.testify.exec.Output;
import de.unipassau.testify.generator.OffspringGenerator;
import de.unipassau.testify.metaheuristics.algorithm.GeneticAlgorithm;
import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.metaheuristics.chromosome.FixedSizePopulationGenerator;
import de.unipassau.testify.mir.MirAnalysis;
import de.unipassau.testify.source.ChromosomeContainer;
import java.util.ArrayList;
import java.util.List;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

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

  public DynaMOSA(int maxGenerations, int populationSize,
      FixedSizePopulationGenerator<C> populationGenerator,
      OffspringGenerator<C> offspringGenerator,
      PreferenceSorter<C> preferenceSorter,
      Archive<C> archive,
      SVD<C> svd,
      ChromosomeContainer<C> container,
      MirAnalysis<C> mir,
      Output<C> output
      ) {
    this.maxGenerations = maxGenerations;
    this.populationSize = populationSize;
    this.populationGenerator = populationGenerator;
    this.offspringGenerator = offspringGenerator;
    this.archive = archive;
    this.preferenceSorter = preferenceSorter;
    this.svd = svd;
    this.container = container;
    this.mir = mir;
    this.output = output;
  }


  @Override
  public List<C> findSolution() {
    var nOfTargets = mir.targets().size();

    var population = populationGenerator.get();

    output.addPopulation(0, population);

    var allTargets = mir.targets();
    var targets = mir.independentTargets();
    container.addAll(population);
    container.executeWithInstrumentation();
    archive.update(population);
    targets = mir.updateTargets(targets, population);

    output.addCoveredTargets(0, nOfTargets - targets.size(), nOfTargets);

    for (int gen = 1; gen < maxGenerations; gen++) {
      System.out.printf("-- Generation %d%n", gen);
      var offspring = offspringGenerator.get(population);

      output.addPopulation(gen, offspring);

      container.addAll(offspring);
      container.executeWithInstrumentation();
      archive.update(offspring);
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
