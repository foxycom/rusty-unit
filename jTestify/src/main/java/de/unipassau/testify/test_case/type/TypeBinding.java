package de.unipassau.testify.test_case.type;

import com.google.common.collect.Streams;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Map.Entry;
import java.util.Objects;
import java.util.Set;
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

  public TypeBinding(Set<Generic> generics) {
    this(generics, generics.stream().map(g -> (Type) null).toList());
  }

  public TypeBinding() {
    binding = new HashMap<>();
  }

  public Type getBindingFor(Generic generic) {
    return binding.get(generic);
  }

  public void bindGeneric(Generic generic, Type type) {
    binding.put(generic, type);
  }

  public List<Generic> unboundedGenerics() {
    return binding.entrySet().stream().filter(e -> e.getValue() == null).map(Entry::getKey).toList();
  }

  public boolean hasUnboundedGeneric() {
    return binding.values().stream().anyMatch(Objects::isNull);
  }
}
