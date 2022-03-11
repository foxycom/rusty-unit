package de.unipassau.testify.algorithm;

import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.LinkedHashSet;
import java.util.LinkedList;
import java.util.List;
import java.util.Map;
import java.util.Set;

public class PreferenceSorterImpl<C extends AbstractTestCaseChromosome<C>> implements PreferenceSorter<C> {

  private final Set<MinimizingFitnessFunction<C>> objectives;
  private final FNDS<C> fnds;

  public PreferenceSorterImpl(
      Set<MinimizingFitnessFunction<C>> objectives, FNDS<C> fnds) {
    this.objectives = objectives;
    this.fnds = fnds;
  }


  @Override
  public Map<Integer, List<C>> sort(List<C> pPopulation) {
    return sort(pPopulation, objectives);
  }

  @Override
  public Map<Integer, List<C>> sort(List<C> pPopulation, Set<MinimizingFitnessFunction<C>> targets) {
    var population = new LinkedList<>(pPopulation);
    var fronts = new HashMap<Integer, List<C>>();
    var f0 = new LinkedHashSet<C>();
    var uncoveredBranches = new ArrayList<MinimizingFitnessFunction<C>>();

    for (var objective : targets) {
      double minDist = Double.MAX_VALUE;
      C bestIndividual = null;
      for (var individual : population) {
        var dist = individual.getFitness(objective);
        if (dist < minDist) {
          bestIndividual = individual;
          minDist = dist;
        }
      }

      if (minDist > 0.0) {
        uncoveredBranches.add(objective);
        if (bestIndividual != null) {
          f0.add(bestIndividual);
        }
      }
    }

    fronts.put(0, new ArrayList<>(f0));
    population.removeIf(f0::contains);
    if (!population.isEmpty()) {
      var remainingFronts = fnds.sort(population, uncoveredBranches);
      for (int i = 0; i < remainingFronts.size(); i++) {
        fronts.put(i + 1, remainingFronts.get(i));
      }
    }
    return fronts;
  }
}
