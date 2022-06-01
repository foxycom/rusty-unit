package de.unipassau.rustyunit.linearity;

import java.util.List;

public interface Operator {
  String name();
  List<Integer> parents();
}
