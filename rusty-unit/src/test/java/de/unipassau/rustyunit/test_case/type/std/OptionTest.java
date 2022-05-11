package de.unipassau.rustyunit.test_case.type.std;

import static org.junit.jupiter.api.Assertions.*;

import de.unipassau.rustyunit.test_case.type.AbstractEnum;
import de.unipassau.rustyunit.test_case.type.Generic;
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
    var actualOption = new AbstractEnum("std::option::Option",
        List.of(new Generic("T", Collections.emptyList())),
        Collections.emptyList(),
        false, Collections.emptySet());
    assertTrue(genericOption.canBeSameAs(actualOption));
  }
}