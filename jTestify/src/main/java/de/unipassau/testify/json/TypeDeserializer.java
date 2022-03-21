package de.unipassau.testify.json;

import com.fasterxml.jackson.core.JacksonException;
import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.DeserializationContext;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.deser.std.StdDeserializer;
import de.unipassau.testify.test_case.type.Array;
import de.unipassau.testify.test_case.type.AbstractStruct;
import de.unipassau.testify.test_case.type.AbstractEnum;
import de.unipassau.testify.test_case.type.Generic;
import de.unipassau.testify.test_case.type.Ref;
import de.unipassau.testify.test_case.type.Tuple;
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
      case "Struct" -> mapper.readValue(node.toString(), AbstractStruct.class);
      case "Generic" -> mapper.readValue(node.toString(), Generic.class);
      case "Ref" -> {
        var mutable = node.get(1).asBoolean();
        var entry = node.get(0).fields().next();
        var innerTypeName = entry.getKey();
        var innerNode = entry.getValue();
        var innerType = createType(innerTypeName, innerNode);
        yield new Ref(innerType, mutable);
      }
      case "Prim" -> mapper.readValue(node.toString(), Prim.class);
      case "Enum" -> mapper.readValue(node.toString(), AbstractEnum.class);
      case "Tuple" -> mapper.readValue(node.toString(), Tuple.class);
      case "Array" -> mapper.readValue(node.toString(), Array.class);
      default -> throw new RuntimeException("Not implemented: "+ typeName);
    };
  }
}
