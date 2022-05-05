package de.unipassau.testify.allone;

import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.var.VarReference;
import de.unipassau.testify.test_case.statement.Statement;
import de.unipassau.testify.test_case.type.Type;
import java.util.List;
import java.util.Optional;
import java.util.UUID;

public class One implements Statement {

    private int bit;

    public One(int bit) {
        this.bit = bit;
    }

    public int bit() {
        return bit;
    }

    public void setBit(int bit) {
        this.bit = bit;
    }

    @Override
    public UUID id() {
        throw new RuntimeException("id is not implemented");
    }

    @Override
    public Optional<Type> returnType() {
        throw new RuntimeException("returnType is not implemented");
    }

    @Override
    public Optional<VarReference> returnValue() {
        throw new RuntimeException("returnValue is not implemented");
    }

    @Override
    public boolean returnsValue() {
        throw new RuntimeException("returnsValue is not implemented");
    }

    @Override
    public List<VarReference> args() {
        throw new RuntimeException("args is not implemented");
    }

    @Override
    public void setArgs(List<VarReference> args) {
        throw new RuntimeException("setArgs is not implemented");
    }

    @Override
    public void setArg(int pos, VarReference var) {
        throw new RuntimeException("setArg is not implemented");
    }

    @Override
    public List<Param> params() {
        throw new RuntimeException("params is not implemented");
    }

    @Override
    public List<Type> actualParamTypes() {
        throw new RuntimeException("actualParamTypes is not implemented");
    }

    @Override
    public TestCase testCase() {
        throw new RuntimeException("testCase is not implemented");
    }

    @Override
    public String getSrcFilePath() {
        throw new RuntimeException("getSrcFilePath is not implemented");
    }

    @Override
    public boolean isPublic() {
        throw new RuntimeException("isPublic is not implemented");
    }

    @Override
    public boolean consumes(VarReference var) {
        throw new RuntimeException("consumes is not implemented");
    }

    @Override
    public boolean borrows(VarReference var) {
        throw new RuntimeException("borrows is not implemented");
    }

    @Override
    public boolean mutates(VarReference var) {
        throw new RuntimeException("mutates is not implemented");
    }

    @Override
    public void replace(VarReference oldVar, VarReference newVar) {
        throw new RuntimeException("replace is not implemented");
    }

    @Override
    public Statement copy(TestCase testCase) {
        throw new RuntimeException("copy is not implemented");
    }

    @Override
    public int position() {
        throw new RuntimeException("position is not implemented");
    }

    @Override
    public String toString() {
        return String.format("%d", bit);
    }
}
