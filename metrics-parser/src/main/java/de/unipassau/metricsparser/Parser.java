package de.unipassau.metricsparser;

import com.jayway.jsonpath.JsonPath;
import com.lexicalscope.jewel.cli.CliFactory;
import com.lexicalscope.jewel.cli.Option;
import java.io.IOException;
import java.nio.file.FileVisitOption;
import java.nio.file.Files;
import java.nio.file.Paths;

public class Parser {

  interface CLI {
    @Option
    String getPath();
  }

  public static void main(String[] args) throws IOException {
    CLI cli = CliFactory.parseArguments(CLI.class, args);
    var path = Paths.get(cli.getPath());
    try (var stream = Files.walk(path, Integer.MAX_VALUE, FileVisitOption.FOLLOW_LINKS)) {
      var metrics = stream
          .filter(p -> p.toFile().isFile())
          .map(p -> {
        String content;
        try {
          content = Files.readString(p);
        } catch (IOException e) {
          throw new RuntimeException(e);
        }
        String name = JsonPath.read(content, "$.name");
        double lloc = JsonPath.read(content, "$.metrics.loc.lloc");
        double methods = JsonPath.read(content, "$.metrics.nom.total");

        return new Metrics(name, (int) methods, (int) lloc);
      }).reduce(new Metrics("", 0, 0), (acc, x) -> new Metrics("", acc.methods() + x.methods(), acc.lloc() + x.lloc()));

      System.out.println(metrics);
    }

  }

}
