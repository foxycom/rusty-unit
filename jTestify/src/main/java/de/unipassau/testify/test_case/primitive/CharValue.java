package de.unipassau.testify.test_case.primitive;

import de.unipassau.testify.Constants;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.prim.Char;
import de.unipassau.testify.test_case.type.prim.Prim;
import de.unipassau.testify.test_case.type.prim.Str;
import de.unipassau.testify.util.Rnd;

public class CharValue implements PrimitiveValue<Character> {
  private final Prim type = Char.INSTANCE;
  private char value;

  public CharValue(char value) {
    this.value = value;
  }

  @Override
  public Character get() {
    return value;
  }

  @Override
  public Prim type() {
    return type;
  }

  @Override
  public PrimitiveValue<Character> delta() {
    var delta = Rnd.get().nextInt(2 * Constants.MAX_DELTA) - Constants.MAX_DELTA;
    return new CharValue((char) (value + delta));
  }

  @Override
  public PrimitiveValue<?> copy() {
    return new CharValue(value);
  }

  @Override
  public boolean isChar() {
    return true;
  }

  @Override
  public CharValue asChar() {
    return this;
  }

  @Override
  public String toString() {
    return String.format("'%s'", value);
  }
}