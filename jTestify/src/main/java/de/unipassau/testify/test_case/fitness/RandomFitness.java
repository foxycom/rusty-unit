package de.unipassau.testify.test_case.fitness;

import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.util.Rnd;

public class RandomFitness extends Fitness {

  public RandomFitness(int branchId) {
    super(branchId);
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
