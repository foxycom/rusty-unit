package de.unipassau.testify.mir;


import static com.google.common.truth.Truth.assertThat;

import de.unipassau.testify.test_case.TestCase;
import java.util.Set;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

public class CDGTest {

  private CDG<BasicBlock, TestCase> cdg;

  @BeforeEach
  public void setUp() throws Exception {
    cdg = CDG.parse("id",
          "{\"nodes\":[42069,1,2,3,4,6,5,0,7],\"node_holes\":[],\"edge_property\":\"directed\",\"edges\":[[1,2,1],[1,3,1],[1,4,1],[4,5,1],[4,6,1],[0,7,1],[0,1,1],[0,8,1],[0,0,1]]}");
  }

  @Test
  public void testIndependentTargets() {
    var independetTargets = cdg.independentTargets();

    assertThat(independetTargets).containsExactly(new BasicBlock("id", 1),
          new BasicBlock("id", 0),
          new BasicBlock("id", 7));
  }

  @Test
  public void testDependentTargets() {
    var dependentTargets = cdg.dependentTargets(new BasicBlock("id", 1));
    assertThat(dependentTargets).containsExactly(new BasicBlock("id", 2), new BasicBlock("id", 3), new BasicBlock("id", 4));
  }

  @Test
  public void testAllSubTargets() {
    var subTargets = cdg.allSubTargets(new BasicBlock("id", 1));
    assertThat(subTargets).containsExactly(
          new BasicBlock("id", 2),
          new BasicBlock("id", 3),
          new BasicBlock("id", 4),
          new BasicBlock("id", 5),
          new BasicBlock("id", 6)
    );
  }

  @Test
  public void testApproachLevelIsTwo() {
    var approachLevel = cdg.approachLevel(new BasicBlock("id", 6), Set.of(new BasicBlock("id", 1)));
    assertThat(approachLevel).isEqualTo(2);
  }

  @Test
  public void testApproachLevelIsZero() {
    var approachLevel = cdg.approachLevel(new BasicBlock("id", 3), Set.of(new BasicBlock("id", 1), new BasicBlock("id", 3)));
    assertThat(approachLevel).isEqualTo(0);
  }

  @Test
  public void testPathTo() {
    var target = new BasicBlock("id", 10);
    var path = cdg.pathTo(target);

    var ids = path.stream().map(BasicBlock::blockId).toList();

    assertThat(ids).containsExactly(1, 2, 7, 10);
  }

  @Test
  public void testComplexPath() {
    cdg = CDG.parse("id", "{\"nodes\":[42069,3,6,24,25,26,4,13,14,12,9,10,11,15,16,17,29,53,52,49,27,36,37,35,32,7,8,18,19,20,50,51,54,33,34,38,39,40,2,21,22,23,47,48,55,30,31,41,42,43,58,65,64,61,56,57,44,45,46,62,63,66,59,60,67,70,71,72,73,76,68,69,75,74,0,1,77],\"node_holes\":[],\"edge_property\":\"directed\",\"edges\":[[1,2,1],[1,3,1],[1,4,1],[1,5,1],[1,6,1],[1,7,1],[1,8,1],[1,9,1],[1,10,1],[8,11,1],[8,12,1],[8,13,1],[8,14,1],[8,15,1],[5,16,1],[5,17,1],[5,18,1],[5,19,1],[5,20,1],[5,21,1],[5,22,1],[5,23,1],[5,24,1],[9,25,1],[9,26,1],[9,27,1],[9,28,1],[9,29,1],[17,30,1],[17,31,1],[17,32,1],[22,33,1],[22,34,1],[22,35,1],[22,36,1],[22,37,1],[10,38,1],[10,39,1],[10,40,1],[10,41,1],[18,42,1],[18,43,1],[18,44,1],[23,45,1],[23,46,1],[23,47,1],[23,48,1],[23,49,1],[19,50,1],[19,51,1],[19,52,1],[19,53,1],[19,54,1],[19,55,1],[24,4,1],[24,56,1],[24,57,1],[24,58,1],[51,59,1],[51,60,1],[51,61,1],[52,62,1],[52,63,1],[52,64,1],[53,65,1],[53,66,1],[53,67,1],[53,68,1],[53,69,1],[53,70,1],[53,71,1],[68,72,1],[68,73,1],[0,74,1],[0,75,1],[0,1,1],[0,76,1],[0,0,1]]}");
    var target = new BasicBlock("id", 73);
    var path = cdg.pathTo(target);

    assertThat(path).isNotNull();
    var ids = path.stream().map(BasicBlock::blockId).toList();
    assertThat(ids).containsExactly(42069, 3, 26, 49, 61, 73);
  }
}