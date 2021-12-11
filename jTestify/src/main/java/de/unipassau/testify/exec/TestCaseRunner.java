package de.unipassau.testify.exec;

import java.io.File;
import java.io.IOException;

public class TestCaseRunner {

  public TestCaseRunner() {
  }

  public void run(String path) throws IOException, InterruptedException {
    var directory = new File(path);
    var process = new ProcessBuilder("cargo", "test", "testify_tests")
        .directory(directory)
        .start();
    var result = process.waitFor();
    System.out.println("Result is " + result);
  }
}
