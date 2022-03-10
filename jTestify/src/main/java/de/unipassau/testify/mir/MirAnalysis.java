package de.unipassau.testify.mir;

import static de.unipassau.testify.Constants.MIR_LOG_PATH;

import java.io.BufferedReader;
import java.io.FileReader;
import java.io.IOException;
import java.nio.file.FileVisitOption;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.HashMap;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.stream.Collectors;
import org.json.JSONObject;

public class MirAnalysis {
  private static final Map<String, CDG> CDGs = parseCDGs();

  private static Map<String, CDG> parseCDGs() {
    Map<String, CDG> cdgs = new HashMap<>();
    var path = Paths.get(MIR_LOG_PATH);
    try (var stream = Files.walk(path, Integer.MAX_VALUE)) {
      stream
          .filter(Files::isRegularFile)
          .filter(file -> file.getFileName().toString().startsWith("mir"))
          .forEach(file -> {
            try {
              var content = Files.readString(file);
              var jsonRoot = new JSONObject(content);
              var globalId = jsonRoot.getString("global_id");
              var cdg = CDG.parse(globalId, jsonRoot.getString("cdg"));
              cdgs.put(globalId, cdg);
            } catch (IOException e) {
              throw new RuntimeException(e);
            }
          });
    } catch (IOException e) {
      throw new RuntimeException("Could not parse CDGs from mir logs", e);
    }

    return cdgs;
  }

  public static Set<BasicBlock> targets() {
    return CDGs.values().stream()
        .map(CDG::targets)
        .flatMap(Set::stream)
        .collect(Collectors.toSet());
  }

  public static Set<BasicBlock> targets(String globalId) {
    return CDGs.entrySet().stream().filter(e -> Objects.equals(e.getKey(), globalId))
        .map(e -> e.getValue().targets())
        .findFirst().get();
  }

  public static void main(String[] args) {
    System.out.println();
  }
}
