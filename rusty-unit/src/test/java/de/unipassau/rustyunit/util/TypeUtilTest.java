package de.unipassau.rustyunit.util;

import static com.google.common.truth.Truth.assertThat;
import static de.unipassau.rustyunit.util.TypeUtil.getDeepGenerics;

import de.unipassau.rustyunit.test_case.type.AbstractStruct;
import de.unipassau.rustyunit.test_case.type.Generic;
import java.util.Collections;
import java.util.List;
import org.junit.jupiter.api.Test;

class TypeUtilTest {

  @Test
  void testGetDeepGenerics() {
    var genericA = new Generic("A", Collections.emptyList());
    var genericB = new Generic("B", Collections.emptyList());

    var vecType = new AbstractStruct("Vec", List.of(
        genericB
    ), false, Collections.emptySet());

    var hashMapType = new AbstractStruct(
        "HashMap",
        List.of(genericA, vecType),
        false,
        Collections.emptySet());

    assertThat(getDeepGenerics(hashMapType)).containsExactly(genericA, genericB);
  }
}