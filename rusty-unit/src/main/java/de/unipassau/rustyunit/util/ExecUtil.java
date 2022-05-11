package de.unipassau.rustyunit.util;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;

public class ExecUtil {

  public static String escapeJson(String json) throws IOException, InterruptedException {
    var pb = new ProcessBuilder("/bin/bash", "-c", String.format("echo '%s' | jq -aR .", json));
    var process = pb.start();

    BufferedReader reader =
        new BufferedReader(new InputStreamReader(process.getInputStream()));
    var sb = new StringBuilder();
    String line = null;
    while ((line = reader.readLine()) != null) {
      sb.append(line);
      sb.append(System.getProperty("line.separator"));
    }

    process.waitFor();
    return sb.toString();
  }
}
