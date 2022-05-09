package de.unipassau.testify.test_case.operators;

import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.when;

import com.google.common.collect.Lists;
import de.unipassau.testify.hir.TyCtxt;
import de.unipassau.testify.metaheuristics.operators.Mutation;
import de.unipassau.testify.mir.MirAnalysis;
import de.unipassau.testify.test_case.CallableSelector;
import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.var.VarReference;
import de.unipassau.testify.test_case.callable.Method;
import de.unipassau.testify.test_case.callable.RefItem;
import de.unipassau.testify.test_case.callable.StaticMethod;
import de.unipassau.testify.test_case.primitive.UIntValue;
import de.unipassau.testify.test_case.statement.MethodStmt;
import de.unipassau.testify.test_case.statement.PrimitiveStmt;
import de.unipassau.testify.test_case.statement.RefStmt;
import de.unipassau.testify.test_case.statement.Statement;
import de.unipassau.testify.test_case.statement.StaticMethodStmt;
import de.unipassau.testify.test_case.type.AbstractStruct;
import de.unipassau.testify.test_case.type.Ref;
import de.unipassau.testify.test_case.type.prim.UInt.USize;
import de.unipassau.testify.test_case.visitor.CrossoverDebugVisitor;
import de.unipassau.testify.test_case.visitor.TestCaseVisitor;
import java.math.BigInteger;
import java.util.Collections;
import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

@ExtendWith(MockitoExtension.class)
class SinglePointFixedCrossoverTest {

  @Mock
  private Mutation<TestCase> mutation;

  private SinglePointFixedCrossover crossover;

  @Mock
  private TyCtxt hir;

  @Mock
  private MirAnalysis<TestCase> mir;

  @Mock
  private CallableSelector callableSelector;

  @BeforeEach
  void setUp() {
    crossover = new SinglePointFixedCrossover();
  }

  /*
   * let mut usize_0: usize = 34usize;
   * let mut address_0: Address = Address::new();
   * let mut usize_1: usize = 1234566usize;
   * let mut person_0: Person = Person::new(address_0, usize_0, usize_1);
   */
  List<Statement> getStatementsA(TestCase testCase) {
    var age = new VarReference(testCase, USize.INSTANCE);
    var ageStmt = new PrimitiveStmt(testCase, age, new UIntValue(BigInteger.valueOf(34), USize.INSTANCE));

    var addressType = new AbstractStruct("Address", Collections.emptyList(), true);
    var addressVar = new VarReference(testCase, addressType);
    var addressConstructor = new StaticMethod("new", Collections.emptyList(), addressType,
        addressType, "");
    var addressStmt = new StaticMethodStmt(testCase, Collections.emptyList(), addressVar,
        addressConstructor);

    var phone = new VarReference(testCase, USize.INSTANCE);
    var phoneStmt = new PrimitiveStmt(testCase, phone, new UIntValue(BigInteger.valueOf(1234566), USize.INSTANCE));

    var personType = new AbstractStruct("Person",
        Collections.emptyList(),
        true
    );
    var personVar = new VarReference(testCase, personType);
    var personConstructor = new StaticMethod("new",
        List.of(
            new Param(addressType, false, null),
            new Param(USize.INSTANCE, false, null),
            new Param(USize.INSTANCE, false, null)
        ),
        personType, personType, "");
    var personStmt = new StaticMethodStmt(testCase, List.of(
        addressVar, age, phone
    ), personVar, personConstructor);

    return List.of(ageStmt, addressStmt, phoneStmt, personStmt);
  }

  /*
   * let mut vec_0: Vec<usize> = Vec::new();
   * let mut vec_0_ref: &mut Vec<usize> = &mut vec_0;
   * let mut usize_0: usize = 45usize;
   * Vec::push(&mut vec_0, usize_0);
   * let mut grades_0: Grades = Grades::new(vec_0);
   */
  List<Statement> getStatementsB(TestCase testCase) {
    var vecType = new AbstractStruct("Vec", List.of(USize.INSTANCE), false);
    var vec = new VarReference(testCase, vecType);
    var vecConstructor = new StaticMethod("new", Collections.emptyList(), vecType, vecType, "");
    var vecStmt = new StaticMethodStmt(testCase, Collections.emptyList(), vec, vecConstructor);

    var vecRef = new VarReference(testCase, new Ref(vecType, true));
    var vecRefStmt = new RefStmt(testCase, vec, vecRef, RefItem.MUTABLE);

    var grade = new VarReference(testCase, USize.INSTANCE);
    var gradeStmt = new PrimitiveStmt(testCase, grade, new UIntValue(BigInteger.valueOf(45), USize.INSTANCE));

    var pushMethod = new Method("push", Collections.emptyList(),
        List.of(new Param(new Ref(vecType, true), false, null),
        new Param(USize.INSTANCE, false, null)
    ), null, vecType);
    var vecPushStmt = new MethodStmt(testCase, List.of(
        vecRef,
        grade
    ), null, pushMethod);

    var gradesType = new AbstractStruct("Grades", Collections.emptyList(), true);
    var grades = new VarReference(testCase, gradesType);
    var gradesConstructor = new StaticMethod("new", List.of(
        new Param(vecType, false, null)
    ), gradesType, gradesType, "");
    var gradesStmt = new StaticMethodStmt(testCase, List.of(vec), grades, gradesConstructor);

    return List.of(vecStmt, vecRefStmt, gradeStmt, vecPushStmt, gradesStmt);
  }

  @Test
  void test() {
    var visitor = new TestCaseVisitor();
    var debugVisitor = new CrossoverDebugVisitor(2);

    var parentA = new TestCase(1, hir, mutation, crossover, mir, callableSelector);
    parentA.setStatements(getStatementsA(parentA));

    System.out.println(parentA.visit(debugVisitor));

    var vecType = new AbstractStruct("Vec", List.of(USize.INSTANCE), false);
    var vecConstructor = new StaticMethod("new", Collections.emptyList(), vecType, vecType, "");
    var vecRefType = new Ref(vecType, true);

    when(hir.generatorsOf(vecRefType, ""))
        .thenReturn(Lists.newArrayList(RefItem.MUTABLE));

    when(hir.generatorsOf(vecType, "")).thenReturn(Lists.newArrayList(vecConstructor));

    var parentB = new TestCase(2, hir, mutation, crossover, mir, callableSelector);
    parentB.setStatements(getStatementsB(parentB));

    System.out.println(parentB.visit(debugVisitor));
    var childA = crossover.crossOver(parentA, parentB, 2);
    //System.out.println(childA.visit(visitor));
  }
}