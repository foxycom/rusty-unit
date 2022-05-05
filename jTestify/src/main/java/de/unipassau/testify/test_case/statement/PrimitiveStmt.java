package de.unipassau.testify.test_case.statement;

import de.unipassau.testify.Constants;
import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.var.VarReference;
import de.unipassau.testify.test_case.callable.Callable;
import de.unipassau.testify.test_case.primitive.PrimitiveValue;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.util.Rnd;
import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.UUID;

public class PrimitiveStmt implements Statement {

  private final UUID id;
  private VarReference varReference;
  private PrimitiveValue<?> value;
  private TestCase testCase;

  public PrimitiveStmt(TestCase testCase, VarReference varReference, PrimitiveValue<?> value) {
    this.id = UUID.randomUUID();
    this.testCase = testCase;
    this.varReference = varReference;
    this.value = value;
  }

  public PrimitiveValue<?> getValue() {
    return value;
  }

  @Override
  public UUID id() {
    return null;
  }

  @Override
  public Optional<Type> returnType() {
    return Optional.of(varReference.type());
  }

  @Override
  public Optional<VarReference> returnValue() {
    return Optional.of(varReference);
  }

  @Override
  public boolean returnsValue() {
    return true;
  }

  @Override
  public List<VarReference> args() {
    return Collections.emptyList();
  }

  @Override
  public void setArgs(List<VarReference> args) {
    if (!args.isEmpty()) {
      throw new RuntimeException("There should be no args");
    }
  }

  @Override
  public void setArg(int pos, VarReference var) {

  }

  @Override
  public List<Param> params() {
    return Collections.emptyList();
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
    return true;
  }

  @Override
  public boolean isPrimitiveStmt() {
    return true;
  }

  @Override
  public PrimitiveStmt asPrimitiveStmt() {
    return this;
  }

  @Override
  public Callable getCallable() {
    throw new RuntimeException("Not implemented");
  }

  @Override
  public boolean consumes(VarReference var) {
    return false;
  }

  @Override
  public boolean borrows(VarReference var) {
    return false;
  }

  @Override
  public boolean mutates(VarReference var) {
    throw new RuntimeException("mutates is not implemented");
  }

  @Override
  public boolean mutate(TestCase testCase) {
    var oldValue = value;
    while (value == oldValue && value != null) {
      if (Rnd.get().nextDouble() <= Constants.P_RANDOM_PERTURBATION) {
        if (value.isInt()) {
          if (Rnd.get().nextDouble() <= Constants.P_RANDOM_PERTURBATION) {
            value = value.asInt().negate();
          } else {
            value = value.type().random();
          }
        } else if (value.isFloat()) {
          if (Rnd.get().nextDouble() <= Constants.P_RANDOM_PERTURBATION) {
            value = value.asFloat().negate();
          } else {
            value = value.type().random();;
          }
        } else {
          value = value.type().random();
        }
      } else {
        value = value.delta();
      }
    }

    return true;
  }

  @Override
  public void replace(VarReference oldVar, VarReference newVar) {
    throw new RuntimeException("Not with me");
  }

  @Override
  public Statement copy(TestCase testCase) {
    var var = varReference.copy(testCase);
    return new PrimitiveStmt(testCase, var, value.copy());
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
    if (!(o instanceof PrimitiveStmt)) {
      return false;
    }
    PrimitiveStmt that = (PrimitiveStmt) o;
    return id.equals(that.id) && varReference.equals(that.varReference) && value.equals(that.value);
  }

  @Override
  public int hashCode() {
    return Objects.hash(id, varReference, value);
  }
}
