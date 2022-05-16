package de.unipassau.rustyunit.algorithm.random;

import de.unipassau.rustyunit.algorithm.Archive;
import de.unipassau.rustyunit.metaheuristics.algorithm.GeneticAlgorithm;
import de.unipassau.rustyunit.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.rustyunit.metaheuristics.chromosome.ChromosomeGenerator;
import de.unipassau.rustyunit.source.ChromosomeContainer;
import java.util.List;
import java.util.stream.IntStream;
import java.util.stream.Stream;

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
        var population = Stream.generate(chromosomeGenerator).limit(samples).toList();
        container.addAll(population);
        container.execute();
        archive.update(population);
        return archive.get();
    }
}
