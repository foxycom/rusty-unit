package de.unipassau.testify.exec;

import java.io.IOException;
import java.net.InetSocketAddress;
import java.nio.channels.AsynchronousServerSocketChannel;

public class TestServer {

  public TestServer() throws IOException {
    var server = AsynchronousServerSocketChannel.open();
    server.bind(new InetSocketAddress("localhost", 4444));
    var acceptFuture = server.accept();
  }
}
