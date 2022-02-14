package de.unipassau.testify.mir;

import static de.unipassau.testify.Constants.MIR_LOG_PATH;

import java.io.BufferedReader;
import java.io.FileReader;
import java.io.IOException;
import java.util.HashMap;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.stream.Collectors;

public class MirAnalysis {
  private static final Map<String, CDG> CDGs = parseCDGs();

  private static Map<String, CDG> parseCDGs() {
    Map<String, CDG> cdgs = new HashMap<>();
    try (var in = new BufferedReader(new FileReader(MIR_LOG_PATH))) {
      String globalId = null;
      var readingCdg = false;
      for (String line; (line = in.readLine()) != null; ) {
        if (line.startsWith(">>")) {
          globalId = line.substring(2);
        } else if (line.startsWith("#cdg")) {
          readingCdg = true;
        } else if (readingCdg && !line.startsWith("<data>")) {
          readingCdg = false;
        } else if (readingCdg && line.startsWith("<data>")) {
          var cdgStr = line.substring(6);
          var cdg = CDG.parse(globalId, cdgStr);
          cdgs.put(globalId, cdg);
        }
      }
    } catch (IOException e) {
      throw new RuntimeException("Could not parse CDGs from mir.log", e);
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
