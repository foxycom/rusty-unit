package de.unipassau.testify.test_case.callable;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.json.CallableDeserializer;
import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.var.VarReference;
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
  boolean isPublic();

  void setPublic(boolean isPublic);
  Statement toStmt(TestCase testCase, List<VarReference> args, VarReference returnValue);

  default String globalId() {
    return null;
  }


  default boolean isMethod() {
    return false;
  }

  default Method asMethod() {
    throw new RuntimeException("Not with me");
  }

  default boolean isTupleAccess() {
    return false;
  }

  default TupleAccess asTupleAccess() {
    throw new RuntimeException("Not with me");
  }

  default boolean generates(Type type) {
    return false;
  }

  default String getSrcFilePath() {
    throw new RuntimeException("Not with me");
  }
  default void setSrcFilePath(String path) {
    throw new RuntimeException("Not with me");
  }
}
