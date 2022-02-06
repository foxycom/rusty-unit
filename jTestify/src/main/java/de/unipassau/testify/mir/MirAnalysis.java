package de.unipassau.testify.mir;

import de.unipassau.testify.mir.Branch.RootBranch;
import java.util.List;

public class MirAnalysis {

  public static final String MIR_LOG_PATH = "/Users/tim/Documents/master-thesis/testify/log/mir.log";

  private static List<Branch> branches;

  private static List<Branch> parseBranches() {
    throw new RuntimeException("Not implemented yet");
  }

  public static List<Branch> getBranches() {
    return branches;
  }

  public static Branch getDecisionBranch(final int globalId, final int localId, final int blockId) {
    return branches
        .stream()
        .filter(Branch::isDecisionBranch)
        .map(Branch::toDecisionBranch)
        .filter(branch -> branch.getGlobalId() == globalId
            && branch.getLocalId() == localId
            && branch.getBlockId() == blockId)
        .findFirst()
        .get();
  }

  public static Branch getRootBranch(final int globalId, final int localId) {
    return branches
        .stream()
        .filter(Branch::isRootBranch)
        .map(Branch::toRootBranch)
        .filter(branch -> branch.getGlobalId() == globalId
            && branch.getLocalId() == localId)
        .findFirst()
        .get();
  }
}
