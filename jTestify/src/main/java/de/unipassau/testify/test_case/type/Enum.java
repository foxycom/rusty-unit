package de.unipassau.testify.test_case.type;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.json.EnumVariantDeserializer;
import de.unipassau.testify.test_case.Param;
import java.util.Collections;
import java.util.List;
import java.util.Locale;
import java.util.Objects;
import java.util.Set;
import java.util.stream.Collectors;
import java.util.stream.IntStream;
import lombok.AllArgsConstructor;
import lombok.Builder;

@Builder
@JsonDeserialize(as = Enum.class)
public class Enum implements Type {

  private String name;
  private List<Type> generics;
  private List<EnumVariant> variants;
  @JsonProperty("is_local")
  private boolean isLocal;
  private Set<Trait> implementedTraits = Collections.emptySet();

  public Enum() {
  }

  public Enum(Enum other) {
    this.name = other.name;
    this.isLocal = other.isLocal;
    this.generics = other.generics.stream().map(Type::copy).toList();
    // Variants stay the same throughout the whole lifetime
    this.variants = other.variants;
  }

  public Enum(String name, List<Type> generics,
      List<EnumVariant> variants, boolean isLocal, Set<Trait> implementedTraits) {
    this.name = name;
    this.generics = generics;
    this.variants = variants;
    this.isLocal = isLocal;
    this.implementedTraits = implementedTraits;
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
  public boolean isEnum() {
    return true;
  }

  @Override
  public Enum asEnum() {
    return this;
  }

  @Override
  public String varString() {
    var segments = name.split("::");
    return segments[segments.length - 1].toLowerCase(Locale.ROOT);
  }

  @Override
  public boolean canBeSameAs(Type other) {
    if (other.isEnum()) {
      return isSameEnum(other.asEnum()) &&
          generics.size() == other.generics().size() &&
          IntStream.range(0, generics.size()).allMatch(i ->
              generics.get(i).canBeSameAs(other.generics().get(i)));
    } else {
      return other.isGeneric();
    }
  }

  @Override
  public boolean canBeIndirectlySameAs(Type other) {
    return variants.stream().anyMatch(v -> v.getParams().stream()
        .anyMatch(p -> p.getType().equals(other) || p.getType().canBeIndirectlySameAs(other)));
  }

  public boolean isSameEnum(Enum other) {
    return isLocal == other.isLocal && name.equals(other.name);
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
  public boolean wraps(Type type) {
    return generics.stream().anyMatch(g -> g.canBeSameAs(type) || g.wraps(type));
  }

  @Override
  public void setGenerics(List<Type> generics) {
    this.generics = generics;
  }

  @Override
  public Type replaceGenerics(List<Type> generics) {
    var copy = new Enum(this);
    copy.generics = generics;
    return copy;
  }

  @Override
  public Type bindGenerics(TypeBinding binding) {
    var copy = new Enum(this);
    if (binding.hasUnboundedGeneric()) {
      throw new RuntimeException("Unbound generics");
    }

    copy.generics = generics.stream().map(g -> g.bindGenerics(binding)).toList();
    copy.variants = variants.stream().map(v -> v.bindGenerics(binding)).toList();
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
    Enum anEnum = (Enum) o;
    return isLocal == anEnum.isLocal && name.equals(anEnum.name) && generics.equals(anEnum.generics)
        && variants.equals(anEnum.variants);
  }

  @Override
  public int hashCode() {
    return Objects.hash(name, generics, variants, isLocal);
  }

  @Override
  public String toString() {
    var sb = new StringBuilder(name);
    if (!generics.isEmpty()) {
      sb.append("<");
      var genericNames = generics.stream().map(Type::toGenericString)
          .collect(Collectors.joining(", "));
      sb.append(genericNames);
      sb.append(">");
    }
    return sb.toString();
  }

  public List<EnumVariant> getVariants() {
    return variants;
  }

  public void setVariants(List<EnumVariant> variants) {
    this.variants = variants;
  }

  public List<Type> getGenerics() {
    return generics;
  }

  public boolean isLocal() {
    return isLocal;
  }

  public void setLocal(boolean local) {
    isLocal = local;
  }

  @Override
  public Type copy() {
    return new Enum(this);
  }

  @JsonDeserialize(using = EnumVariantDeserializer.class)
  public static abstract class EnumVariant {

    protected String name;

    public EnumVariant() {
    }

    public EnumVariant(String name) {
      this.name = name;
    }

    public String getName() {
      return name;
    }

    public void setName(String name) {
      this.name = name;
    }

    public abstract EnumVariant bindGenerics(TypeBinding binding);

    public abstract List<Param> getParams();

    public abstract boolean hasParams();

    public abstract void setParams(List<Param> params);

    public abstract EnumVariant copy();
  }

  public static class UnitEnumVariant extends EnumVariant {

    public UnitEnumVariant(String name) {
      super(name);
    }

    @Override
    public EnumVariant bindGenerics(TypeBinding binding) {
      return this;
    }

    @Override
    public List<Param> getParams() {
      return Collections.emptyList();
    }

    @Override
    public boolean hasParams() {
      return false;
    }

    @Override
    public void setParams(List<Param> params) {
      throw new RuntimeException("setParams is not implemented");
    }

    @Override
    public EnumVariant copy() {
      return new UnitEnumVariant(name);
    }
  }

  public static class StructEnumVariant extends EnumVariant {

    public StructEnumVariant() {
      throw new RuntimeException("Not implemented yet");
    }

    @Override
    public EnumVariant bindGenerics(TypeBinding binding) {
      throw new RuntimeException("bindGenerics is not implemented");
    }

    @Override
    public List<Param> getParams() {
      throw new RuntimeException("getParams is not implemented");
    }

    @Override
    public boolean hasParams() {
      throw new RuntimeException("hasParams is not implemented");
    }

    @Override
    public void setParams(List<Param> params) {
      throw new RuntimeException("setParams is not implemented");
    }

    @Override
    public EnumVariant copy() {
      throw new RuntimeException("copy is not implemented");
    }

  }

  public static class TupleEnumVariant extends EnumVariant {

    private List<Param> params;

    public TupleEnumVariant(String name, List<Param> params) {
      super(name);
      this.params = params;
    }

    public TupleEnumVariant(TupleEnumVariant other) {
      this.name = other.name;
      this.params = other.params.stream().map(Param::copy).toList();
    }

    @Override
    public boolean hasParams() {
      return !params.isEmpty();
    }

    @Override
    public List<Param> getParams() {
      return params;
    }

    @Override
    public EnumVariant bindGenerics(TypeBinding binding) {
      var copy = new TupleEnumVariant(this);
      copy.params = copy.params.stream().map(p -> p.bindGenerics(binding)).toList();
      return copy;
    }

    @Override
    public void setParams(List<Param> params) {
      this.params = params;
    }

    @Override
    public EnumVariant copy() {
      return new TupleEnumVariant(this);
    }

    @Override
    public boolean equals(Object o) {
      if (this == o) {
        return true;
      }
      if (o == null || getClass() != o.getClass()) {
        return false;
      }
      TupleEnumVariant that = (TupleEnumVariant) o;
      return name.equals(that.name) && params.equals(that.params);
    }

    @Override
    public int hashCode() {
      return Objects.hash(name, params);
    }
  }
}
