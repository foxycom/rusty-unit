package de.unipassau.testify.json;

import com.fasterxml.jackson.core.JacksonException;
import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.databind.DeserializationContext;
import com.fasterxml.jackson.databind.deser.std.StdDeserializer;
import de.unipassau.testify.test_case.type.prim.UInt;
import java.io.IOException;

public class UIntDeserializer extends StdDeserializer<UInt> {

  public UIntDeserializer() {
    this(null);
  }

  protected UIntDeserializer(Class<?> vc) {
    super(vc);
  }

  @Override
  public UInt deserialize(JsonParser p, DeserializationContext ctxt)
      throws IOException, JacksonException {
    return null;
  }
}
