package de.unipassau.testify.test_case.type;

public enum TypeEquality {
  // Types are same, e.g., Vec<u32> == Vec<u32>
  DIRECT,
  // Type is a container and contains another type, e.g., Vec<u32> == u32
  INDIRECT,
  // Type is a generic and can be filled with a concrete ones wrt the trait
  // bounds, e.g., T == Vec<u32>
  GENERIC,
  // Types cannot be same
  NONE;

  public boolean isDirect() {
    return this == DIRECT;
  }

  public boolean isIndirect() {
    return this == INDIRECT;
  }

  public boolean isGeneric() {
    return this == GENERIC;
  }

  public boolean isNone() {
    return this == NONE;
  }

  public boolean canBeSame() {
    return this == DIRECT || this == INDIRECT || this == GENERIC;
  }
}
