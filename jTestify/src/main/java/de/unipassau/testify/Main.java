package de.unipassau.testify;

import com.lexicalscope.jewel.cli.CliFactory;
import com.lexicalscope.jewel.cli.Option;
import de.unipassau.testify.algorithm.Algorithm;
import java.io.IOException;
import java.util.List;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class Main {

  private static final Logger logger = LoggerFactory.getLogger(Main.class);

  public interface CLI {

    @Option(shortName = "c", longName = "crate")
    String getCrateRoot();

    @Option(shortName = "m")
    List<String> getMainFiles();

    @Option(shortName = "n")
    String getCrateName();

    @Option(shortName = "a", longName = "algo")
    String getAlgorithm();

    @Option(longName = "hir")
    String getHirPath();

    @Option(longName = "mir")
    String getMirPath();

    @Option(shortName = "i", longName = "instrumenter")
    String getInstrumenterPath();
  }

  public static void main(String[] args) throws IOException, InterruptedException {
    var cli = CliFactory.parseArguments(CLI.class, args);
    switch (Algorithm.from(cli.getAlgorithm())) {
      case MOSA -> TestsGenerator.runMOSA(cli);
      case DYNA_MOSA -> TestsGenerator.runDynaMOSA(cli);
      case RANDOM -> TestsGenerator.runRandomSearch(cli);
    }
  }
}
