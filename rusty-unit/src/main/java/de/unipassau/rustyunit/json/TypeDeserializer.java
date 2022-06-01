package de.unipassau.rustyunit.json;

import com.fasterxml.jackson.core.JacksonException;
import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.DeserializationContext;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.deser.std.StdDeserializer;
import de.unipassau.rustyunit.type.Array;
import de.unipassau.rustyunit.type.AbstractStruct;
import de.unipassau.rustyunit.type.AbstractEnum;
import de.unipassau.rustyunit.type.Fn;
import de.unipassau.rustyunit.type.Generic;
import de.unipassau.rustyunit.type.Ref;
import de.unipassau.rustyunit.type.Slice;
import de.unipassau.rustyunit.type.TraitObj;
import de.unipassau.rustyunit.type.Tuple;
import de.unipassau.rustyunit.type.Type;
import de.unipassau.rustyunit.type.prim.Prim;
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
    String typeName;
    JsonNode typeNode;
    if (node.fields().hasNext()) {
      var typeEntry = node.fields().next();
      typeName = typeEntry.getKey();
      typeNode = typeEntry.getValue();
    } else {
      if (node.textValue().equals("Fn")) {
        typeName = "Fn";
        typeNode = null;
      } else {
        throw new RuntimeException("Not implemented");
      }
    }


    return createType(typeName, typeNode);
  }

  private Type createType(String typeName, JsonNode node) throws JsonProcessingException {
    var mapper = new ObjectMapper();
    return switch (typeName) {
      case "Struct" -> parseStruct(node);
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
      case "Enum" -> parseEnum(node);
      case "Tuple" -> mapper.readValue(node.toString(), Tuple.class);
      case "Array" -> mapper.readValue(node.toString(), Array.class);
      case "TraitObj" -> mapper.readValue(node.toString(), TraitObj.class);
      case "Slice" -> parseSlice(node);
      case "Fn" -> new Fn();
      default -> throw new RuntimeException("Not implemented: "+ typeName);
    };
  }

  private Type parseSlice(JsonNode node) throws JsonProcessingException {
    var mapper = new ObjectMapper();
    return new Slice(mapper.readValue(node.toString(), Type.class));
  }

  private Type parseEnum(JsonNode node) throws JsonProcessingException {
    var mapper = new ObjectMapper();
    try {
      var className = className(node.get("name").textValue());
      Class<Type> enumClass = (Class<Type>) Class.forName(className);
      return mapper.readValue(node.toString(), enumClass);
    } catch (ClassNotFoundException e) {
      return mapper.readValue(node.toString(), AbstractEnum.class);
    }
  }

  private Type parseStruct(JsonNode node) throws JsonProcessingException {
    var mapper = new ObjectMapper();
    var className = className(node.get("name").textValue());
    try {
      Class<Type> structClass = (Class<Type>) Class.forName(className);
      var struct = mapper.readValue(node.toString(), structClass);
      return struct;
    } catch (ClassNotFoundException e) {
      return mapper.readValue(node.toString(), AbstractStruct.class);
    }
  }

  private String className(String name) {
    return String.format("de.unipassau.rustyunit.type.%s", name.replaceAll("::", "."));
  }
}
