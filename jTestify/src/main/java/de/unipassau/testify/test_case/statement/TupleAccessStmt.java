package de.unipassau.testify.test_case.statement;

import com.google.common.base.Preconditions;
import de.unipassau.testify.test_case.var.Index;
import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.var.VarReference;
import de.unipassau.testify.test_case.callable.Callable;
import de.unipassau.testify.test_case.callable.TupleAccess;
import de.unipassau.testify.test_case.type.Type;
import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.UUID;

public class TupleAccessStmt implements Statement {

    private final TupleAccess tupleAccess;
    private final TestCase testCase;
    private Index index;

    private VarReference owner;

    private VarReference returnValue;

    @Override
    public TupleAccessStmt asTupleAccessStmt() {
        return this;
    }

    @Override
    public boolean isTupleAccessStmt() {
        return true;
    }

    private UUID id;

    public TupleAccessStmt(TestCase testCase, VarReference owner, VarReference index,
          VarReference returnValue, TupleAccess tupleAccess) {
        this.id = UUID.randomUUID();
        this.testCase = testCase;
        this.index = (Index) index;
        this.owner = owner;
        this.returnValue = returnValue;
        this.tupleAccess = tupleAccess;
    }

    @Override
    public UUID id() {
        return id;
    }

    @Override
    public Optional<Type> returnType() {
        return Optional.of(returnValue.type());
    }

    @Override
    public Optional<VarReference> returnValue() {
        return Optional.of(returnValue);
    }

    @Override
    public boolean returnsValue() {
        return true;
    }

    @Override
    public List<VarReference> args() {
        return List.of(owner);
    }

    @Override
    public void setArgs(List<VarReference> args) {
        Preconditions.checkArgument(args.size() == 1);
        owner = args.get(0);
    }

    @Override
    public void setArg(int pos, VarReference var) {
        Preconditions.checkArgument(pos == 0);
        owner = var;
    }

    @Override
    public List<Param> params() {
        return tupleAccess.getParams();
    }

    @Override
    public List<Type> actualParamTypes() {
        return Collections.emptyList();
    }

    @Override
    public TestCase testCase() {
        return testCase;
    }

    @Override
    public String getSrcFilePath() {
        return null;
    }

    @Override
    public boolean isPublic() {
        return tupleAccess.isPublic();
    }

    @Override
    public Callable getCallable() {
        return tupleAccess;
    }

    @Override
    public boolean consumes(VarReference var) {
        // Index is always usize or something which is copyable, so
        // we say it is not consumed from the test perspective
        return false;
    }

    @Override
    public boolean borrows(VarReference var) {
        return false;
    }

    @Override
    public boolean mutates(VarReference var) {
        return false;
    }

    @Override
    public void replace(VarReference oldVar, VarReference newVar) {
        throw new RuntimeException("Not implemented");
    }

    @Override
    public Statement copy(TestCase testCase) {
        return new TupleAccessStmt(testCase, owner.copy(testCase), index,
              returnValue.copy(testCase), tupleAccess);
    }

    public Index index() {
        return index;
    }

    public VarReference owner() {
        return owner;
    }

    @Override
    public int position() {
        return testCase.stmtPosition(this).orElseThrow();
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) {
            return true;
        }
        if (o == null || getClass() != o.getClass()) {
            return false;
        }
        TupleAccessStmt that = (TupleAccessStmt) o;
        return tupleAccess.equals(that.tupleAccess) && index == that.index && returnValue.equals(
              that.returnValue) && id.equals(that.id);
    }

    @Override
    public int hashCode() {
        return Objects.hash(tupleAccess, index, returnValue, id);
    }
}
