package de.unipassau.testify.exec;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.PrintWriter;
import java.net.Socket;

public class TestClient {
  private Socket clientSocket;
  private PrintWriter out;
  private BufferedReader in;

  public void startConnection(String ip, int port) throws IOException {
    clientSocket = new Socket(ip, port);
    out = new PrintWriter(clientSocket.getOutputStream(), true);
    in = new BufferedReader(new InputStreamReader(clientSocket.getInputStream()));
  }

  public String sendMessage(String msg) throws IOException {
    out.println(msg);
    return in.readLine();
  }

  public void stopConnection() throws IOException {
    in.close();
    out.close();
    clientSocket.close();
  }

  public static void main(String[] args) throws IOException {
    var client = new TestClient();
    client.startConnection("localhost", 4444);
    client.sendMessage("Hello");
  }
}
