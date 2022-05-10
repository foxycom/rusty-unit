package de.unipassau.testify.algorithm.random;

import de.unipassau.testify.algorithm.Archive;
import de.unipassau.testify.metaheuristics.algorithm.GeneticAlgorithm;
import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.metaheuristics.chromosome.ChromosomeGenerator;
import de.unipassau.testify.source.ChromosomeContainer;
import java.util.ArrayList;
import java.util.List;
import java.util.stream.Collectors;
import java.util.stream.IntStream;

public class RandomSearch<C extends AbstractTestCaseChromosome<C>> implements GeneticAlgorithm<C> {

    private final ChromosomeGenerator<C> chromosomeGenerator;
    private final Archive<C> archive;
    private final ChromosomeContainer<C> container;

    private final int samples;

    public RandomSearch(int samples, ChromosomeGenerator<C> chromosomeGenerator,
          Archive<C> archive, ChromosomeContainer<C> container) {
        this.chromosomeGenerator = chromosomeGenerator;
        this.container = container;
        this.archive = archive;
        this.samples = samples;
    }

    @Override
    public List<C> findSolution() {
        var population = IntStream.of(samples).mapToObj(i -> chromosomeGenerator.get()).toList();
        container.addAll(population);
        container.execute();
        archive.update(population);
        return archive.get();
    }
}
