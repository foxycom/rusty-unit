package de.unipassau.testify.test_case.callable;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.json.CallableDeserializer;
import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.VarReference;
import de.unipassau.testify.test_case.statement.Statement;
import de.unipassau.testify.test_case.type.Type;
import java.util.List;

@JsonDeserialize(using = CallableDeserializer.class)
public interface Callable {
  String getName();
  void setName(String name);
  List<Param> getParams();
  void setParams(List<Param> params);
  Type getReturnType();
  void setReturnType(Type type);
  Type getParent();
  void setParent(Type parent);
  boolean returnsValue();
  Statement toStmt(TestCase testCase, List<VarReference> args, VarReference returnValue);

  default boolean isMethod() {
    return false;
  }
}
