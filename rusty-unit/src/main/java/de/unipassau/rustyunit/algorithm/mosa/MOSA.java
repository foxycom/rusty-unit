package de.unipassau.rustyunit.algorithm.mosa;

import de.unipassau.rustyunit.algorithm.Archive;
import de.unipassau.rustyunit.algorithm.PreferenceSorter;
import de.unipassau.rustyunit.algorithm.SVD;
import de.unipassau.rustyunit.generator.OffspringGenerator;
import de.unipassau.rustyunit.metaheuristics.algorithm.GeneticAlgorithm;
import de.unipassau.rustyunit.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.rustyunit.metaheuristics.chromosome.FixedSizePopulationGenerator;
import de.unipassau.rustyunit.source.ChromosomeContainer;
import java.util.ArrayList;
import java.util.List;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class MOSA<C extends AbstractTestCaseChromosome<C>> implements GeneticAlgorithm<C> {
  private static final Logger logger = LoggerFactory.getLogger(MOSA.class);

  private final int maxGenerations;
  private final int populationSize;
  private final FixedSizePopulationGenerator<C> populationGenerator;
  private final OffspringGenerator<C> offspringGenerator;
  private final Archive<C> archive;
  private final PreferenceSorter<C> preferenceSorter;
  private final SVD<C> svd;
  private final ChromosomeContainer<C> container;

  public MOSA(int maxGenerations, int populationSize,
      FixedSizePopulationGenerator<C> populationGenerator,
      OffspringGenerator<C> offspringGenerator,
      PreferenceSorter<C> preferenceSorter,
      Archive<C> archive,
      SVD<C> svd,
      ChromosomeContainer<C> container) {
    this.maxGenerations = maxGenerations;
    this.populationSize = populationSize;
    this.populationGenerator = populationGenerator;
    this.offspringGenerator = offspringGenerator;
    this.archive = archive;
    this.preferenceSorter = preferenceSorter;
    this.svd = svd;
    this.container = container;
  }


  @Override
  public List<C> findSolution() {
    var population = populationGenerator.get();

    // TODO: 10.02.22 run tests
    container.addAll(population);
    container.execute();

    archive.update(population);

    for (int gen = 0; gen < maxGenerations; gen++) {
      System.out.printf("Generation %d started%n", gen);
      var offspring = offspringGenerator.get(population);

      container.addAll(offspring);
      container.execute();
      // TODO: 10.02.22 run tests

      archive.update(offspring);
      var combined = new ArrayList<C>(population.size() + offspring.size());
      combined.addAll(population);
      combined.addAll(offspring);

      var fronts = preferenceSorter.sort(combined);
      population.clear();

      for (int i = 0; i < fronts.size(); i++) {
        var front = fronts.get(i);
        svd.compute(front);
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