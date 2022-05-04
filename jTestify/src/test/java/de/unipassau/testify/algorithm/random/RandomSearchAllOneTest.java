package de.unipassau.testify.algorithm.random;

import de.unipassau.testify.algorithm.Archive;
import de.unipassau.testify.algorithm.DefaultArchive;
import de.unipassau.testify.allone.MaxOne;
import de.unipassau.testify.allone.MaxOneContainer;
import de.unipassau.testify.allone.MaxOneCrossover;
import de.unipassau.testify.allone.MaxOneFitness;
import de.unipassau.testify.allone.MaxOneGenerator;
import de.unipassau.testify.allone.MaxOneMutation;
import de.unipassau.testify.metaheuristics.chromosome.ChromosomeGenerator;
import de.unipassau.testify.metaheuristics.chromosome.FixedSizePopulationGenerator;
import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import de.unipassau.testify.metaheuristics.operators.Crossover;
import de.unipassau.testify.metaheuristics.operators.Mutation;
import de.unipassau.testify.source.ChromosomeContainer;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.stream.Collectors;
import java.util.stream.IntStream;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

class RandomSearchAllOneTest {

    private RandomSearch<MaxOne> randomSearch;
    private FixedSizePopulationGenerator<MaxOne> populationGenerator;
    private ChromosomeGenerator<MaxOne> chromosomeGenerator;
    private Mutation<MaxOne> mutation;
    private Crossover<MaxOne> crossover;
    private Archive<MaxOne> archive;
    private Set<MinimizingFitnessFunction<MaxOne>> objectives;
    private ChromosomeContainer<MaxOne> container;
    private int length = 10000;

    @BeforeEach
    void setUp() {
        objectives = IntStream.range(0, length).mapToObj(MaxOneFitness::new).collect(
              Collectors.toSet());
        mutation = new MaxOneMutation();
        crossover = new MaxOneCrossover();
        chromosomeGenerator = new MaxOneGenerator(length, mutation, crossover);
        populationGenerator = new FixedSizePopulationGenerator<>(chromosomeGenerator, 3);
        archive = new DefaultArchive<>(objectives);
        container = new MaxOneContainer();
        randomSearch = new RandomSearch<>(2, populationGenerator, archive, container);
    }

    @Test
    void testAllOne() {
        var solution = randomSearch.findSolution();

        System.out.printf("Avg fitness %d%n", avgFitness(solution));
    }

    private int avgFitness(List<MaxOne> solutions) {
        Map<MinimizingFitnessFunction<MaxOne>, Double> fitness = new HashMap<>();
        for (var objective : objectives) {
            fitness.put(objective, 1.0);
            for (var solution : solutions) {
                var value = objective.getFitness(solution);
                var bestFitness = fitness.get(objective);
                if (value < bestFitness) {
                    fitness.put(objective, value);
                }
            }
        }

        return fitness.values().stream().map(Double::intValue).reduce(Integer::sum).get();
    }
}