package de.unipassau.testify.server;

import de.unipassau.testify.mir.BasicBlock;
import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.PrintWriter;
import java.net.Socket;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import org.javatuples.Pair;

public class DataRequester {
  private static final int PORT = 3333;
  private static final String HOST = "127.0.0.1";

  private final Socket socket;
  private final PrintWriter out;
  private final BufferedReader in;

  public DataRequester() throws IOException {
    socket = new Socket(HOST, PORT);
    out = new PrintWriter(socket.getOutputStream(), true);
    in = new BufferedReader(new InputStreamReader(socket.getInputStream()));
  }

  public Map<Integer, List<Pair<BasicBlock, Double>>> request() throws IOException {
    out.println("get");
    String line;

    Map<Integer, List<Pair<BasicBlock, Double>>> coverage = new HashMap<>();
    while ((line = in.readLine()) != null) {
      var result = TraceParser.parse(line);

      coverage.putIfAbsent(result.getValue0(), new ArrayList<>());
      coverage.get(result.getValue0()).add(Pair.with(result.getValue1(), result.getValue2()));
    }

    return coverage;
  }

  public static void main(String[] args) throws IOException {
    var dataRequester = new DataRequester();
    var coverage = dataRequester.request();
    System.out.println(coverage);
  }
}
