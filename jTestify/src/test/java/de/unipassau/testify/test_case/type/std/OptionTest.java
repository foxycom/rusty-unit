package de.unipassau.testify.test_case.type.std;

import static com.google.common.truth.ExpectFailure.assertThat;
import static org.junit.jupiter.api.Assertions.*;

import de.unipassau.testify.test_case.type.Enum;
import de.unipassau.testify.test_case.type.Generic;
import java.util.Collections;
import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

class OptionTest {

  @BeforeEach
  void setUp() {

  }

  @Test
  public void testGenericOptionCanBeOptionOfSomeType() {
    var genericOption = new Option();
    var actualOption = new Enum("std::option::Option",
        List.of(new Generic("T", Collections.emptyList())),
        Collections.emptyList(),
        false, Collections.emptySet());
    assertTrue(genericOption.canBeSameAs(actualOption));
  }
}