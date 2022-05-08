package de.unipassau.testify.algorithm;

import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.LinkedList;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import org.javatuples.Pair;

public class FNDSImpl<C extends AbstractTestCaseChromosome<C>> implements FNDS<C> {

  private final DominationStrategy<C> domination;

  public FNDSImpl(DominationStrategy<C> domination) {
    Objects.requireNonNull(domination);
    this.domination = domination;
  }

  @Override
  public Map<Integer, List<C>> sort(List<C> population,
      Set<MinimizingFitnessFunction<C>> objectives) {
    final Map<Integer, List<C>> front = new HashMap<>();
    final Map<C, List<C>> S = new HashMap<>();
    final Map<C, Integer> n = new HashMap<>();

    for (var p : population) {
      S.put(p, new LinkedList<>());
      n.put(p, 0);

      for (var q : population) {
        if (p.getId() == q.getId()) {
          continue;
        }
        if (domination.dominates(p, q, objectives)) {
          S.get(p).add(q);
        } else if (domination.dominates(q, p, objectives)) {
          n.compute(p, (key, counter) -> counter + 1);
        }
      }

      if (n.get(p) == 0) {
        front.putIfAbsent(0, new ArrayList<>());
        front.get(0).add(p);
      }
    }

    int i = 0;
    while (!front.get(i).isEmpty()) {
      List<C> Q = new ArrayList<>();
      for (var p : front.get(i)) {
        for (var q : S.get(p)) {
          n.compute(q, (key, counter) -> counter - 1);
          if (n.get(q) == 0) {
            Q.add(q);
          }
        }
      }

      i++;
      front.put(i, Q);
    }

    // The last entry seems to be always empty
    front.remove(front.size() - 1);
    return front;
  }
}
