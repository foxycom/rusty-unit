package de.unipassau.testify.test_case.type;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.test_case.Param;
import java.util.List;
import java.util.Locale;
import java.util.Objects;
import java.util.stream.Collectors;
import java.util.stream.IntStream;

@JsonDeserialize(as = Enum.class)
public class Enum implements Type {

  private String name;
  private List<Type> generics;
  private List<EnumVariant> variants;
  @JsonProperty("is_local")
  private boolean isLocal;

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
      List<EnumVariant> variants, boolean isLocal) {
    this.name = name;
    this.generics = generics;
    this.variants = variants;
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

  public boolean isSameEnum(Enum other) {
    return isLocal == other.isLocal && name.equals(other.name);
  }

  @Override
  public List<Type> generics() {
    return generics;
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
      var genericNames = generics.stream().map(Type::toGenericString).collect(Collectors.joining(", "));
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

  @JsonDeserialize(as = EnumVariant.class)
  public static class EnumVariant {

    private String name;
    private List<Param> params;

    public EnumVariant() {
    }

    public EnumVariant(String name, List<Param> params) {
      this.name = name;
      this.params = params;
    }

    public EnumVariant(EnumVariant other) {
      this.name = other.name;
      this.params = other.params.stream().map(Param::copy).toList();
    }

    public String getName() {
      return name;
    }

    public void setName(String name) {
      this.name = name;
    }

    public boolean hasParams() {
      return !params.isEmpty();
    }

    public List<Param> getParams() {
      return params;
    }

    public EnumVariant bindGenerics(TypeBinding binding) {
      var copy = new EnumVariant(this);
      copy.params = copy.params.stream().map(p -> p.bindGenerics(binding)).toList();
      return copy;
    }

    public void setParams(List<Param> params) {
      this.params = params;
    }

    @Override
    public boolean equals(Object o) {
      if (this == o) {
        return true;
      }
      if (o == null || getClass() != o.getClass()) {
        return false;
      }
      EnumVariant that = (EnumVariant) o;
      return name.equals(that.name) && params.equals(that.params);
    }

    @Override
    public int hashCode() {
      return Objects.hash(name, params);
    }

  }
}
