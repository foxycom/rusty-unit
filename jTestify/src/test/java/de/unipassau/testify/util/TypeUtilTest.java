package de.unipassau.testify.util;

import static com.google.common.truth.Truth.assertThat;
import static de.unipassau.testify.util.TypeUtil.getDeepGenerics;
import static org.junit.jupiter.api.Assertions.*;

import de.unipassau.testify.test_case.type.Complex;
import de.unipassau.testify.test_case.type.Generic;
import java.util.Collections;
import java.util.List;
import org.junit.jupiter.api.Test;

class TypeUtilTest {

  @Test
  void testGetDeepGenerics() {
    var genericA = new Generic("A", Collections.emptyList());
    var genericB = new Generic("B", Collections.emptyList());

    var vecType = new Complex("Vec", List.of(
        genericB
    ), false);

    var hashMapType = new Complex(
        "HashMap",
        List.of(genericA, vecType),
        false);

    assertThat(getDeepGenerics(hashMapType)).containsExactly(genericA, genericB);
  }
}