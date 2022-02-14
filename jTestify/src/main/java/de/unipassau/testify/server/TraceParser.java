package de.unipassau.testify.server;

import de.unipassau.testify.mir.BasicBlock;
import org.javatuples.Triplet;

public class TraceParser {

  /**
   * A branch line example: <test id> $<global id>$ branch[<block id> <distance>]
   * A root line example: <test id> $<global id>$ root
   */
  public static Triplet<Integer, BasicBlock, Double> parse(String line) {
    int testId;
    try {
      testId = Integer.parseInt(line.substring(0, line.indexOf(" ")));
    } catch (NumberFormatException e) {
      return null;
    }

    String globalId = line.substring(line.indexOf("$"), line.lastIndexOf("$") + 1);

    line = line.substring(line.lastIndexOf("$") + 2);
    if (line.startsWith("branch")) {
      var dataBegin = line.indexOf("[") + 1;
      var dataEnd = line.length() - 1;
      var data = line.substring(dataBegin, dataEnd).split(" ");

      var blockId = Integer.parseInt(data[0]);
      var distance = Double.parseDouble(data[1]);

      return Triplet.with(
          testId,
          BasicBlock.of(globalId, blockId),
          distance
      );
    } else if (line.startsWith("root")) {
      /*var dataBegin = line.indexOf("[") + 1;
      var dataEnd = line.indexOf("]");
      var data = line.substring(dataBegin, dataEnd).split(" ");*/

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
