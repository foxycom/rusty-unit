package de.unipassau.testify.test_case.type;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import java.util.List;
import java.util.Locale;
import java.util.Objects;
import java.util.stream.Collectors;
import java.util.stream.IntStream;

@JsonDeserialize(as = Complex.class)
public class Complex implements Type {

  private String name;
  private List<Type> generics;
  @JsonProperty("is_local")
  private boolean isLocal;

  public Complex() {

  }

  public Complex(Complex other) {
    this.name = other.name;
    this.isLocal = other.isLocal;
    this.generics = other.generics.stream().map(Type::copy).peek(Objects::requireNonNull).toList();
  }

  public Complex(String name, List<Type> generics, boolean isLocal) {
    Objects.requireNonNull(generics).forEach(Objects::requireNonNull);

    this.name = name;
    this.generics = generics;
    this.isLocal = isLocal;
  }

  @Override
  public String getName() {
    return name;
  }

  @Override
  public void setName(String name) {
    this.name = name;
  }

  @Override
  public String fullName() {
    if (isLocal) {
      return String.format("crate::%s", name);
    } else {
      return name;
    }
  }

  @Override
  public boolean isComplex() {
    return true;
  }

  @Override
  public Complex asComplex() {
    return this;
  }

  @Override
  public String varString() {
    var segments = name.split("::");
    return segments[segments.length - 1].toLowerCase(Locale.ROOT);
  }

  @Override
  public boolean canBeSameAs(Type other) {
    if (other.isComplex()) {
      return isSameType(other.asComplex()) &&
          generics.size() == other.generics().size() &&
          IntStream.range(0, generics.size())
              .allMatch(i -> generics.get(i).canBeSameAs(other.generics().get(i)));
    } else {
      return other.isGeneric();
    }
  }

  @Override
  public boolean canBeIndirectlySameAs(Type other) {
    return canBeSameAs(other);
  }

  public boolean isSameType(Complex other) {
    return name.equals(other.name) && isLocal == other.isLocal;
  }

  @Override
  public List<Type> generics() {
    return generics;
  }

  @Override
  public void setGenerics(List<Type> generics) {
    generics.forEach(Objects::requireNonNull);
    this.generics = generics;
  }

  @Override
  public Type replaceGenerics(List<Type> generics) {
    var copy = new Complex(this);
    generics.forEach(Objects::requireNonNull);
    copy.generics = generics;
    return copy;
  }

  @Override
  public Type bindGenerics(TypeBinding binding) {
    var copy = new Complex(this);
    if (binding.hasUnboundedGeneric()) {
      throw new RuntimeException("Unbounded generics");
    }

    copy.generics = generics.stream().map(g -> g.bindGenerics(binding)).toList();
    return copy;
  }


  @Override
  public Type copy() {
    return new Complex(this);
  }

  public boolean isLocal() {
    return isLocal;
  }

  public void setLocal(boolean local) {
    isLocal = local;
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    Complex complex = (Complex) o;
    return isLocal == complex.isLocal
        && name.equals(complex.name)
        && generics.equals(complex.generics);
  }

  @Override
  public int hashCode() {
    return Objects.hash(name, generics, isLocal);
  }

  @Override
  public String toString() {
    generics.forEach(Objects::requireNonNull);

    var sb = new StringBuilder(fullName());
    if (!generics.isEmpty()) {
      sb.append("<");
      var genericsStr = generics.stream().map(Type::toGenericString).collect(Collectors.joining(", "));
      sb.append(genericsStr);
      sb.append(">");
    }
    return sb.toString();
  }
}
