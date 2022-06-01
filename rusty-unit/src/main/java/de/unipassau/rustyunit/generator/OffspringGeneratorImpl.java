package de.unipassau.rustyunit.generator;

import static de.unipassau.rustyunit.Constants.P_CROSSOVER;

import de.unipassau.rustyunit.Constants;
import de.unipassau.rustyunit.exec.TestCaseRunner;
import de.unipassau.rustyunit.exec.Timer;
import de.unipassau.rustyunit.metaheuristics.operators.Selection;
import de.unipassau.rustyunit.test_case.TestCase;
import de.unipassau.rustyunit.test_case.UncoveredObjectives;
import de.unipassau.rustyunit.util.Rnd;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.TimeUnit;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class OffspringGeneratorImpl implements OffspringGenerator<TestCase> {
  private static final Logger logger = LoggerFactory.getLogger(OffspringGeneratorImpl.class);

  private final Selection<TestCase> selection;
  private final UncoveredObjectives<TestCase> uncoveredObjectives;

  public OffspringGeneratorImpl(
      Selection<TestCase> selection, UncoveredObjectives<TestCase> uncoveredObjectives) {
    this.selection = selection;
    this.uncoveredObjectives = uncoveredObjectives;
  }

  @Override
  public List<TestCase> get(List<TestCase> population) {
    var timer = new Timer();
    timer.start();

    logger.info("\t>> Generating offspring");
    List<TestCase> offspringPopulation = new ArrayList<>(Constants.POPULATION_SIZE);
    uncoveredObjectives.setCurrentPopulation(population);
    while (offspringPopulation.size() < Constants.POPULATION_SIZE) {
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

      offspring1.cleanup();
      offspring2.cleanup();

      if (population.size() - offspringPopulation.size() >= 2) {
        offspringPopulation.add(offspring1);
        offspringPopulation.add(offspring2);
      } else {
        offspringPopulation.add(List.of(offspring1, offspring2).get(Rnd.get().nextInt(2)));
      }
    }

    var elapsedTime = timer.end();
    logger.info("\t>> Finished. Took {}s", TimeUnit.MILLISECONDS.toSeconds(elapsedTime));

    return offspringPopulation;
  }
}
