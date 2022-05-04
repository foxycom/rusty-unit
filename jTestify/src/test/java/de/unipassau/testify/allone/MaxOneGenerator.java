package de.unipassau.testify.allone;

import de.unipassau.testify.metaheuristics.chromosome.ChromosomeGenerator;
import de.unipassau.testify.metaheuristics.operators.Crossover;
import de.unipassau.testify.metaheuristics.operators.Mutation;
import de.unipassau.testify.util.Rnd;
import java.util.ArrayList;
import java.util.List;

public class MaxOneGenerator implements ChromosomeGenerator<MaxOne> {
    private final int length;
    private final Mutation<MaxOne> mutation;
    private final Crossover<MaxOne> crossover;

    public MaxOneGenerator(int length,
          Mutation<MaxOne> mutation,
          Crossover<MaxOne> crossover) {
        this.length = length;
        this.mutation = mutation;
        this.crossover = crossover;
    }

    @Override
    public MaxOne get() {
        final List<Integer> bitVector = new ArrayList<>(length);
        for (int i = 0; i < length; i++) {
            bitVector.add(Rnd.get().nextInt(2));
        }

        return new MaxOne(bitVector, mutation, crossover);

    }
}
