package de.unipassau.testify.exec;

import static org.junit.jupiter.api.Assertions.*;

import java.io.IOException;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

class TestCaseRunnerTest {

  private TestCaseRunner runner;

  @BeforeEach
  void setUp() {
    runner = new TestCaseRunner();
  }

  @Test
  void testRun() throws IOException, InterruptedException {
    runner.run("/Users/tim/Documents/master-thesis/evaluation/trying-main");
  }
}