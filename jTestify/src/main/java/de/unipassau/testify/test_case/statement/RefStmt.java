package de.unipassau.testify.test_case.statement;

import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.callable.RefItem;
import de.unipassau.testify.test_case.type.Type;
import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.UUID;

public class RefStmt implements Statement {
  private VarReference arg;
  private RefItem refItem;
  private TestCase testCase;
  private VarReference returnValue;
  private UUID id;

  public RefStmt(TestCase testCase, VarReference arg, VarReference returnValue, RefItem refItem) {
    this.testCase = testCase;
    this.arg = arg;
    this.refItem = refItem;
    this.returnValue = returnValue;
    this.id = UUID.randomUUID();
  }

  @Override
  public UUID id() {
    return id;
  }

  @Override
  public Optional<Type> returnType() {
    return Optional.of(refItem.getReturnType());
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
    return Collections.singletonList(arg);
  }

  @Override
  public void setArgs(List<VarReference> args) {
    if (args.size() != 1) {
      throw new RuntimeException("There should be exactly one arg");
    }

    this.arg = args.get(0);
  }

  @Override
  public void setArg(int pos, VarReference var) {
    if (pos != 0) {
      throw new RuntimeException("Something is wrong");
    }
    this.arg = var;
  }

  @Override
  public List<Param> params() {
    return refItem.getParams();
  }

  @Override
  public List<Type> actualParamTypes() {
    return Collections.singletonList(arg.type());
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
    return true;
  }

  @Override
  public boolean isRefStmt() {
    return true;
  }

  @Override
  public RefStmt asRefStmt() {
    return this;
  }

  @Override
  public boolean consumes(VarReference var) {
    return false;
  }

  public VarReference arg() {
    return arg;
  }

  @Override
  public boolean uses(VarReference var) {
    return arg.equals(var);
  }

  @Override
  public boolean borrows(VarReference var) {
    return arg.equals(var);
  }

  @Override
  public boolean mutates(VarReference var) {
    throw new RuntimeException("mutates is not implemented");
  }

  @Override
  public void replace(VarReference oldVar, VarReference newVar) {
    if (!arg.equals(oldVar)) {
      throw new RuntimeException("Statement does not use this var");
    }

    this.arg = newVar;
  }

  @Override
  public Statement copy(TestCase testCase) {
    var returnValueCopy = returnValue.copy(testCase);
    var argCopy = arg.copy(testCase);
    return new RefStmt(testCase, argCopy, returnValueCopy, refItem);
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
    if (!(o instanceof RefStmt)) {
      return false;
    }
    RefStmt refStmt = (RefStmt) o;
    return arg.equals(refStmt.arg) && refItem == refStmt.refItem && returnValue.equals(
        refStmt.returnValue) && id.equals(refStmt.id);
  }

  @Override
  public int hashCode() {
    return Objects.hash(arg, refItem, returnValue, id);
  }
}
