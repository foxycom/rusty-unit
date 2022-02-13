package de.unipassau.testify.server;

import de.unipassau.testify.mir.BasicBlock;
import org.javatuples.Triplet;

public class TraceParser {

  /**
   * A branch line example: <test id> branch[<global id> <local id> <block id> <distance>]
   * A root line example: <test id> root[<global id> <local id>]
   */
  public static Triplet<Integer, BasicBlock, Double> parse(String line) {
    int testId;
    try {
      testId = Integer.parseInt(line.substring(0, line.indexOf(" ")));
    } catch (NumberFormatException e) {
      return null;
    }

    line = line.substring(line.indexOf(" ") + 1);
    if (line.startsWith("branch")) {
      var dataBegin = line.indexOf("[") + 1;
      var dataEnd = line.length() - 1;
      var data = line.substring(dataBegin, dataEnd).split(" ");

      var globalId = Integer.parseInt(data[0]);
      var blockId = Integer.parseInt(data[1]);
      var distance = Double.parseDouble(data[2]);

      return Triplet.with(
          testId,
          BasicBlock.of(globalId, blockId),
          distance
      );
    } else if (line.startsWith("root")) {
      var dataBegin = line.indexOf("[") + 1;
      var dataEnd = line.indexOf("]");
      var data = line.substring(dataBegin, dataEnd).split(" ");

      var globalId = Integer.parseInt(data[0]);

      return Triplet.with(
          testId,
          BasicBlock.of(globalId, 0),
          0.0
      );
    } else {
      throw new RuntimeException("Not implemented yet");
    }
  }
}
