package de.unipassau.testify.mir;

import com.google.common.base.Preconditions;

public record BasicBlock(String globalId, int blockId) {

  public BasicBlock {
    Preconditions.checkState(blockId >= 0);
  }

  public static BasicBlock of(String globalId, int blockId) {
    return new BasicBlock(globalId, blockId);
  }

  @Override
  public String toString() {
    return String.format("%s:%d", globalId, blockId);
  }
}
