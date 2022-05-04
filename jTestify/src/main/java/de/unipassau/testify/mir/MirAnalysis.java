package de.unipassau.testify.mir;

import static de.unipassau.testify.Constants.MIR_LOG_PATH;

import de.unipassau.testify.exec.Timer;
import de.unipassau.testify.metaheuristics.chromosome.AbstractTestCaseChromosome;
import de.unipassau.testify.metaheuristics.fitness_functions.FitnessFunction;
import de.unipassau.testify.metaheuristics.fitness_functions.MinimizingFitnessFunction;
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

  public MirAnalysis(String mirPath) {
    this.mirPath = mirPath;
    cdgs = parseCDGs();
  }

  private Map<String, CDG<MinimizingFitnessFunction<C>, C>> parseCDGs() {
    System.out.println("-- Graph analysis");
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
    for (var target : targets) {
      if (covered(target, population)) {
        updatedTargets.remove(target);
        visit(target, updatedTargets, population);
      }
    }

    return updatedTargets;
  }

  public void visit(MinimizingFitnessFunction<C> target,
      Set<MinimizingFitnessFunction<C>> targets, List<C> population) {
    var cdg = cdgs.get(target.id());
    var dependentTargets = cdg.dependentTargets(target);
    for (MinimizingFitnessFunction<C> dependentTarget : dependentTargets) {
      if (visitedBlocks.contains(dependentTarget)) {
        continue;
      }

      if (!covered(dependentTarget, population)) {
        targets.add(dependentTarget);
        visitedBlocks.add(dependentTarget);
      } else {
        visit(dependentTarget, targets, population);
      }
    }
  }

  public boolean covered(MinimizingFitnessFunction<C> target, List<C> population) {
    return population.stream()
        .anyMatch(chromosome -> chromosome.getFitness(target) == 0.0);
  }
}
