package de.unipassau.testify.test_case.statement;

import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.callable.EnumInit;
import de.unipassau.testify.test_case.type.Enum;
import de.unipassau.testify.test_case.type.Enum.EnumVariant;
import de.unipassau.testify.test_case.type.Type;
import java.util.List;
import java.util.Optional;
import java.util.UUID;

public class EnumStmt implements Statement {

  private UUID id;
  private VarReference returnValue;

  private List<VarReference> args;
  private TestCase testCase;
  private EnumInit enumInit;

  public EnumStmt(TestCase testCase, List<VarReference> args, VarReference returnValue, EnumInit enumInit) {
    this.id = UUID.randomUUID();
    this.enumInit = enumInit;
    this.testCase = testCase;
    this.returnValue = returnValue;
    this.args = args;
  }

  public VarReference getReturnValue() {
    return returnValue;
  }

  public Enum getType() {
    return enumInit.getReturnType().asEnum();
  }

  public EnumVariant getVariant() {
    return enumInit.getVariant();
  }

  public List<VarReference> getArgs() {
    return args;
  }

  public EnumInit getEnumInit() {
    return enumInit;
  }

  @Override
  public UUID id() {
    return id;
  }

  @Override
  public Optional<Type> returnType() {
    return Optional.of(enumInit.getReturnType());
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
  public boolean isEnumStmt() {
    return true;
  }

  @Override
  public EnumStmt asEnumStmt() {
    return this;
  }
}
