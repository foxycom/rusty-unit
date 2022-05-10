package de.unipassau.testify.test_case.type;

import static com.google.common.truth.Truth.assertThat;
import static org.junit.jupiter.api.Assertions.*;

import java.util.Collections;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

class RefTest {

  Ref ref;

  @BeforeEach
  void setUp() {
    Type innerType = new AbstractStruct("Struct", Collections.emptyList(), true);
    ref = new Ref(innerType, false);
  }

  @Test
  public void testRefCanBeSameAsItsInnerValue() {
    var other = new AbstractStruct("Struct", Collections.emptyList(), true);
    assertThat(ref.canBeSameAs(other)).isFalse();
  }
}