package de.unipassau.testify.test_case.callable;

import com.google.common.base.Preconditions;
import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.var.VarReference;
import de.unipassau.testify.test_case.statement.Statement;
import de.unipassau.testify.test_case.statement.TupleAccessStmt;
import de.unipassau.testify.test_case.type.Type;
import java.util.List;
import java.util.Objects;

public class TupleAccess implements Callable {

    private final Type returnType;

    private final Type parent;

    public TupleAccess(Type parent, Type returnType) {
        this.returnType = Objects.requireNonNull(returnType);
        this.parent = Objects.requireNonNull(parent);
    }

    @Override
    public TupleAccess asTupleAccess() {
        return this;
    }

    @Override
    public boolean isTupleAccess() {
        return true;
    }

    @Override
    public String getName() {
        return "tuple access";
    }

    @Override
    public void setName(String name) {
        throw new RuntimeException("setName is not implemented");
    }

    @Override
    public List<Param> getParams() {
        return List.of(new Param(parent, false, null));
    }

    @Override
    public void setParams(List<Param> params) {
        throw new RuntimeException("setParams is not implemented");
    }

    @Override
    public Type getReturnType() {
        return returnType;
    }

    @Override
    public void setReturnType(Type type) {
        throw new RuntimeException("setReturnType is not implemented");
    }

    @Override
    public Type getParent() {
        return parent;
    }

    @Override
    public void setParent(Type parent) {
        throw new RuntimeException("setParent is not implemented");
    }

    @Override
    public boolean returnsValue() {
        return true;
    }

    @Override
    public boolean isPublic() {
        return true;
    }

    @Override
    public void setPublic(boolean isPublic) {
        throw new RuntimeException("setPublic is not implemented");
    }

    @Override
    public Statement toStmt(TestCase testCase, List<VarReference> args, VarReference returnValue) {
        Preconditions.checkArgument(args.size() == 2);
        return new TupleAccessStmt(testCase, args.get(0), args.get(1), returnValue, this);
    }
}
