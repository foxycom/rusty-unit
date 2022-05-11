package de.unipassau.rustyunit.type;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import com.google.common.collect.Streams;
import de.unipassau.rustyunit.test_case.callable.Callable;
import de.unipassau.rustyunit.test_case.callable.TupleAccess;
import de.unipassau.rustyunit.type.traits.Trait;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.Set;
import java.util.stream.Collectors;
import java.util.stream.IntStream;
import lombok.EqualsAndHashCode;
import org.javatuples.Pair;

@EqualsAndHashCode
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
  public Optional<Integer> wraps(Type type) {
    var res = IntStream.range(0, types.size())
          .filter(i -> types.get(i).canBeSameAs(type) || types.get(i).wraps(type).isPresent())
          .findFirst();
    if (res.isPresent()) {
      return Optional.of(res.getAsInt());
    } else {
      return Optional.empty();
    }
  }

  @Override
  public Callable unwrapMethod(int at) {
    return new TupleAccess(this, types.get(at));
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
  public Set<Trait> implementedTraits() {
    throw new RuntimeException("implementedTraits is not implemented");
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
  public Type copy() {
    return new Tuple(this);
  }

  @Override
  public String encode() {
    var innerTypes = types.stream().map(Type::toString).collect(Collectors.joining(", "));
    return String.format("(%s)", innerTypes);
  }

  @Override
  public String toString() {
    return encode();
  }
}
