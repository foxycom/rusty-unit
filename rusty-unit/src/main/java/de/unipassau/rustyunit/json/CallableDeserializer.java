package de.unipassau.rustyunit.json;

import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.DeserializationContext;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.deser.std.StdDeserializer;
import de.unipassau.rustyunit.test_case.callable.Callable;
import de.unipassau.rustyunit.test_case.callable.EnumInit;
import de.unipassau.rustyunit.test_case.callable.Function;
import de.unipassau.rustyunit.test_case.callable.Method;
import de.unipassau.rustyunit.test_case.callable.StaticMethod;
import de.unipassau.rustyunit.test_case.callable.StructInit;
import java.io.IOException;

public class CallableDeserializer extends StdDeserializer<Callable> {

  public CallableDeserializer() {
    this(null);
  }

  protected CallableDeserializer(Class<?> vc) {
    super(vc);
  }

  @Override
  public Callable deserialize(JsonParser p, DeserializationContext ctxt)
      throws IOException {
    JsonNode node = p.getCodec().readTree(p);
    var callableEntry = node.fields().next();
    var callableTypeName = callableEntry.getKey();
    var callableNode = callableEntry.getValue();

    return createCallable(callableTypeName, callableNode);
  }

  private static Callable createCallable(String callableType, JsonNode node)
      throws JsonProcessingException {
    var mapper = new ObjectMapper();

    return switch (callableType) {
      case "StaticFunction" -> mapper.readValue(node.toString(), StaticMethod.class);
      case "Method" -> mapper.readValue(node.toString(), Method.class);
      case "StructInit" -> mapper.readValue(node.toString(), StructInit.class);
      case "Function" -> mapper.readValue(node.toString(), Function.class);
      case "EnumInit" -> mapper.readValue(node.toString(), EnumInit.class);
      default -> throw new RuntimeException("Not implemented: " + callableType + ", node: " + node);
    };
  }
}
