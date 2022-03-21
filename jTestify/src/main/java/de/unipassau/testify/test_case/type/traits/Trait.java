package de.unipassau.testify.test_case.type.traits;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import de.unipassau.testify.test_case.type.AssociatedType;
import de.unipassau.testify.test_case.type.Type;
import java.util.List;

@JsonDeserialize(as = AbstractTrait.class)
public interface Trait {
  String getName();

  List<Type> generics();

  List<AssociatedType> associatedTypes();
}
