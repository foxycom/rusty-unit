package de.unipassau.testify.test_case;

import static org.mockito.Mockito.when;

import de.unipassau.testify.hir.HirAnalysis;
import de.unipassau.testify.metaheuristics.operators.Crossover;
import de.unipassau.testify.metaheuristics.operators.Mutation;
import de.unipassau.testify.test_case.callable.StaticMethod;
import de.unipassau.testify.test_case.statement.StaticMethodStmt;
import de.unipassau.testify.test_case.type.Complex;
import de.unipassau.testify.test_case.type.Generic;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.prim.Int.ISize;
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
    when(analysis.generatorsOf(vecType)).thenReturn(List.of(vecCallable));

    testCase.insertCallable(callableUnderTest);

    var visitor = new TestCaseVisitor();
    System.out.println(testCase.visit(visitor));
  }
}