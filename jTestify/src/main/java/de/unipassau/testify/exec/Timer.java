package de.unipassau.testify.exec;

public class Timer {
  private long startTime;
  private boolean started;

  public void start() {
    if (started) {
      throw new RuntimeException("Already started");
    }
    startTime = System.currentTimeMillis();
    started = true;
  }

  public long end() {
    if (!started) {
      throw new RuntimeException("Not started yet");
    }

    long endTime = System.currentTimeMillis();
    started = false;
    return endTime - startTime;
  }
}
