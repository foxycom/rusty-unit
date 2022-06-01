package de.unipassau.rustyunit.test_case.type;

import static com.google.common.truth.Truth.assertThat;

import de.unipassau.rustyunit.type.AbstractStruct;
import de.unipassau.rustyunit.type.Ref;
import de.unipassau.rustyunit.type.Type;
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