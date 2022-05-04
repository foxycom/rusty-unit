package de.unipassau.testify.allone;

import de.unipassau.testify.generator.OffspringGenerator;
import de.unipassau.testify.metaheuristics.operators.Selection;
import de.unipassau.testify.test_case.UncoveredObjectives;
import de.unipassau.testify.util.Rnd;
import java.util.ArrayList;
import java.util.List;

public class MaxOneOffspringGenerator implements OffspringGenerator<MaxOne> {
    private final double P_xover = 0.7;
    private final Selection<MaxOne> selection;
    private final UncoveredObjectives<MaxOne> uncoveredObjectives;

    public MaxOneOffspringGenerator(
          Selection<MaxOne> selection, UncoveredObjectives<MaxOne> uncoveredObjectives) {
        this.selection = selection;
        this.uncoveredObjectives = uncoveredObjectives;
    }

    @Override
    public List<MaxOne> get(List<MaxOne> population) {
        List<MaxOne> offspringPopulation = new ArrayList<>();
        uncoveredObjectives.setCurrentPopulation(population);
        while (offspringPopulation.size() < population.size()) {
            final var parent1 = selection.apply(population);
            final var parent2 = selection.apply(population);

            MaxOne offspring1;
            MaxOne offspring2;

            if (Rnd.get().nextDouble() < P_xover) {
                var offspring = parent1.crossover(parent2);
                offspring1 = offspring.getValue0();
                offspring2 = offspring.getValue1();
            } else {
                offspring1 = parent1;
                offspring2 = parent2;
            }

            offspring1 = offspring1.mutate();
            offspring2 = offspring2.mutate();

            if (population.size() - offspringPopulation.size() >= 2) {
                offspringPopulation.add(offspring1);
                offspringPopulation.add(offspring2);
            } else {
                offspringPopulation.add(List.of(offspring1, offspring2).get(Rnd.get().nextInt(2)));
            }
        }

        return offspringPopulation;
    }
}