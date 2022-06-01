package de.unipassau.rustyunit.mir;


import com.fasterxml.jackson.databind.ObjectMapper;
import de.unipassau.rustyunit.algorithm.dynamosa.DynaMOSA;
import de.unipassau.rustyunit.exec.Timer;
import de.unipassau.rustyunit.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.rustyunit.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import de.unipassau.rustyunit.test_case.primitive.PrimitiveValue;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.concurrent.TimeUnit;
import java.util.stream.Collectors;
import org.json.JSONObject;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class MirAnalysis<C extends AbstractTestCaseChromosome<C>> {
  private static final Logger logger = LoggerFactory.getLogger(MirAnalysis.class);

  private final Map<String, CDG<MinimizingFitnessFunction<C>, C>> cdgs;
  private final Set<MinimizingFitnessFunction<C>> visitedBlocks = new HashSet<>();
  private final String mirPath;

  private static Set<PrimitiveValue<?>> CONSTANT_POOL = new HashSet<>();
  private static Map<String, Set<PrimitiveValue<?>>> CONSTANT_POOL_BY_ID;

  public MirAnalysis(String mirPath) {
    this.mirPath = mirPath;
    cdgs = parseCDGs();
    CONSTANT_POOL_BY_ID = parseConstants();
    CONSTANT_POOL = CONSTANT_POOL_BY_ID.values().stream().collect(HashSet::new, HashSet::addAll, HashSet::addAll);
  }

  private Map<String, Set<PrimitiveValue<?>>> parseConstants() {
    logger.info("-- Constant pool analysis");
    var timer = new Timer();
    timer.start();
    Map<String, Set<PrimitiveValue<?>>> constants = new HashMap<>();
    var objectMapper = new ObjectMapper();
    var path = Paths.get(mirPath);
    try (var stream = Files.walk(path, Integer.MAX_VALUE)) {
      stream
          .filter(Files::isRegularFile)
          .filter(file -> file.getFileName().toString().startsWith("mir"))
          .forEach(file -> {
            try {
              var content = Files.readString(file);
              var jsonRoot = new JSONObject(content);
              var jsonConstants = jsonRoot.getJSONArray("constant_pool");
              var globalId = jsonRoot.getString("global_id");
              Set<PrimitiveValue<?>> localConstants = new HashSet<>();
              for (var c : jsonConstants) {
                localConstants.add(objectMapper.readValue(c.toString(), PrimitiveValue.class));
              }
              constants.put(globalId, localConstants);
            } catch (IOException e) {
              throw new RuntimeException(e);
            }
          });
    } catch (IOException e) {
      throw new RuntimeException("Could not parse constant pool", e);
    }

    logger.info(">> Finished. Took {}s", TimeUnit.MILLISECONDS.toSeconds(timer.end()));
    return constants;
  }

  private Map<String, CDG<MinimizingFitnessFunction<C>, C>> parseCDGs() {
    logger.info("-- Control dependence graph analysis");
    var timer = new Timer();
    timer.start();
    Map<String, CDG<MinimizingFitnessFunction<C>, C>> cdgs = new HashMap<>();

    var path = Paths.get(mirPath);
    try (var stream = Files.walk(path, Integer.MAX_VALUE)) {
      stream
          .filter(Files::isRegularFile)
          .filter(file -> file.getFileName().toString().startsWith("mir"))
          .forEach(file -> {
            try {
              var content = Files.readString(file);
              var jsonRoot = new JSONObject(content);
              var globalId = jsonRoot.getString("global_id");
              var branches = jsonRoot.getInt("branches");
              var assertions = jsonRoot.getInt("assertions");
              var cdg = CDG.<MinimizingFitnessFunction<C>, C>parse(globalId,
                  jsonRoot.getString("cdg"), branches, assertions);
              cdgs.put(globalId, cdg);
            } catch (IOException e) {
              throw new RuntimeException(e);
            }
          });
    } catch (IOException e) {
      throw new RuntimeException("Could not parse CDGs from mir logs", e);
    }

    int assertions = cdgs.values().stream().map(CDG::assertions).reduce(Integer::sum).get();
    double averageDepthOverCdgs = cdgs.values().stream().map(CDG::averageDepth).reduce(Double::sum).get() / cdgs.size();
    int branches = cdgs.values().stream().map(CDG::branches).reduce(Integer::sum).get();
    int targets = cdgs.values().stream().map(CDG::targets).map(Set::size).reduce(Integer::sum).get();
    logger.info("\t>> Targets: {}", targets);
    logger.info("\t>> Number of CDGs: {}, average depth: {}", cdgs.size(), averageDepthOverCdgs);
    var elapsedTime = timer.end();
    logger.info("\t>> Finished. Took {}s", TimeUnit.MILLISECONDS.toSeconds(elapsedTime));
    logger.info("\t>> Branches: {}", branches);
    logger.info("\t>> Assertions: {}", assertions);
    return cdgs;
  }



  public static Set<PrimitiveValue<?>> constantPool() {
    return CONSTANT_POOL;
  }

  public static Set<PrimitiveValue<?>> constantPool(String globalId) {
    return Objects.requireNonNull(CONSTANT_POOL_BY_ID.get(globalId));
  }

  public CDG<MinimizingFitnessFunction<C>, C> getCdgFor(String globalId) {
    return Objects.requireNonNull(cdgs.get(globalId));
  }

  public Set<MinimizingFitnessFunction<C>> targets() {
    return cdgs.values().stream()
        .map(CDG::targets)
        .flatMap(Set::stream)
        .collect(Collectors.toSet());
  }

  public Set<MinimizingFitnessFunction<C>> independentTargets() {
    return cdgs.values().stream()
        .map(CDG::independentTargets)
        .flatMap(Set::stream)

        .collect(Collectors.toSet());
  }

  public Set<MinimizingFitnessFunction<C>> targets(String globalId) {
    return cdgs.get(globalId).targets();
  }

  public Set<MinimizingFitnessFunction<C>> updateTargets(
      Set<MinimizingFitnessFunction<C>> targets, List<C> population) {
    Set<MinimizingFitnessFunction<C>> updatedTargets = new HashSet<>(targets);
    int newTargets = 0;
    for (var target : targets) {
      if (covered(target, population)) {
        updatedTargets.remove(target);
        newTargets += visit(target, updatedTargets, population);
      }
    }

    logger.info("\t>> Number of targets to cover next: {} (+{})", updatedTargets.size(), newTargets);
    return updatedTargets;
  }

  public int visit(MinimizingFitnessFunction<C> target,
      Set<MinimizingFitnessFunction<C>> targets, List<C> population) {
    int newTargets = 0;
    var cdg = cdgs.get(target.id());
    var dependentTargets = cdg.dependentTargets(target);
    for (MinimizingFitnessFunction<C> dependentTarget : dependentTargets) {
      if (visitedBlocks.contains(dependentTarget)) {
        continue;
      }

      if (!covered(dependentTarget, population)) {
        targets.add(dependentTarget);
        visitedBlocks.add(dependentTarget);
        newTargets++;
      } else {
        newTargets += visit(dependentTarget, targets, population);
      }
    }
    return newTargets;
  }

  public boolean covered(MinimizingFitnessFunction<C> target, List<C> population) {
    return population.stream()
        .anyMatch(chromosome -> chromosome.getFitness(target) == 0.0);
  }
}
