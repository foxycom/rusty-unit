package de.unipassau.testify.test_case.statement;

import de.unipassau.testify.test_case.Primitive;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.type.Type;
import java.util.Optional;
import java.util.UUID;

public class PrimitiveStmt implements Statement {

  private UUID id;
  private VarReference varReference;
  private Primitive value;

  public PrimitiveStmt(VarReference varReference, Primitive value) {
    this.id = UUID.randomUUID();
    this.varReference = varReference;
    this.value = value;
  }

  public Primitive getValue() {
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
  public boolean isPrimitiveStmt() {
    return true;
  }

  @Override
  public PrimitiveStmt asPrimitiveStmt() {
    return this;
  }
}
