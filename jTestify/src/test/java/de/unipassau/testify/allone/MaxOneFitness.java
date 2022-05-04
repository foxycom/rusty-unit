package de.unipassau.testify.allone;

import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import java.util.Objects;

public class MaxOneFitness implements MinimizingFitnessFunction<MaxOne> {

    private final int index;

    public MaxOneFitness(int index) {
        this.index = index;
    }

    @Override
    public double getFitness(MaxOne individual) throws NullPointerException {
        if (individual.getBitVector().get(index) == 0) {
            return 1;
        } else {
            return 0;
        }
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) {
            return true;
        }
        if (!(o instanceof MaxOneFitness)) {
            return false;
        }
        MaxOneFitness that = (MaxOneFitness) o;
        return index == that.index;
    }

    @Override
    public int hashCode() {
        return Objects.hash(index);
    }
}
