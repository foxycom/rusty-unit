package de.unipassau.testify.mir;


import static com.google.common.truth.Truth.assertThat;

import de.unipassau.testify.test_case.TestCase;
import java.util.List;
import java.util.stream.Collectors;
import java.util.stream.Stream;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

public class CDGTest {

  private CDG<BasicBlock, TestCase> cdg;

  @BeforeEach
  public void setUp() throws Exception {
    cdg = CDG.parse("id",
        "{\"nodes\":[42069,1,2,3,4,5,6,7,8,9,10],\"node_holes\":[],\"edge_property\":\"directed\",\"edges\":[[1,2,1],[1,3,1],[1,4,1],[1,5,1],[2,6,1],[2,7,1],[3,8,1],[4,9,1],[7,10,1],[1,1,1]]}");
  }

  @Test
  public void testIndependentTargets() {
  }

  @Test
  public void testDependentTargets() {
  }

  @Test
  public void testPathTo() {
    var target = new BasicBlock("id", 10);
    var path = cdg.pathTo(target);

    var ids = path.stream().map(BasicBlock::blockId).toList();

    assertThat(ids).containsExactly(1, 2, 7, 10);
  }
}