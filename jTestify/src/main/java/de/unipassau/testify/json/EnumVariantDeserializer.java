package de.unipassau.testify.json;

import com.fasterxml.jackson.core.JacksonException;
import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.databind.DeserializationContext;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.deser.std.StdDeserializer;
import de.unipassau.testify.test_case.type.Enum.EnumVariant;
import java.io.IOException;

public class EnumVariantDeserializer extends StdDeserializer<EnumVariant> {

  public EnumVariantDeserializer() {
    this(null);
  }

  protected EnumVariantDeserializer(Class<?> vc) {
    super(vc);
  }

  @Override
  public EnumVariant deserialize(JsonParser p, DeserializationContext ctxt)
      throws IOException, JacksonException {
    JsonNode node = p.getCodec().readTree(p);
    var mapper = new ObjectMapper();
    return mapper.readValue(node.toString(), EnumVariant.class);
  }
}
