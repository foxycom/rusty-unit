package de.unipassau.testify.mir;


import com.fasterxml.jackson.databind.ObjectMapper;
import de.unipassau.testify.exec.Timer;
import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
import de.unipassau.testify.test_case.primitive.PrimitiveValue;
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

public class MirAnalysis<C extends AbstractTestCaseChromosome<C>> {

  private final Map<String, CDG<MinimizingFitnessFunction<C>, C>> cdgs;
  private final Set<MinimizingFitnessFunction<C>> visitedBlocks = new HashSet<>();
  private final String mirPath;

  private static final Set<PrimitiveValue<?>> CONSTANT_POOL = new HashSet<>();

  public MirAnalysis(String mirPath) {
    this.mirPath = mirPath;
    cdgs = parseCDGs();
    MirAnalysis.CONSTANT_POOL.addAll(parseConstants());
  }

  private Set<PrimitiveValue<?>> parseConstants() {
    System.out.println("-- Constant pool analysis");
    Set<PrimitiveValue<?>> constants = new HashSet<>();
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
              for (var c : jsonConstants) {
                constants.add(objectMapper.readValue(c.toString(), PrimitiveValue.class));
              }
            } catch (IOException e) {
              throw new RuntimeException(e);
            }
          });
    } catch (IOException e) {
      throw new RuntimeException("Could not parse constant pool", e);
    }

    return constants;
  }

  private Map<String, CDG<MinimizingFitnessFunction<C>, C>> parseCDGs() {
    System.out.println("-- Control dependence graph analysis");
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
              var cdg = CDG.<MinimizingFitnessFunction<C>, C>parse(globalId,
                  jsonRoot.getString("cdg"));
              cdgs.put(globalId, cdg);
            } catch (IOException e) {
              throw new RuntimeException(e);
            }
          });
    } catch (IOException e) {
      throw new RuntimeException("Could not parse CDGs from mir logs", e);
    }

    var elapsedTime = timer.end();
    System.out.printf("-- Finished. Took %ds%n", TimeUnit.MILLISECONDS.toSeconds(elapsedTime));
    return cdgs;
  }

  public static Set<PrimitiveValue<?>> constantPool() {
    return CONSTANT_POOL;
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

    System.out.printf("\t>> Number of targets to cover next: %d (+%d)%n", updatedTargets.size(), newTargets);
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
