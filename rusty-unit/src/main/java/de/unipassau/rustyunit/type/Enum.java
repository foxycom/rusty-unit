package de.unipassau.rustyunit.type;

import de.unipassau.rustyunit.type.AbstractEnum.EnumVariant;
import java.util.List;
import java.util.Locale;
import java.util.stream.Collectors;
import java.util.stream.IntStream;

public interface Enum extends Type {

  @Override
  default String varString() {
    var segments = getName().split("::");
    return segments[segments.length - 1].toLowerCase(Locale.ROOT);
  }

  @Override
  default boolean isEnum() {
    return true;
  }

  @Override
  default Enum asEnum() {
    return this;
  }

  @Override
  default boolean canBeSameAs(Type other) {
    if (other.isEnum()) {
      return isSameEnum(other.asEnum()) &&
          generics().size() == other.generics().size() &&
          IntStream.range(0, generics().size()).allMatch(i ->
              generics().get(i).canBeSameAs(other.generics().get(i)));
    } else if (other.isGeneric()) {
      return implementedTraits().containsAll(other.asGeneric().getBounds());
    } else {
      return false;
    }

  }

  default boolean isSameEnum(Enum other) {
    return isLocal() == other.isLocal() && getName().equals(other.getName());
  }

  @Override
  default boolean canBeIndirectlySameAs(Type other) {
    return variants().stream().anyMatch(v -> v.getParams().stream()
        .anyMatch(p -> p.type().equals(other) || p.type().canBeIndirectlySameAs(other)));
  }

  boolean isLocal();

  List<EnumVariant> variants();

  @Override
  default String encode() {
    var sb = new StringBuilder(getName());
    if (!generics().isEmpty()) {
      sb.append("<");
      var genericNames = generics().stream().map(Type::toGenericString)
          .collect(Collectors.joining(", "));
      sb.append(genericNames);
      sb.append(">");
    }
    return sb.toString();
  }
}
