package de.unipassau.rustyunit.json;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.ObjectMapper;
import de.unipassau.rustyunit.test_case.callable.Callable;
import de.unipassau.rustyunit.test_case.callable.StaticMethod;
import java.util.ArrayList;
import java.util.List;
import org.json.JSONObject;

public class JSONParser {
  public static List<Callable> parse(String json, boolean parseTraits) throws JsonProcessingException {
    var callablesArray = new JSONObject(json).getJSONArray("callables");

    var mapper = new ObjectMapper();
    var callables = new ArrayList<Callable>();
    for (int i = 0; i < callablesArray.length(); i++) {
      var obj = callablesArray.getJSONObject(i);
      var callable = mapper.readValue(obj.toString(), Callable.class);
      if (!parseTraits && callable.isStaticMethod()) {
        if (callable.asStaticMethod().ofTrait() != null) {
          continue;
        }
      } else if (!parseTraits && callable.isMethod()) {
        if (callable.asMethod().ofTrait() != null) {
          continue;
        }
      }
      callables.add(callable);
    }

    return callables;
  }

  private static Callable createCallable(String callableType, JSONObject obj)
      throws JsonProcessingException {
    var mapper = new ObjectMapper();
    return switch (callableType) {
      case "StaticFunction" -> mapper.readValue(obj.toString(), StaticMethod.class);
      default -> throw new RuntimeException("Not implemented");
    };
  }
}
