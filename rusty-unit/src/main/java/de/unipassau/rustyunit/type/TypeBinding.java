package de.unipassau.rustyunit.type;

import com.google.common.base.Preconditions;
import com.google.common.collect.Sets;
import com.google.common.collect.Streams;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Map.Entry;
import java.util.Objects;
import java.util.Set;
import java.util.stream.Collectors;
import org.javatuples.Pair;

public class TypeBinding {

  private Map<Generic, Type> binding;

  public TypeBinding(LinkedHashSet<Generic> generics, List<Type> actualTypes) {
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

  public TypeBinding(LinkedHashSet<Generic> generics) {
    this(generics, generics.stream().map(g -> (Type) null).toList());
  }

  public TypeBinding() {
    binding = new HashMap<>();
  }

  public static TypeBinding fromTypes(Type generic, Type concrete) {
    if (generic.isGeneric()) {
      LinkedHashSet<Generic> generics = Sets.newLinkedHashSet();
      generics.add(generic.asGeneric());
      return new TypeBinding(generics, List.of(concrete));
    } else {
      var combinations = Streams.zip(generic.generics().stream(), concrete.generics().stream(),
              Pair::with)
          .filter(p -> p.getValue0().isGeneric())
          .map(p -> Pair.with(p.getValue0().asGeneric(), p.getValue1())).toList();

      LinkedHashSet<Generic> generics = new LinkedHashSet<>(combinations.size());
      List<Type> actualTypes = new ArrayList<>(combinations.size());
      combinations.forEach(p -> {
        generics.add(p.getValue0());
        actualTypes.add(p.getValue1());
      });

      return new TypeBinding(generics, actualTypes);
    }
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
    if (binding.get(generic).isGeneric()) {
      throw new RuntimeException();
    }
    return Objects.requireNonNull(binding.get(generic), "Does not contain " + generic.getName());
  }

  public void addGenerics(Set<Generic> generics) {
    generics.forEach(g -> binding.putIfAbsent(g, null));
  }

  public void addGeneric(Generic generic) {
    binding.put(generic, null);
  }

  public void bindGeneric(Generic generic, Type type) {
    Preconditions.checkArgument(!type.isGeneric());
    binding.put(generic, type);
  }

  public List<Generic> unboundedGenerics() {
    return binding.entrySet().stream().filter(e -> e.getValue() == null).map(Entry::getKey)
        .toList();
  }

  public boolean hasUnboundedGeneric() {
    return binding.values().stream().anyMatch(Objects::isNull);
  }

  public TypeBinding leftOuterMerge(TypeBinding other) {
    var merged = new HashMap<>(binding);
    other.binding.forEach(merged::putIfAbsent);
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
