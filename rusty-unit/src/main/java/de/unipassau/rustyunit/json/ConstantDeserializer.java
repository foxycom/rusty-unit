package de.unipassau.rustyunit.json;

import com.fasterxml.jackson.core.JacksonException;
import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.databind.DeserializationContext;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.deser.std.StdDeserializer;
import de.unipassau.rustyunit.test_case.primitive.PrimitiveValue;
import de.unipassau.rustyunit.test_case.type.Type;
import java.io.IOException;

public class ConstantDeserializer extends StdDeserializer<PrimitiveValue<?>> {

  public ConstantDeserializer() {
    this(null);
  }

  protected ConstantDeserializer(Class<?> vc) {
    super(vc);
  }

  @Override
  public PrimitiveValue<?> deserialize(JsonParser p, DeserializationContext ctxt)
      throws IOException, JacksonException {
    JsonNode node = p.getCodec().readTree(p);
    var val = node.get("val");
    var ty = node.get("ty");
    var objectMapper = new ObjectMapper();
    Type type = objectMapper.readValue(ty.toString(), Type.class);

    return toPrimitiveValue(type, val.textValue());
  }

  private PrimitiveValue<?> toPrimitiveValue(Type type, String value) {
    var prim = type.asPrimitive();
    return prim.from(value);
  }
}
