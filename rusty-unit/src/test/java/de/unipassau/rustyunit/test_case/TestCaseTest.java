package de.unipassau.rustyunit.test_case;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.when;

import de.unipassau.rustyunit.hir.TyCtxt;
import de.unipassau.rustyunit.metaheuristics.operators.Crossover;
import de.unipassau.rustyunit.metaheuristics.operators.Mutation;
import de.unipassau.rustyunit.mir.MirAnalysis;
import de.unipassau.rustyunit.test_case.callable.StaticMethod;
import de.unipassau.rustyunit.type.AbstractStruct;
import de.unipassau.rustyunit.type.Generic;
import de.unipassau.rustyunit.type.Type;
import de.unipassau.rustyunit.type.prim.Int.ISize;
import de.unipassau.rustyunit.test_case.visitor.TestCaseVisitor;
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
  private TyCtxt hir;

  @Mock
  private MirAnalysis<TestCase> mir;

  @Mock
  private Mutation<TestCase> mutation;

  @Mock
  private Crossover<TestCase> crossover;

  @Mock
  private CallableSelector callableSelector;

  @BeforeEach
  void setUp() {
    testCase = new TestCase(2, hir, mutation, crossover, mir);
  }

  @Test
  void testInsertCallableWithSameGeneric() {
    Type generic_A = new Generic("A", Collections.emptyList());
    Type parent = new AbstractStruct("MyType", List.of(generic_A), true);
    Type vecType = new AbstractStruct("std::vec::Vec", List.of(generic_A), false);

    var params = List.of(
        new Param(generic_A, false, "x"),
        new Param(vecType, false, "v")
    );

    var callableUnderTest = new StaticMethod("a", params, ISize.INSTANCE, parent, "");

    var vecCallable = new StaticMethod("new", Collections.emptyList(), vecType, vecType, "");
    when(hir.generatorsOf(any(), null)).thenReturn(List.of(vecCallable));

    testCase.insertCallable(callableUnderTest, "");

    var visitor = new TestCaseVisitor();
    System.out.println(testCase.visit(visitor));
  }

  @Test
  void testInsertVecConstructor() {
    var genericT = new Generic("T", Collections.emptyList());
    var vecType = new AbstractStruct("std::vec::Vec", Collections.singletonList(genericT), false);
    var vecConstructor = new StaticMethod("new", Collections.emptyList(), vecType, vecType, "");
    testCase.insertCallable(vecConstructor, "");

    var visitor = new TestCaseVisitor();
    System.out.println(testCase.visit(visitor));
  }


}