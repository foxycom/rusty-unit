package de.unipassau.testify.test_case;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.when;

import de.unipassau.testify.hir.HirAnalysis;
import de.unipassau.testify.metaheuristics.operators.Crossover;
import de.unipassau.testify.metaheuristics.operators.Mutation;
import de.unipassau.testify.test_case.callable.EnumInit;
import de.unipassau.testify.test_case.callable.RefItem;
import de.unipassau.testify.test_case.callable.StaticMethod;
import de.unipassau.testify.test_case.callable.StructInit;
import de.unipassau.testify.test_case.statement.StaticMethodStmt;
import de.unipassau.testify.test_case.type.Complex;
import de.unipassau.testify.test_case.type.Enum;
import de.unipassau.testify.test_case.type.Enum.EnumVariant;
import de.unipassau.testify.test_case.type.Generic;
import de.unipassau.testify.test_case.type.Ref;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.prim.Int;
import de.unipassau.testify.test_case.type.prim.Int.ISize;
import de.unipassau.testify.test_case.type.prim.UInt.USize;
import java.util.Collections;
import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

@ExtendWith(MockitoExtension.class)
class TestCaseTest {

  private TestCase testCase;

  @Mock
  private HirAnalysis analysis;

  @Mock
  private Mutation<TestCase> mutation;

  @Mock
  private Crossover<TestCase> crossover;

  @BeforeEach
  void setUp() {
    testCase = new TestCase(2, analysis, mutation, crossover);
  }

  @Test
  void testInsertCallableWithSameGeneric() {

    Type generic_A = new Generic("A", Collections.emptyList());
    Type parent = new Complex("MyType", List.of(generic_A), true);
    Type vecType = new Complex("std::vec::Vec", List.of(generic_A), false);
    List<Param> params = List.of(
        new Param(generic_A, false, "x"),
        new Param(vecType, false, "v")
    );
    var callableUnderTest = new StaticMethod("a", params, ISize.INSTANCE, parent, 2);

    var vecCallable = new StaticMethod("new", Collections.emptyList(), vecType, vecType, 2);
    when(analysis.generatorsOf(any())).thenReturn(List.of(vecCallable));

    testCase.insertCallable(callableUnderTest);

    var visitor = new TestCaseVisitor();
    System.out.println(testCase.visit(visitor));
  }

  @Test
  void getId() {
  }

  @Test
  void setId() {
  }

  @Test
  void size() {
  }

  @Test
  void getStatements() {
  }

  @Test
  void setStatements() {
  }

  @Test
  void getCoverage() {
  }

  @Test
  void setCoverage() {
  }

  @Test
  void getDdg() {
  }

  @Test
  void isCutable() {
  }

  @Test
  void insertStmt() {
  }

  @Test
  void addStmt() {
  }

  @Test
  void removeStmt() {
  }

  @Test
  void testRemoveStmt() {
  }

  @Test
  void stmtPosition() {
  }

  @Test
  void varPosition() {
  }

  @Test
  void getName() {
  }

  @Test
  void instantiatedTypes() {
  }

  @Test
  void variables() {
  }

  @Test
  void getVar() {
  }

  @Test
  void unconsumedVariablesOfType() {
  }

  @Test
  void variablesOfType() {
  }

  @Test
  void availableCallables() {
  }

  @Test
  void insertRandomStmt() {
  }

  @Test
  void testInsertVecConstructor() {
    var genericT = new Generic("T", Collections.emptyList());
    var vecType = new Complex("std::vec::Vec", Collections.singletonList(genericT), false);
    var vecConstructor = new StaticMethod("new", Collections.emptyList(), vecType, vecType, 3);
    testCase.insertCallable(vecConstructor);

    var visitor = new TestCaseVisitor();
    System.out.println(testCase.visit(visitor));
  }

  @Test
  void testGenerateOptionOfUsizeRef() {
    var genericVariant = new Enum.EnumVariant("Some", List.of(
        new Param(new Generic("T", Collections.emptyList()), false, null)
    ));
    var genericOption = new Enum("Option",
        List.of(new Generic("T", Collections.emptyList())),
        List.of(genericVariant),
        false
    );

    var option = new Enum(
        "Option",
        List.of(new Ref(USize.INSTANCE)),
        List.of(
            new EnumVariant("Some", List.of(
                new Param(
                    new Ref(USize.INSTANCE),
                    false,
                    null
                )
            ))
        ),
        false
    );

    var params = List.of(
        new Param(
            option, false, "value"
        )
    );

    var structType = new Complex("SomeStruct", Collections.emptyList(), true);
    var structInit = new StructInit(params, structType, 2);

    var enumGenerator = new EnumInit(genericOption, genericVariant);
    var refGenerator = new RefItem(new Param(new Generic("T", Collections.emptyList()), true, null));
    when(analysis.generatorsOf(any(Enum.class))).thenReturn(List.of(enumGenerator));
    when(analysis.generatorsOf(any(Ref.class))).thenReturn(List.of(refGenerator));

    testCase.insertCallable(structInit);
    var visitor = new TestCaseVisitor();
    System.out.println(testCase.visit(visitor));
  }

  @Test
  void generateArg() {
  }

  @Test
  void visit() {
  }

  @Test
  void testToString() {
  }

  @Test
  void copy() {
  }

  @Test
  void testEquals() {
  }

  @Test
  void testHashCode() {
  }

  @Test
  void self() {
  }
}