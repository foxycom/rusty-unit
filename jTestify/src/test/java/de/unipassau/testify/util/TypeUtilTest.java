package de.unipassau.testify.util;

import static com.google.common.truth.Truth.assertThat;
import static de.unipassau.testify.util.TypeUtil.getDeepGenerics;

import de.unipassau.testify.test_case.type.Struct;
import de.unipassau.testify.test_case.type.Generic;
import java.util.Collections;
import java.util.List;
import org.junit.jupiter.api.Test;

class TypeUtilTest {

  @Test
  void testGetDeepGenerics() {
    var genericA = new Generic("A", Collections.emptyList());
    var genericB = new Generic("B", Collections.emptyList());

    var vecType = new Struct("Vec", List.of(
        genericB
    ), false, Collections.emptySet());

    var hashMapType = new Struct(
        "HashMap",
        List.of(genericA, vecType),
        false,
        Collections.emptySet());

    assertThat(getDeepGenerics(hashMapType)).containsExactly(genericA, genericB);
  }
}