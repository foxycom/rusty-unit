package de.unipassau.testify.generator;

import static de.unipassau.testify.Constants.P_CROSSOVER;

import de.unipassau.testify.metaheuristics.operators.Selection;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.UncoveredObjectives;
import de.unipassau.testify.util.Rnd;
import java.util.ArrayList;
import java.util.List;

public class OffspringGeneratorImpl implements OffspringGenerator<TestCase> {
  private final Selection<TestCase> selection;
  private final UncoveredObjectives<TestCase> uncoveredObjectives;

  public OffspringGeneratorImpl(
      Selection<TestCase> selection, UncoveredObjectives<TestCase> uncoveredObjectives) {
    this.selection = selection;
    this.uncoveredObjectives = uncoveredObjectives;
  }

  @Override
  public List<TestCase> get(List<TestCase> population) {
    List<TestCase> offspringPopulation = new ArrayList<>();
    uncoveredObjectives.setCurrentPopulation(population);
    while (offspringPopulation.size() < population.size()) {
      final var parent1 = selection.apply(population);
      final var parent2 = selection.apply(population);

      TestCase offspring1;
      TestCase offspring2;

      if (Rnd.get().nextDouble() < P_CROSSOVER) {
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
