package de.unipassau.testify.json;

import com.fasterxml.jackson.core.JacksonException;
import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.DeserializationContext;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.deser.std.StdDeserializer;
import de.unipassau.testify.test_case.type.Complex;
import de.unipassau.testify.test_case.type.Enum;
import de.unipassau.testify.test_case.type.Generic;
import de.unipassau.testify.test_case.type.Ref;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.prim.Prim;
import java.io.IOException;

public class TypeDeserializer extends StdDeserializer<Type> {

  public TypeDeserializer() {
    this(null);
  }

  protected TypeDeserializer(Class<?> vc) {
    super(vc);
  }

  @Override
  public Type deserialize(JsonParser p, DeserializationContext ctxt)
      throws IOException, JacksonException {
    JsonNode node = p.getCodec().readTree(p);
    var typeEntry = node.fields().next();
    var typeName = typeEntry.getKey();
    var typeNode = typeEntry.getValue();

    return createType(typeName, typeNode);
  }

  private Type createType(String typeName, JsonNode node) throws JsonProcessingException {
    var mapper = new ObjectMapper();
    return switch (typeName) {
      case "Complex" -> mapper.readValue(node.toString(), Complex.class);
      case "Generic" -> mapper.readValue(node.toString(), Generic.class);
      case "Ref" -> {
        var entry = node.fields().next();
        var innerTypeName = entry.getKey();
        var innerNode = entry.getValue();
        var innerType = createType(innerTypeName, innerNode);
        yield new Ref(innerType);
      }
      case "Prim" -> mapper.readValue(node.toString(), Prim.class);
      case "Enum" -> mapper.readValue(node.toString(), Enum.class);
      default -> throw new RuntimeException("Not implemented: "+ typeName);
    };
  }
}
