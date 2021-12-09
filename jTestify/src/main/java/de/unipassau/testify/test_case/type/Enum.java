package de.unipassau.testify.test_case.type;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.test_case.Param;
import java.util.List;
import java.util.Locale;
import java.util.Objects;
import java.util.stream.Collectors;

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
  public boolean isSameType(Type other) {
    if (other.isRef()) {
      var ref = other.asRef();
      return isSameType(ref.getInnerType());
    } else if (other.isEnum()) {
      return isSameEnum(other.asEnum());
    } else {
      return false;
    }
  }

  public boolean isSameEnum(Enum other) {
    // TODO prolly also compare variants
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
      var genericNames = generics.stream().map(Object::toString).collect(Collectors.joining(", "));
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
