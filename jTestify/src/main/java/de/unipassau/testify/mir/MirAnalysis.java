package de.unipassau.testify.mir;

import de.unipassau.testify.mir.Branch.RootBranch;
import java.util.List;

public class MirAnalysis {
  public static List<Branch> getBranches() {
    return List.of(
        new RootBranch(2, "clals", "method", "desc"),
        new RootBranch(3, "clals", "method", "desc")

    );
  }
}
