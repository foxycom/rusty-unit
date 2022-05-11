package de.unipassau.rustyunit.json;

import com.fasterxml.jackson.core.JacksonException;
import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.DeserializationContext;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.deser.std.StdDeserializer;
import de.unipassau.rustyunit.test_case.type.prim.Bool;
import de.unipassau.rustyunit.test_case.type.prim.Char;
import de.unipassau.rustyunit.test_case.type.prim.Float.Float32;
import de.unipassau.rustyunit.test_case.type.prim.Float.Float64;
import de.unipassau.rustyunit.test_case.type.prim.Int.ISize;
import de.unipassau.rustyunit.test_case.type.prim.Int.Int128;
import de.unipassau.rustyunit.test_case.type.prim.Int.Int16;
import de.unipassau.rustyunit.test_case.type.prim.Int.Int32;
import de.unipassau.rustyunit.test_case.type.prim.Int.Int64;
import de.unipassau.rustyunit.test_case.type.prim.Int.Int8;
import de.unipassau.rustyunit.test_case.type.prim.Prim;
import de.unipassau.rustyunit.test_case.type.prim.Str;
import de.unipassau.rustyunit.test_case.type.prim.UInt.UInt128;
import de.unipassau.rustyunit.test_case.type.prim.UInt.UInt16;
import de.unipassau.rustyunit.test_case.type.prim.UInt.UInt32;
import de.unipassau.rustyunit.test_case.type.prim.UInt.UInt64;
import de.unipassau.rustyunit.test_case.type.prim.UInt.UInt8;
import de.unipassau.rustyunit.test_case.type.prim.UInt.USize;
import java.io.IOException;

public class PrimDeserializer extends StdDeserializer<Prim> {

  public PrimDeserializer() {
    this(null);
  }

  protected PrimDeserializer(Class<?> vc) {
    super(vc);
  }

  @Override
  public Prim deserialize(JsonParser p, DeserializationContext ctxt)
      throws IOException, JacksonException {
    JsonNode node = p.getCodec().readTree(p);

    if (node.isTextual()) {
      return createPrim(node.asText());
    } else {
      var entry = node.fields().next();
      return createPrim(entry.getKey(), entry.getValue().asText());
    }
  }

  private Prim createPrim(String typeName) {
    return switch (typeName) {
      case "Bool" -> Bool.INSTANCE;
      case "Char" -> Char.INSTANCE;
      case "Str" -> Str.INSTANCE;
      default -> throw new RuntimeException("Not implemented");
    };
  }

  private Prim createPrim(String typeName, String subType) throws JsonProcessingException {
    return switch (typeName) {
      case "Int" -> createInt(subType);
      case "Uint" -> createUInt(subType);
      case "Float" -> createFloat(subType);
      default -> throw new RuntimeException("Not implemented");
    };
  }

  private Prim createInt(String typeName) {
    return switch (typeName) {
      case "I8" -> Int8.INSTANCE;
      case "I16" -> Int16.INSTANCE;
      case "I32" -> Int32.INSTANCE;
      case "I64" -> Int64.INSTANCE;
      case "I128" -> Int128.INSTANCE;
      case "Isize" -> ISize.INSTANCE;
      default -> throw new RuntimeException("Not impplemented");
    };
  }

  private Prim createUInt(String typeName) {
    return switch (typeName) {
      case "U8" -> UInt8.INSTANCE;
      case "U16" -> UInt16.INSTANCE;
      case "U32" -> UInt32.INSTANCE;
      case "U64" -> UInt64.INSTANCE;
      case "U128" -> UInt128.INSTANCE;
      case "Usize" -> USize.INSTANCE;
      default -> throw new RuntimeException("Not implemented");
    };
  }

  private Prim createFloat(String typeName) {
    return switch (typeName) {
      case "F32" -> Float32.INSTANCE;
      case "F64" -> Float64.INSTANCE;
      default -> throw new RuntimeException("Not implemented");
    };
  }
}
