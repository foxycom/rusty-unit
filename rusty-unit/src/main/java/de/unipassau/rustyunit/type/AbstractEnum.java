package de.unipassau.rustyunit.type;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import com.google.common.base.Preconditions;
import de.unipassau.rustyunit.json.EnumVariantDeserializer;
import de.unipassau.rustyunit.test_case.Param;
import de.unipassau.rustyunit.test_case.callable.Callable;
import de.unipassau.rustyunit.test_case.callable.Method;
import de.unipassau.rustyunit.type.traits.Trait;
import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.Set;
import lombok.Builder;

@Builder
@JsonDeserialize(as = AbstractEnum.class)
public class AbstractEnum implements Enum {

  private String name;
  private List<Type> generics;
  private List<EnumVariant> variants;
  @JsonProperty("is_local")
  private boolean isLocal;
  private Set<Trait> implementedTraits = Collections.emptySet();

  public AbstractEnum() {
  }

  public AbstractEnum(AbstractEnum other) {
    this.name = other.name;
    this.isLocal = other.isLocal;
    this.generics = other.generics.stream().map(Type::copy).toList();
    // Variants stay the same throughout the whole lifetime
    this.variants = other.variants;
    this.implementedTraits = other.implementedTraits;
  }

  public AbstractEnum(String name, List<Type> generics,
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
  public List<Type> generics() {
    return generics;
  }

  @Override
  public Set<Trait> implementedTraits() {
    return implementedTraits;
  }

  @Override
  public Callable unwrapMethod(int at) {
    return new Method("unwrap", Collections.emptyList(), List.of(new Param(this, false, null)),
        generics.get(0), this);
  }

  @Override
  public Optional<Integer> wraps(Type type) {

    if (!generics.isEmpty() && generics.get(0).canBeSameAs(type)) {
      return Optional.of(0);
    } else {
      return Optional.empty();
    }
  }

  @Override
  public void setGenerics(List<Type> generics) {
    this.generics = generics;
  }

  @Override
  public Type replaceGenerics(List<Type> generics) {
    var copy = new AbstractEnum(this);
    copy.generics = generics;
    return copy;
  }

  @Override
  public Type bindGenerics(TypeBinding binding) {
    var copy = (AbstractEnum) copy();
    if (binding.hasUnboundedGeneric()) {
      throw new RuntimeException("Unbound generics");
    }

    copy.generics = generics.stream().map(g -> g.bindGenerics(binding)).toList();
    copy.variants = variants.stream().map(v -> v.bindGenerics(binding)).toList();
    Preconditions.checkState(this.generics.size() == copy.generics.size());
    return copy;
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof AbstractEnum)) {
      return false;
    }
    AbstractEnum that = (AbstractEnum) o;
    return isLocal == that.isLocal && name.equals(that.name) && generics.equals(that.generics)
        && variants.equals(that.variants);
  }

  @Override
  public int hashCode() {
    return Objects.hash(name, generics, variants, isLocal);
  }

  @Override
  public String toString() {
    return name;
    //return encode();
  }

  public void setVariants(List<EnumVariant> variants) {
    this.variants = variants;
  }

  public List<Type> getGenerics() {
    return generics;
  }

  @Override
  public boolean isLocal() {
    return isLocal;
  }

  @Override
  public List<EnumVariant> variants() {
    return variants;
  }

  public void setLocal(boolean local) {
    isLocal = local;
  }

  @Override
  public Type copy() {
    return new AbstractEnum(this);
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

    public boolean isTuple() {
      return false;
    }

    public boolean isUnit() {
      return false;
    }

    public boolean isStruct() {
      return false;
    }

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
      throw new RuntimeException("Not implemented");
    }

    @Override
    public List<Param> getParams() {
      return Collections.emptyList();
    }

    @Override
    public boolean isUnit() {
      return true;
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

    private List<Param> params;

    @Override
    public boolean isStruct() {
      return true;
    }

    public StructEnumVariant(String name, List<Param> params) {
      super(name);
      this.params = params;
    }

    public StructEnumVariant(StructEnumVariant other) {
      this.name = other.name;
      this.params = other.params.stream().map(Param::copy).toList();
    }

    @Override
    public EnumVariant bindGenerics(TypeBinding binding) {
      var copy = new StructEnumVariant(this);
      copy.params = copy.params.stream().map(p -> p.bindGenerics(binding)).toList();
      return copy;
    }

    @Override
    public List<Param> getParams() {
      return params;
    }

    @Override
    public boolean hasParams() {
      return !params.isEmpty();
    }

    @Override
    public void setParams(List<Param> params) {
      this.params = params;
    }

    @Override
    public EnumVariant copy() {
      return new StructEnumVariant(this);
    }

  }

  public static class TupleEnumVariant extends EnumVariant {

    private List<Param> params;

    @Override
    public boolean isTuple() {
      return true;
    }

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
