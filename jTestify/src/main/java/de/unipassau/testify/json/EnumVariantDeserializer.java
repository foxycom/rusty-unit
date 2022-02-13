package de.unipassau.testify.json;

import com.fasterxml.jackson.core.JacksonException;
import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.DeserializationContext;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.deser.std.StdDeserializer;
import de.unipassau.testify.test_case.Param;
import de.unipassau.testify.test_case.type.Enum;
import de.unipassau.testify.test_case.type.Enum.EnumVariant;
import de.unipassau.testify.test_case.type.Enum.TupleEnumVariant;
import de.unipassau.testify.test_case.type.Enum.UnitEnumVariant;
import java.io.IOException;
import java.util.List;

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

    var entry = node.fields().next();
    var variantSpec = entry.getKey();
    var variantNode = entry.getValue();
    return createEnumVariant(variantSpec, variantNode);
  }

  private static EnumVariant createEnumVariant(String variantSpec, JsonNode node)
      throws JsonProcessingException {
    var mapper = new ObjectMapper();
    return switch (variantSpec) {
      case "Unit" -> new UnitEnumVariant(node.asText());
      case "Tuple" -> {
        var variantName = node.get(0).asText();

        var collectionType = mapper.getTypeFactory().constructCollectionType(List.class, Param.class);
        var javaType = mapper.constructType(collectionType);
        var params = mapper.<List<Param>>readValue(node.get(1).toString(), javaType);
        yield new TupleEnumVariant(variantName, params);
      }
      default -> throw new RuntimeException("Not implemented: " + variantSpec);
    };
  }
}
