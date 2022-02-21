package de.unipassau.testify.test_case.type;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import java.util.List;
import java.util.Objects;
import java.util.Set;

@JsonDeserialize(as = Ref.class)
public class Ref implements Type {

  private boolean mutable;
  private Type innerType;

  public Ref() {
  }

  public Ref(Ref other) {
    this.innerType = other.innerType.copy();
    this.mutable = other.mutable;
  }

  public Ref(Type innerType, boolean mutable) {
    this.mutable = mutable;
    this.innerType = innerType;
  }

  public boolean isMutable() {
    return mutable;
  }

  public void setMutable(boolean mutable) {
    this.mutable = mutable;
  }

  @Override
  public String getName() {
    return innerType.getName();
  }

  @Override
  public void setName(String name) {
    innerType.setName(name);
  }

  @Override
  public String fullName() {
    return innerType.fullName();
  }

  @Override
  public String varString() {
    return innerType.varString();
  }

  @Override
  public boolean canBeSameAs(Type other) {
    if (other.isRef()) {
      return innerType.canBeSameAs(other.asRef().getInnerType());
    } else {
      return other.isGeneric();
    }
  }

  @Override
  public boolean canBeIndirectlySameAs(Type other) {
    if (other.isRef()) {
      var otherRef = other.asRef();
      if (otherRef.isMutable() || !this.isMutable()) {
        return innerType.canBeSameAs(other.asRef().getInnerType());
      }
    } else {
      return other.isGeneric();
    }

    return false;
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    Ref ref = (Ref) o;
    return innerType.equals(ref.innerType) && ref.mutable == this.mutable;
  }

  @Override
  public int hashCode() {
    return Objects.hash(innerType, mutable);
  }

  @Override
  public List<Type> generics() {
    return innerType.generics();
  }

  @Override
  public Set<Trait> implementedTraits() {
    throw new RuntimeException("implementedTraits is not implemented");
  }

  @Override
  public void setGenerics(List<Type> generics) {
    innerType.setGenerics(Objects.requireNonNull(generics));
  }

  @Override
  public Type replaceGenerics(List<Type> generics) {
    return new Ref(innerType.replaceGenerics(Objects.requireNonNull(generics)), mutable);
  }

  @Override
  public Type bindGenerics(TypeBinding binding) {
    return new Ref(innerType.bindGenerics(binding), mutable);
  }

  @Override
  public boolean isRef() {
    return true;
  }

  @Override
  public Ref asRef() {
    return this;
  }

  public Type getInnerType() {
    return innerType;
  }

  @Override
  public String toString() {
    if (mutable) {
      return String.format("&mut %s", innerType);
    } else {
      return String.format("&%s", innerType);
    }
  }

  @Override
  public String toGenericString() {
    return String.format("&%s", innerType);
  }

  @Override
  public Type copy() {
    return new Ref(this);
  }
}
