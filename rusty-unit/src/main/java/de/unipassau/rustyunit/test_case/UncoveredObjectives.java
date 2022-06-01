package de.unipassau.rustyunit.test_case;

import de.unipassau.rustyunit.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.rustyunit.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

public class UncoveredObjectives<C extends AbstractTestCaseChromosome<C>> {

  private final Set<MinimizingFitnessFunction<C>> objectives;
  private Set<MinimizingFitnessFunction<C>> uncoveredObjectives = new HashSet<>();

  public UncoveredObjectives(Set<MinimizingFitnessFunction<C>> objectives) {
    this.objectives = objectives;
  }

  public void setCurrentPopulation(List<C> population) {
    uncoveredObjectives = uncoveredObjectives(population);
  }

  public Set<MinimizingFitnessFunction<C>> getUncoveredObjectives() {
    return uncoveredObjectives;
  }

  private Set<MinimizingFitnessFunction<C>> uncoveredObjectives(List<C> population) {
    Set<MinimizingFitnessFunction<C>> uncoveredObjectives = new HashSet<>();
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
