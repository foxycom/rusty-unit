package de.unipassau.rustyunit.test_case.primitive;

import de.unipassau.rustyunit.Constants;
import de.unipassau.rustyunit.type.prim.Char;
import de.unipassau.rustyunit.type.prim.Prim;
import de.unipassau.rustyunit.util.Rnd;
import java.util.List;
import java.util.Objects;

public class CharValue implements PrimitiveValue<Character> {
  private final Prim type = Char.INSTANCE;
  private char value;

  private static final List<Character> ESCAPABLE_CHARS = List.of('r', 'n', '\'', 't', '\\', '\0');

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

  private boolean needsEscaping(char c) {
    return ESCAPABLE_CHARS.contains(c);
  }

  @Override
  public String toString() {
    if (needsEscaping(value)) {
      return String.format("'\\%s'", value);
    } else {
      return String.format("'%s'", value);
    }
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof CharValue)) {
      return false;
    }
    CharValue charValue = (CharValue) o;
    return value == charValue.value && type.equals(charValue.type);
  }

  @Override
  public int hashCode() {
    return Objects.hash(type, value);
  }
}
