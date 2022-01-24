package de.unipassau.testify.test_case.statement;

import de.unipassau.testify.Constants;
import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.primitive.PrimitiveValue;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.util.Rnd;
import java.util.Collections;
import java.util.List;
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
}
