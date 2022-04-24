package de.unipassau.testify.test_case.type;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.test_case.type.traits.Trait;
import java.util.Collections;
import java.util.HashSet;
import java.util.List;
import java.util.Objects;
import java.util.Set;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.NoArgsConstructor;

@Builder
@AllArgsConstructor
@NoArgsConstructor
@EqualsAndHashCode(exclude = "implementedTraits")
@JsonDeserialize(as = AbstractStruct.class)
public class AbstractStruct implements Struct {

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
  public List<Type> generics() {
    return generics;
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
    return encode();
  }

  @Override
  public boolean isLocal() {
    return isLocal;
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof AbstractStruct)) {
      return false;
    }
    AbstractStruct that = (AbstractStruct) o;
    return isLocal == that.isLocal && name.equals(that.name) && generics.equals(that.generics)
        && implementedTraits.equals(that.implementedTraits);
  }

  @Override
  public int hashCode() {
    return Objects.hash(name, generics, isLocal, implementedTraits);
  }
}
