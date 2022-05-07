package de.unipassau.testify.algorithm.random;

import de.unipassau.testify.algorithm.Archive;
import de.unipassau.testify.metaheuristics.algorithm.GeneticAlgorithm;
import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.metaheuristics.chromosome.FixedSizePopulationGenerator;
import de.unipassau.testify.source.ChromosomeContainer;
import java.util.List;

public class RandomSearch<C extends AbstractTestCaseChromosome<C>> implements GeneticAlgorithm<C> {

    private final FixedSizePopulationGenerator<C> populationGenerator;
    private final Archive<C> archive;
    private final ChromosomeContainer<C> container;
    private final int maxGenerations;

    public RandomSearch(int maxGenerations, FixedSizePopulationGenerator<C> populationGenerator,
          Archive<C> archive, ChromosomeContainer<C> container) {
        this.maxGenerations = maxGenerations;
        this.populationGenerator = populationGenerator;
        this.container = container;
        this.archive = archive;
    }

    @Override
    public List<C> findSolution() {
        for (int gen = 0; gen < maxGenerations; gen++) {
            var population = populationGenerator.get();
            container.addAll(population);
            container.execute();
            archive.update(population);
        }

        return archive.get();
    }
}
