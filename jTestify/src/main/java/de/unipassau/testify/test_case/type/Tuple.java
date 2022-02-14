package de.unipassau.testify.test_case.type;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import com.google.common.collect.Streams;
import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;
import org.javatuples.Pair;

@JsonDeserialize(as = Tuple.class)
public class Tuple implements Type {

  private List<Type> types;

  public Tuple() {
  }

  public Tuple(Tuple other) {
    this.types = other.types.stream().map(Type::copy).peek(Objects::requireNonNull).toList();
  }

  public Tuple(List<Type> types) {
    this.types = types;
  }

  public List<Type> getTypes() {
    return types;
  }

  @Override
  public boolean isTuple() {
    return true;
  }

  @Override
  public Tuple asTuple() {
    return this;
  }

  public void setTypes(List<Type> types) {
    this.types = types;
  }

  @Override
  public String getName() {
    return "tuple";
  }

  @Override
  public void setName(String name) {

  }

  @Override
  public String fullName() {
    return "tuple";
  }

  @Override
  public String varString() {
    return "tuple";
  }

  @Override
  public boolean canBeSameAs(Type other) {
    if (other.isTuple()) {
      var otherTuple = other.asTuple();
      return types.size() == otherTuple.types.size()
          && Streams.zip(types.stream(), otherTuple.types.stream(), Pair::with)
          .allMatch(pair -> pair.getValue0().canBeSameAs(pair.getValue1()));
    } else {
      return other.isGeneric();
    }
  }

  @Override
  public boolean wraps(Type type) {
    return types.stream()
        .anyMatch(innerType -> innerType.canBeSameAs(type) || innerType.wraps(type));
  }

  @Override
  public boolean canBeIndirectlySameAs(Type other) {
    return types.stream().anyMatch(ty -> ty.canBeSameAs(other) || ty.canBeIndirectlySameAs(other));
  }

  @Override
  public List<Type> generics() {
    return types;
  }

  @Override
  public void setGenerics(List<Type> generics) {
    throw new RuntimeException("setGenerics is not implemented");
  }

  @Override
  public Type replaceGenerics(List<Type> generics) {
    return new Tuple(
        types.stream().map(ty -> ty.replaceGenerics(Objects.requireNonNull(generics))).toList()
    );
  }

  @Override
  public Type bindGenerics(TypeBinding binding) {
    var copy = new Tuple(this);
    if (binding.hasUnboundedGeneric()) {
      throw new RuntimeException("Unbounded generics");
    }

    copy.types = types.stream().map(ty -> ty.bindGenerics(binding)).toList();
    return copy;
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    Tuple tuple = (Tuple) o;
    return types.equals(tuple.types);
  }

  @Override
  public int hashCode() {
    return Objects.hash(types);
  }

  @Override
  public Type copy() {
    return new Tuple(this);
  }

  @Override
  public String toString() {
    var innerTypes = types.stream().map(Type::toString).collect(Collectors.joining(", "));
    return String.format("(%s)", innerTypes);
  }
}
