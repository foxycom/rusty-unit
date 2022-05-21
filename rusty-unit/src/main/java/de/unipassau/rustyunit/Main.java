package de.unipassau.rustyunit;

import com.lexicalscope.jewel.cli.CliFactory;
import com.lexicalscope.jewel.cli.Option;
import de.unipassau.rustyunit.algorithm.Algorithm;
import de.unipassau.rustyunit.test_case.seed.SeedOptions;
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

    @Option(longName = "seed-all-methods")
    boolean seedMethods();

    @Option(longName = "seed-constant-pool")
    boolean seedConstantPool();

    @Option(longName = "seed-random-population")
    boolean seedRandomPopulation();

    @Option(longName = "seed-by-class")
    boolean seedByClass();

    @Option(longName = "features", defaultValue = "")
    String features();
  }

  public static void main(String[] args) throws IOException, InterruptedException {
    var cli = CliFactory.parseArguments(CLI.class, args);
    SeedOptions.setInitialRandomPopulation(cli.seedRandomPopulation());
    SeedOptions.setUseAllMethods(cli.seedMethods());
    SeedOptions.setUseConstantPool(cli.seedConstantPool());

    switch (Algorithm.from(cli.getAlgorithm())) {
      case MOSA -> TestsGenerator.runMOSA(cli);
      case DYNA_MOSA -> TestsGenerator.runDynaMOSA(cli);
      case RANDOM -> TestsGenerator.runRandomSearch(cli);
    }
  }
}
