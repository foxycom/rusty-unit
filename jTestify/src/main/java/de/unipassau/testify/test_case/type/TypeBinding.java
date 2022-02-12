package de.unipassau.testify.test_case.type;

import com.google.common.collect.Streams;
import java.util.Collections;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Map.Entry;
import java.util.Objects;
import java.util.Set;
import java.util.stream.Collectors;
import org.javatuples.Pair;

public class TypeBinding {

  private Map<Generic, Type> binding;

  public TypeBinding(Set<Generic> generics, List<Type> actualTypes) {
    if (generics.size() != actualTypes.size()) {
      throw new IllegalArgumentException("Lengths should be equal");
    }

    binding = new HashMap<>();
    Streams.zip(generics.stream(), actualTypes.stream(), Pair::with)
        .forEach(pair -> binding.put(pair.getValue0(), pair.getValue1()));
  }

  public TypeBinding(TypeBinding other) {
    this.binding = other.binding.entrySet().stream()
        .map(e -> Pair.with(e.getKey().copy().asGeneric(), e.getValue().copy()))
        .collect(Collectors.toMap(Pair::getValue0, Pair::getValue1));
  }

  public TypeBinding(Set<Generic> generics) {
    this(generics, generics.stream().map(g -> (Type) null).toList());
  }

  public TypeBinding() {
    binding = new HashMap<>();
  }

  public boolean hasBindingFor(Generic generic) {
    return binding.containsKey(generic) && binding.get(generic) != null;
  }

  public Set<Generic> getGenerics() {
    return binding.keySet();
  }

  public Set<Generic> getUnboundGenerics() {
    return binding.keySet().stream().filter(g -> binding.get(g) == null)
        .collect(Collectors.toUnmodifiableSet());
  }

  public Type getBindingFor(Generic generic) {
    if (!binding.containsKey(generic)) {
      // TODO: 12.02.22 remove
      System.out.println();
    }
    return Objects.requireNonNull(binding.get(generic), "Does not contain " + generic);
  }

  public void addGenerics(Set<Generic> generics) {
    generics.forEach(g -> binding.put(g, null));
  }

  public void addGeneric(Generic generic) {
    binding.put(generic, null);
  }

  public void bindGeneric(Generic generic, Type type) {
    binding.put(generic, type);
  }

  public List<Generic> unboundedGenerics() {
    return binding.entrySet().stream().filter(e -> e.getValue() == null).map(Entry::getKey)
        .toList();
  }

  public boolean hasUnboundedGeneric() {
    return binding.values().stream().anyMatch(Objects::isNull);
  }

  public TypeBinding merge(TypeBinding other) {
    var merged = new HashMap<>(binding);
    other.binding.forEach((key, value) -> merged.merge(key, value, (left, right) -> left));
    var typeBinding = new TypeBinding();
    typeBinding.binding = merged;
    return typeBinding;
  }

  public TypeBinding copy() {
    return new TypeBinding(this);
  }

  @Override
  public String toString() {
    return "{" + binding.entrySet().stream().map(e -> e.getKey() + " -> " + e.getValue())
        .collect(Collectors.joining(", "))
        + "}";
  }
}
