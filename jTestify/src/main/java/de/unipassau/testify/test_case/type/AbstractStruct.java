package de.unipassau.testify.test_case.type;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import java.util.Collections;
import java.util.HashSet;
import java.util.List;
import java.util.Locale;
import java.util.Objects;
import java.util.Set;
import java.util.stream.Collectors;
import java.util.stream.IntStream;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.NoArgsConstructor;

@Builder
@AllArgsConstructor
@NoArgsConstructor
@EqualsAndHashCode(exclude = "implementedTraits")
@JsonDeserialize(as = AbstractStruct.class)
public class AbstractStruct implements Type {

  protected String name;
  protected List<Type> generics = Collections.emptyList();
  @JsonProperty("is_local")
  protected boolean isLocal;
  protected Set<Trait> implementedTraits = Collections.emptySet();

  public AbstractStruct(AbstractStruct other) {
    this.name = other.name;
    this.isLocal = other.isLocal;
    this.generics = other.generics.stream().map(Type::copy).peek(Objects::requireNonNull).toList();
    this.implementedTraits = new HashSet<>(other.implementedTraits);
  }

  public AbstractStruct(String name, List<Type> generics, boolean isLocal) {
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
  public boolean isStruct() {
    return true;
  }

  @Override
  public AbstractStruct asStruct() {
    return this;
  }

  @Override
  public List<Type> generics() {
    return generics;
  }

  @Override
  public String varString() {
    var segments = name.split("::");
    return segments[segments.length - 1].toLowerCase(Locale.ROOT);
  }

  @Override
  public boolean canBeSameAs(Type other) {
    if (other.isStruct()) {
      return isSameType(other.asStruct()) &&
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

  public boolean isSameType(AbstractStruct other) {
    return name.equals(other.name) && isLocal == other.isLocal;
  }

  @Override
  public Set<Trait> implementedTraits() {
    return implementedTraits;
  }

  @Override
  public void setGenerics(List<Type> generics) {
    this.generics = generics;
  }

  @Override
  public Type replaceGenerics(List<Type> generics) {
    var copy = new AbstractStruct(this);
    generics.forEach(Objects::requireNonNull);
    copy.generics = generics;
    return copy;
  }

  @Override
  public Type bindGenerics(TypeBinding binding) {
    var copy = new AbstractStruct(this);
    if (binding.hasUnboundedGeneric()) {
      throw new RuntimeException("Unbounded generics");
    }

    copy.generics = generics.stream().map(g -> g.bindGenerics(binding)).toList();
    return copy;
  }


  @Override
  public Type copy() {
    return new AbstractStruct(this);
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
