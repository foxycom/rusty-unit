package de.unipassau.testify.test_case.type;

import static com.google.common.truth.Truth.assertThat;
import static org.junit.jupiter.api.Assertions.*;

import de.unipassau.testify.test_case.type.prim.Int.ISize;
import java.util.Collections;
import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

class ComplexTest {

  @BeforeEach
  void setUp() {
  }

  @Test
  void testGenericCanBeSameAsConcreteInstance() {
    var concreteNodeType = new Complex("trie::Node", List.of(ISize.INSTANCE,   ISize.INSTANCE), true);

    var genericNodeType = new Complex("trie::Node", List.of(
        new Generic("A", Collections.emptyList()),
        new Generic("V", Collections.emptyList())
        ), true);

    assertThat(concreteNodeType.canBeSameAs(genericNodeType)).isTrue();
  }
}