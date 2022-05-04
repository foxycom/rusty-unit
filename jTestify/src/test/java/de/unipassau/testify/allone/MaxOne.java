package de.unipassau.testify.allone;

import de.unipassau.testify.generators.TestIdGenerator;
import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.metaheuristics.operators.Crossover;
import de.unipassau.testify.metaheuristics.operators.Mutation;
import de.unipassau.testify.test_case.TestCaseMetadata;
import de.unipassau.testify.test_case.statement.Statement;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

public class MaxOne extends AbstractTestCaseChromosome<MaxOne> {

    private List<One> bitVector;
    private int id;

    public MaxOne(
          List<Integer> bitVector,
          Mutation<MaxOne> mutation,
          Crossover<MaxOne> crossover) {
        super(mutation, crossover);
        this.id = TestIdGenerator.get();
        this.bitVector = bitVector.stream().map(One::new).collect(Collectors.toList());
    }

    public MaxOne(MaxOne other) {
        super(other.getMutation(), other.getCrossover());
        this.bitVector = new ArrayList<>(other.bitVector);
    }

    public List<Integer> getBitVector() {
        return bitVector.stream().map(One::bit).collect(Collectors.toList());
    }

    public void setBitVector(List<Integer> bitVector) {
        this.bitVector = bitVector.stream().map(One::new).collect(Collectors.toList());
    }

    @Override
    public int size() {
        return bitVector.size();
    }

    @Override
    public int getId() {
        return id;
    }

    @Override
    public List<Statement> getStatements() {
        return bitVector.stream().filter(o -> o.bit() == 0).collect(Collectors.toList());
    }

    @Override
    public TestCaseMetadata metadata() {
        return new TestCaseMetadata(id);
    }

    @Override
    public MaxOne copy() {
        return new MaxOne(this);
    }

    @Override
    public MaxOne self() {
        throw new RuntimeException("self is not implemented");
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) {
            return true;
        }
        if (o == null || getClass() != o.getClass()) {
            return false;
        }
        MaxOne that = (MaxOne) o;
        return bitVector.equals(that.bitVector);
    }

    @Override
    public int hashCode() {
        return Objects.hash(bitVector);
    }

    @Override
    public String toString() {
        return bitVector.toString();
    }
}
