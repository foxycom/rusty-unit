package de.unipassau.rustyunit.test_case.primitive;

import de.unipassau.rustyunit.Constants;
import de.unipassau.rustyunit.test_case.type.prim.Prim;
import de.unipassau.rustyunit.test_case.type.prim.Str;
import de.unipassau.rustyunit.util.Rnd;
import java.util.Objects;
import org.apache.commons.lang3.RandomStringUtils;

public class StringValue implements PrimitiveValue<String> {

  private final Prim type = Str.INSTANCE;
  private String value;


  public StringValue(String value) {
    this.value = value;
  }

  @Override
  public String get() {
    return value;
  }

  @Override
  public Prim type() {
    return type;
  }

  @Override
  public PrimitiveValue<String> delta() {
    String result = value;
    int length = result.length();

    int i = 0;
    while (i < length) {
      if (Rnd.get().nextDouble() < 0.33) {
        // Replace a char
        var sb = new StringBuilder(result);
        sb.setCharAt(i, RandomStringUtils.randomAlphanumeric(1).charAt(0));
        result = sb.toString();
        i++;
      } else if (Rnd.get().nextDouble() < 0.33 && length > 0) {
        // Remove a char
        var sb = new StringBuilder(result);
        sb.deleteCharAt(i);
        result = sb.toString();
        length--;
      } else if (Rnd.get().nextDouble() < 0.33 && length < Constants.MAX_STRING_LENGTH) {
        // Add a char
        var sb = new StringBuilder(result);
        sb.insert(i, RandomStringUtils.randomAlphabetic(1).charAt(0));
        result = sb.toString();
        i += 2;
        length++;
      }
    }

    return new StringValue(result);
  }

  @Override
  public PrimitiveValue<?> copy() {
    return new StringValue(value);
  }

  @Override
  public boolean isString() {
    return true;
  }

  @Override
  public StringValue asString() {
    return this;
  }

  @Override
  public String toString() {
    return String.format("\"%s\"", value);
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof StringValue)) {
      return false;
    }
    StringValue that = (StringValue) o;
    return type.equals(that.type) && value.equals(that.value);
  }

  @Override
  public int hashCode() {
    return Objects.hash(type, value);
  }
}
