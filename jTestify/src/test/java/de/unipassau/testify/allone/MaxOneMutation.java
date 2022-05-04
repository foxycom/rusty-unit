package de.unipassau.testify.allone;

import de.unipassau.testify.metaheuristics.operators.Mutation;
import de.unipassau.testify.util.Rnd;

public class MaxOneMutation implements Mutation<MaxOne> {

    @Override
    public MaxOne apply(MaxOne individual) {
        var copy = individual.copy();
        final var p = 1.0 / copy.getBitVector().size();
        for (int i = 0; i < copy.getBitVector().size(); i++) {
            if (Rnd.get().nextDouble() <= p) {
                var val = copy.getBitVector().get(i);
                copy.getBitVector().set(i, 1 - val);
            }
        }
        return copy;
    }
}
