package de.unipassau.testify.util;

public interface SelfTyped<S extends SelfTyped<S>> {

  /**
   * <p>
   * Returns the runtime type of the implementor (a.k.a. "self-type"). This method must only be
   * implemented in concrete, non-abstract subclasses by returning a reference to {@code this},
   * and nothing else. Returning a reference to any other runtime type other than {@code this}
   * breaks the contract.
   * <p>
   * In other words, every concrete subclass {@code Foo} that implements the interface {@code
   * SelfTyped} must implement this method as follows:
   * <pre>{@code
   * public final class Foo implements SelfTyped<Foo> {
   *     @Override
   *     public Foo self() {
   *         return this;
   *     }
   * }
   * }</pre>
   *
   * @return a reference to the self-type
   */
  S self();
}
