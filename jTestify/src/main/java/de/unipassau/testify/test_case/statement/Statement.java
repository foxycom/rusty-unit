package de.unipassau.testify.test_case.statement;

import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.type.Type;
import java.util.List;
import java.util.Optional;
import java.util.UUID;

public interface Statement {
  UUID id();
  Optional<Type> returnType();
  Optional<VarReference> returnValue();
  boolean returnsValue();

  default boolean isPrimitiveStmt() {
    return false;
  }

  default boolean isEnumStmt() {
    return false;
  }

  default boolean isStructInitStmt() {
    return false;
  }

  default boolean isCallableStmt() {
    return false;
  }

  default boolean isStaticMethodStmt() {
    return false;
  }

  default boolean isMethodStmt() {
    return false;
  }

  default PrimitiveStmt asPrimitiveStmt() {
    throw new RuntimeException("Not with me");
  }

  default StructInitStmt asStructInitStmt() {
    throw new RuntimeException("Not with me");
  }

  default CallableStmt asCallableStmt() {
    throw new RuntimeException("Not with me");
  }

  default StaticMethodStmt asStaticMethodStmt() {
    throw new RuntimeException("Not with me");
  }

  default MethodStmt asMethodStmt() {
    throw new RuntimeException("Not with me");
  }

  default EnumStmt asEnumStmt() {
    throw new RuntimeException("Not with me");
  }

  default boolean consumes(VarReference var) {
    return false;
  }

  default boolean borrows(VarReference var) {
    return false;
  }

  default boolean mutates(VarReference var) {
    return false;
  }
}
