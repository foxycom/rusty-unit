package de.unipassau.testify.allone;

import de.unipassau.testify.metaheuristics.operators.Crossover;
import de.unipassau.testify.util.Rnd;
import java.util.ArrayList;
import org.javatuples.Pair;

public class MaxOneCrossover implements Crossover<MaxOne> {

    @Override
    public Pair<MaxOne, MaxOne> apply(MaxOne parent1, MaxOne parent2) {
        var child1 = parent1.copy();
        var child2 = parent2.copy();

        var idx = Rnd.get().nextInt(child1.getBitVector().size() - 1) + 1;

        var b1 = child1.getBitVector();
        var b2 = child2.getBitVector();

        var b1New = new ArrayList<>(b1.subList(0, idx));
        b1New.addAll(b2.subList(idx, b2.size()));

        var b2New = new ArrayList<>(b2.subList(0, idx));
        b2New.addAll(b1.subList(idx, b1.size()));

        child1.setBitVector(b1New);
        child2.setBitVector(b2New);
        return Pair.with(child1, child2);
    }
}
