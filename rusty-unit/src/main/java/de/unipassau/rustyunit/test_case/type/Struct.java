package de.unipassau.rustyunit.test_case.type;

import java.util.List;
import java.util.Locale;
import java.util.Objects;
import java.util.stream.Collectors;
import java.util.stream.IntStream;

public interface Struct extends Type {
  @Override
  default boolean canBeSameAs(Type other) {
    if (other.isStruct()) {
      return isSameType(other.asStruct()) &&
          generics().size() == other.generics().size() &&
          IntStream.range(0, generics().size())
              .allMatch(i -> generics().get(i).canBeSameAs(other.generics().get(i)));
    } else {
      return other.isGeneric();
    }
  }

  @Override
  default boolean canBeIndirectlySameAs(Type other) {
    return canBeSameAs(other);
  }

  default boolean isSameType(Struct other) {
    return getName().equals(other.getName()) && isLocal() == other.isLocal();
  }

  @Override
  default String varString() {
    var segments = getName().split("::");
    return segments[segments.length - 1].toLowerCase(Locale.ROOT);
  }

  @Override
  default String fullName() {
    if (isLocal()) {
      return String.format("crate::%s", getName());
    } else {
      return getName();
    }
  }

  @Override
  default boolean isStruct() {
    return true;
  }

  @Override
  default Struct asStruct() {
    return this;
  }

  @Override
  default void setGenerics(List<Type> generics) {
    throw new RuntimeException("setGenerics is not implemented");
  }

  @Override
  default Type replaceGenerics(List<Type> generics) {
    throw new RuntimeException("replaceGenerics is not implemented");
  }

  boolean isLocal();

  @Override
  default String encode() {
    generics().forEach(Objects::requireNonNull);

    var sb = new StringBuilder(fullName());
    if (!generics().isEmpty()) {
      sb.append("<");
      var genericsStr = generics().stream().map(Type::toGenericString).collect(Collectors.joining(", "));
      sb.append(genericsStr);
      sb.append(">");
    }
    return sb.toString();
  }
}
