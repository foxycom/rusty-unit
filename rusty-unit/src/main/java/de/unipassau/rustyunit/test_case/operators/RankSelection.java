package de.unipassau.rustyunit.test_case.operators;

import de.unipassau.rustyunit.Constants;
import de.unipassau.rustyunit.algorithm.PreferenceSorter;
import de.unipassau.rustyunit.algorithm.SVD;
import de.unipassau.rustyunit.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.rustyunit.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import de.unipassau.rustyunit.metaheuristics.operators.Selection;
import de.unipassau.rustyunit.util.Rnd;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.LinkedList;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.stream.IntStream;

public class RankSelection<C extends AbstractTestCaseChromosome<C>> implements Selection<C> {

  private final Set<MinimizingFitnessFunction<C>> objectives;
  private final SVD<C> svd;
  private final PreferenceSorter<C> preferenceSorter;

  private final Map<Integer, List<C>> cache;

  public RankSelection(
      Set<MinimizingFitnessFunction<C>> objectives, SVD<C> svd,
      PreferenceSorter<C> preferenceSorter) {
    this.objectives = objectives;
    this.svd = svd;
    this.preferenceSorter = preferenceSorter;
    this.cache = new HashMap<>();
  }

  private List<C> sort(final List<C> population) {
    var sortedPopulation = new LinkedList<C>();
    var fronts = preferenceSorter.sort(population);
    fronts.forEach((key, value) -> svd.compute(value));
    IntStream.range(0, fronts.size()).mapToObj(fronts::get).forEach(sortedPopulation::addAll);
    return sortedPopulation;
  }

  @Override
  public C apply(List<C> pPopulation) {
    if (pPopulation.isEmpty() || pPopulation.size() == 1) {
      throw new RuntimeException("Huh?");
    }

    List<C> population;
    if (cache.containsKey(pPopulation.hashCode())) {
      population = cache.get(pPopulation.hashCode());
    } else {
      population = new ArrayList<>(sort(pPopulation));
      cache.put(pPopulation.hashCode(), population);
    }
    final var N = population.size();

    final var probabilities = new ArrayList<Double>(N);
    final var bias = Constants.SELECTION_BIAS;
    IntStream.range(0, N).forEach(i -> {
      var f2 = bias - (2 * i * (bias - 1)) / (double) (N - 1);
      probabilities.add(f2);
    });

    final var fitnessSum = probabilities.stream().reduce(Double::sum).get();
    final var pick = Rnd.get().nextDouble() * fitnessSum;
    var current = 0.0;
    for (int i = 0; i < probabilities.size(); i++) {
      current += probabilities.get(i);
      if (current > pick) {
        return population.get(i);
      }
    }

    throw new RuntimeException("This should never happen");
  }
}
