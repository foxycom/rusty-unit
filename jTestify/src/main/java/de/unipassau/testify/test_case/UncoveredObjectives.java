package de.unipassau.testify.test_case;

import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import java.util.ArrayList;
import java.util.List;

public class UncoveredObjectives<C extends AbstractTestCaseChromosome<C>> {

  private final List<MinimizingFitnessFunction<C>> objectives;
  private List<MinimizingFitnessFunction<C>> uncoveredObjectives = new ArrayList<>();

  public UncoveredObjectives(List<MinimizingFitnessFunction<C>> objectives) {
    this.objectives = objectives;
  }

  public void setCurrentPopulation(List<C> population) {
    uncoveredObjectives = uncoveredObjectives(population);
  }

  public List<MinimizingFitnessFunction<C>> getUncoveredObjectives() {
    return uncoveredObjectives;
  }

  private List<MinimizingFitnessFunction<C>> uncoveredObjectives(List<C> population) {
    List<MinimizingFitnessFunction<C>> uncoveredObjectives = new ArrayList<>();
    for (var objective : objectives) {
      boolean covered = false;
      for (var individual : population) {
        if (individual.getFitness(objective) == 0) {
          covered = true;
          break;
        }
      }

      if (!covered) {
        uncoveredObjectives.add(objective);
      }
    }

    return uncoveredObjectives;
  }
}
