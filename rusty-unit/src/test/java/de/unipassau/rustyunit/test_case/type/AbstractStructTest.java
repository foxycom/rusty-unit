package de.unipassau.rustyunit.test_case.type;

import static com.google.common.truth.Truth.assertThat;

import de.unipassau.rustyunit.type.AbstractStruct;
import de.unipassau.rustyunit.type.Generic;
import de.unipassau.rustyunit.type.prim.Int.ISize;
import java.util.Collections;
import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

class AbstractStructTest {

  @BeforeEach
  void setUp() {
  }

  @Test
  void testGenericCanBeSameAsConcreteInstance() {
    var concreteNodeType = new AbstractStruct("trie::Node", List.of(ISize.INSTANCE,   ISize.INSTANCE), true);

    var genericNodeType = new AbstractStruct("trie::Node", List.of(
        new Generic("A", Collections.emptyList()),
        new Generic("V", Collections.emptyList())
        ), true);

    assertThat(concreteNodeType.canBeSameAs(genericNodeType)).isTrue();
  }
}