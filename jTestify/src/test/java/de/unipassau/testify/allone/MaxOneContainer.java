package de.unipassau.testify.allone;

import de.unipassau.testify.exec.LLVMCoverage;
import de.unipassau.testify.source.ChromosomeContainer;
import java.io.IOException;
import java.util.List;

public class MaxOneContainer implements ChromosomeContainer<MaxOne> {

    @Override
    public void addAll(List<MaxOne> chromosomes) {

    }

    @Override
    public void refresh() {
    }

    @Override
    public List<MaxOne> chromosomes() {
        throw new RuntimeException("chromosomes is not implemented");
    }

    @Override
    public int executeWithInstrumentation() {
        return 0;
    }

    @Override
    public LLVMCoverage executeWithLlvmCoverage() throws IOException, InterruptedException {
        return null;
    }

    @Override
    public String getPath() {
        throw new RuntimeException("getPath is not implemented");
    }

    @Override
    public String getName() {
        throw new RuntimeException("getName is not implemented");
    }
}