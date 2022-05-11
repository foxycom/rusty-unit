package de.unipassau.testify.ddg;

import de.unipassau.testify.test_case.var.VarReference;
import de.unipassau.testify.test_case.type.Generic;
import de.unipassau.testify.test_case.type.Type;
import java.util.UUID;

public interface Node {
  UUID asStmt();
  VarReference asVar();
  TypeBinding asTypeBinding();
  Generic asGeneric();
  Type asConcreteType();

  boolean isStmt();
  boolean isVar();
  boolean isTypeBinding();
  boolean isGeneric();
  boolean isConcreteType();
}
