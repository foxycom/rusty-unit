package de.unipassau.testify.test_case.type.std;

import de.unipassau.testify.test_case.type.Generic;
import de.unipassau.testify.test_case.type.Struct;
import java.util.Collections;
import java.util.List;

class Vec extends Struct {

  public Vec() {
    super("std::vec::Vec", List.of(new Generic("T", Collections.emptyList())), false);
  }
}