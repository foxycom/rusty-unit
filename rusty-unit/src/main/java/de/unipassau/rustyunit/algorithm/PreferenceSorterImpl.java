package de.unipassau.rustyunit.algorithm;

import de.unipassau.rustyunit.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.rustyunit.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import java.io.BufferedWriter;
import java.io.FileWriter;
import java.io.IOException;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashSet;
import java.util.LinkedList;
import java.util.List;
import java.util.Map;
import java.util.Map.Entry;
import java.util.Set;
import java.util.TreeMap;
import java.util.TreeMap;

public class PreferenceSorterImpl<C extends AbstractTestCaseChromosome<C>> implements PreferenceSorter<C> {

  private final Set<MinimizingFitnessFunction<C>> objectives;
  private final FNDS<C> fnds;

  private Map<String, Double> table;

  public PreferenceSorterImpl(
      Set<MinimizingFitnessFunction<C>> objectives, FNDS<C> fnds) {
    this.objectives = objectives;
    this.fnds = fnds;
    table = new TreeMap<>();
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
    var uncoveredBranches = new HashSet<MinimizingFitnessFunction<C>>();
    Map<String, Double> changed = new TreeMap<>();

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

      var objectiveName = objective.toString();
      if (table.containsKey(objectiveName)) {
        if (table.get(objectiveName) > minDist) {
          changed.put(objectiveName, minDist);
          table.put(objectiveName, minDist);
        }
      } else {
        table.put(objectiveName, minDist);
      }
      if (minDist > 0.0) {
        uncoveredBranches.add(objective);
        if (bestIndividual != null) {
          f0.add(bestIndividual);
        }
      }
    }

    log(changed);
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

  private void log(Map<String, Double> table) {
    var name = String.format("/Users/tim/master-thesis/test-tmp/%d.txt", System.currentTimeMillis());
    try (var out = new BufferedWriter(new FileWriter(name))) {
      for (Entry<String, Double> entry : table.entrySet()) {
        out.write(String.format("%s -> %f\n", entry.getKey(), entry.getValue()));
      }
    } catch (IOException e) {
      throw new RuntimeException(e);
    }
  }
}
