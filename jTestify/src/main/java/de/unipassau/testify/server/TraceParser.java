package de.unipassau.testify.server;

import de.unipassau.testify.mir.Branch;
import de.unipassau.testify.mir.MirAnalysis;
import org.javatuples.Triplet;

public class TraceParser {

  /**
   * A branch line example: <test id> branch[<global id> <local id> <block id> <distance>]
   * A root line example: <test id> root[<global id> <local id>]
   */
  public static Triplet<Integer, Branch, Double> parse(String line) {
    var testId = Integer.parseInt(line.substring(0, line.indexOf(" ")));
    line = line.substring(line.indexOf(" ") + 1);
    if (line.startsWith("branch")) {
      var dataBegin = line.indexOf("[") + 1;
      var dataEnd = line.length() - 1;
      var data = line.substring(dataBegin, dataEnd).split(" ");

      var globalId = Integer.parseInt(data[0]);
      var localId = Integer.parseInt(data[1]);
      var blockId = Integer.parseInt(data[2]);
      var distance = Double.parseDouble(data[3]);

      throw new RuntimeException("Not implemented yet");
      /*return Triplet.with(
          testId,
          //MirAnalysis.getDecisionBranch(globalId, localId, blockId),
          null,
          distance
      );*/
    } else if (line.startsWith("root")) {
      throw new RuntimeException("Not implemented yet");
    } else {
      throw new RuntimeException("Not implemented yet");
    }
  }
}
