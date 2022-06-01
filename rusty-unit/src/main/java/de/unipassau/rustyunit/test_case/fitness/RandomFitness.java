package de.unipassau.rustyunit.test_case.fitness;

import de.unipassau.rustyunit.test_case.TestCase;
import de.unipassau.rustyunit.util.Rnd;

public class RandomFitness extends Fitness {

  public RandomFitness() {
    super(null);
  }

  @Override
  public double getFitness(TestCase testCase) throws NullPointerException {
    if (Rnd.get().nextDouble() < 0.5) {
      return 0d;
    } else {
      return Rnd.get().nextDouble() * 1000d;
    }
  }
}
