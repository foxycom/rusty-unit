package de.unipassau.rustyunit.test_case;

import de.unipassau.rustyunit.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.rustyunit.test_case.callable.Callable;
import de.unipassau.rustyunit.util.Rnd;
import java.util.ArrayList;
import java.util.Collection;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class CallableSelector {
    private static Map<Callable, Integer> usage = new HashMap<>();

    /**
     * Selects a callable with probabilities based on their usage
     * @return
     */
    public static Callable select(Collection<Callable> callables) {
        if (usage.isEmpty()) {
            // Init phase
            return Rnd.choice(callables);
        }

        List<Callable> sortedCallables = new ArrayList<>(callables);
        sortedCallables.sort((o1, o2) -> {
            var f1 = usage.getOrDefault(o1, 0);
            var f2 = usage.getOrDefault(o2, 0);
            return Integer.compare(f1, f2);
        });

        var p = 1.0 / sortedCallables.size();
        for (var callable : sortedCallables) {
            if (Rnd.get().nextDouble() < p) {
                return callable;
            }
        }

        return sortedCallables.get(0);
    }

    public static <C extends AbstractTestCaseChromosome<C>> void setCurrentPopulation(List<C> population) {
        usage.clear();
        for (var chromosome : population) {
            for (var stmt : chromosome.getStatements()) {
                if (stmt.isRefStmt() || stmt.isPrimitiveStmt()) {
                    continue;
                }

                var callable = stmt.getCallable();
                if (!usage.containsKey(callable)) {
                    usage.put(callable, 1);
                } else {
                    usage.compute(callable, (key, value) -> value + 1);
                }
            }
        }
    }
}
