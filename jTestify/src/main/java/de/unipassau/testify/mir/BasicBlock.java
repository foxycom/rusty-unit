package de.unipassau.testify.mir;

import com.google.common.base.Preconditions;

public record BasicBlock(int globalId, int blockId) {

  public BasicBlock {
    Preconditions.checkState(globalId >= 0);
  }

  public static BasicBlock of(int globalId, int blockId) {
    return new BasicBlock(globalId, blockId);
  }

  @Override
  public String toString() {
    return String.format("%d:%d", globalId, blockId);
  }
}
