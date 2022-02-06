package de.unipassau.testify.mir;

import java.util.Objects;

public abstract class Branch {

  protected final int globalId;
  protected final int localId;

  /**
   * Constructs a new branch using the given ID, which must not be negative. It is the
   * responsibility of the caller to ensure that the given ID is unique.
   *
   * @param globalId the global ID of the branch (i.e., module id)
   * @param localId  the local ID of the branch (i.e., function id)
   * @throws IllegalArgumentException if the ID is negative
   */
  Branch(final int globalId, final int localId) throws IllegalArgumentException {
    if (globalId < 0 || localId < 0) {
      throw new IllegalArgumentException("ID must not be negative");
    }

    this.globalId = globalId;
    this.localId = localId;
  }

  /**
   * Returns the global ID of this branch.
   *
   * @return the ID
   */
  public final int getGlobalId() {
    return globalId;
  }

  public final int getLocalId() {
    return localId;
  }

  public boolean isDecisionBranch() {
    return false;
  }

  public boolean isRootBranch() {
    return false;
  }

  public RootBranch toRootBranch() {
    throw new RuntimeException("Not with me");
  }

  public DecisionBranch toDecisionBranch() {
    throw new RuntimeException("Not with me");
  }

  /*
   * The following implementations of equals() and hashCode() are appropriate and sufficient for
   * use in subclasses because the only "relevant" field that constitutes our notion of equality
   * is the id of a branch instance. The id is ensured to be globally unique when constructing a
   * new branch in the BranchDistanceMethodVisitor class. By construction it can never happen
   * that two branches are assigned the same id but represent different branches, and similarly,
   * it can never happen that branches with different ids represent the same branch.
   */

  @Override
  public boolean equals(final Object other) {
    if (this == other) {
      return true;
    }

    if (other == null || getClass() != other.getClass()) {
      return false;
    }

    final Branch that = (Branch) other;
    return this.getGlobalId() == that.getGlobalId() && this.getLocalId() == that.getGlobalId();
  }

  @Override
  public int hashCode() {
    return Objects.hash(getGlobalId(), getLocalId());
  }

  @Override
  public String toString() {
    return "Branch(" + globalId + ")";
  }

  /**
   * Represents the root branch (i.e., the entry point) of a method, which is taken when the method
   * is invoked.
   */
  static class RootBranch extends Branch {

    /**
     * Creates a new root branch with the given ID for the specified method. The method is given by
     * the fully qualified name of its owner class, the name of the method itself, and the
     * descriptor of the method (the latter is required to distinguish between overloaded methods).
     *
     * @param globalId the global ID of the root branch (must not be negative)
     * @param localId  the local ID of the root branch (must not be negative)
     * @throws IllegalArgumentException if one of the arguments is invalid (see above)
     */
    RootBranch(final int globalId, final int localId) throws IllegalArgumentException {
      super(globalId, localId);
    }

    @Override
    public boolean isRootBranch() {
      return true;
    }

    @Override
    public RootBranch toRootBranch() {
      return this;
    }

    @Override
    public String toString() {
      return String.format("Root of %d:%d", getLocalId(), getLocalId());
    }


  }

  static class DecisionBranch extends Branch {

    private final int blockId;

    /*
     * Stores whether this is a {@code true} branch or a {@code false} branch. {@code true} branches
     * are taken when the branch condition of the source node evaluates to {@code true}. Conversely,
     * {@code false} branches are taken when the branch condition evaluates to {@code false}. In
     * Java byte code, branches are often implemented using jump instructions. In that respect, a
     * {@code true} branch is only taken when a jump is
     * <em>not</em> performed (i.e., the jump condition is {@code false}), and a {@code false}
     * branch is taken when a jump occurs (i.e., the jump condition is {@code true}). This implies
     * that a jump condition has to be the logical negation of the branch condition.
     */
    //private boolean value;

    /**
     * Constructs a new branch using the specified non-negative ID, the non-{@code null} branch node
     * as origin and the value telling whether this is a {@code true} branch or {@code false}
     * branch.
     *
     * @throws IllegalArgumentException if an argument is invalid (see above)
     */
    DecisionBranch(final int globalId, final int localId, final int blockId)
        throws IllegalArgumentException {
      super(globalId, localId);

      this.blockId = blockId;
    }

    public int getBlockId() {
      return blockId;
    }

    @Override
    public boolean isDecisionBranch() {
      return true;
    }

    @Override
    public DecisionBranch toDecisionBranch() {
      return this;
    }

    @Override
    public String toString() {
      return String.format("Decision %d:%d:%d", getGlobalId(), getLocalId(), getBlockId());
    }

    @Override
    public boolean equals(Object o) {
      if (this == o) {
        return true;
      }
      if (o == null || getClass() != o.getClass()) {
        return false;
      }
      if (!super.equals(o)) {
        return false;
      }

      DecisionBranch that = (DecisionBranch) o;
      return globalId == that.globalId && localId == that.localId && blockId == that.blockId;
    }

    @Override
    public int hashCode() {
      return Objects.hash(super.hashCode(), blockId);
    }
  }
}
