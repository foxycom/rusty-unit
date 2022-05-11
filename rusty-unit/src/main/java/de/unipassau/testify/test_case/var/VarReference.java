package de.unipassau.testify.test_case.var;

import com.google.common.base.Preconditions;
import com.google.common.collect.Streams;
import de.unipassau.testify.test_case.TestCase;
import de.unipassau.testify.test_case.statement.Statement;
import de.unipassau.testify.test_case.type.Type;
import de.unipassau.testify.test_case.type.TypeBinding;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.UUID;
import java.util.stream.IntStream;
import org.javatuples.Pair;

public class VarReference {

  private final Type type;
  private final UUID id;

  private TestCase testCase;
  private TypeBinding binding;

  public VarReference(TestCase testCase, Type type) {
    this(testCase, type, new TypeBinding());
  }

  public VarReference(TestCase testCase, Type type, TypeBinding typeBinding) {
    this.id = UUID.randomUUID();
    this.testCase = testCase;
    this.type = type;
    this.binding = typeBinding;
  }

  public VarReference(VarReference other) {
    this.id = other.id;
    this.testCase = other.testCase;
    this.type = other.type.copy();
    this.binding = other.binding.copy();
  }

  public Type type() {
    return type;
  }

  public int position() {
    return Streams.zip(IntStream.range(0, testCase.size()).boxed(),
            testCase.getStatements().stream(), Pair::with)
        .filter(pair -> pair.getValue1().returnValue().isPresent()
            && pair.getValue1().returnValue().get().equals(this))
        .map(Pair::getValue0)
        .findFirst().get();

  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    VarReference that = (VarReference) o;
    return type.equals(that.type) && id.equals(that.id);
  }

  @Override
  public int hashCode() {
    return Objects.hash(type, id);
  }

  public Statement definedBy() {
    return testCase.getStatements().stream().filter(stmt ->
        stmt.returnValue().isPresent() && stmt.returnValue().get() == this).findFirst().get();
  }

  public boolean isConsumableAt(int pos) {
    if (isConsumed()) {
      return false;
    }

    var borrowedAt = borrowedAt();
    if (borrowedAt.isEmpty()) {
      return pos > position();
    } else {
      var lastBorrowedPos = borrowedAt.get(borrowedAt.size() - 1);
      return lastBorrowedPos < pos;
    }
  }

  public TestCase testCase() {
    return testCase;
  }

  public boolean isConsumed() {
    return testCase.getStatements().stream().anyMatch(stmt -> stmt.consumes(this));
  }

  public Optional<Integer> consumedAt() {
    var consumingStmt = Streams.zip(IntStream.range(0, testCase.size()).boxed(),
            testCase.getStatements().stream(), Pair::with)
        .filter(pair -> pair.getValue1().consumes(this))
        .findFirst();

    if (consumingStmt.isPresent()) {
      return consumingStmt.map(Pair::getValue0);
    } else {
      return Optional.empty();
    }
  }

  public List<Integer> usedAt() {
    return Streams.zip(IntStream.range(0, testCase.size()).boxed(),
        testCase.getStatements().stream(), Pair::with)
        .filter(pair -> pair.getValue1().borrows(this) || pair.getValue1().consumes(this))
        .map(Pair::getValue0)
        .toList();
  }

  public List<Integer> borrowedAt() {
    return Streams.zip(IntStream.range(0, testCase.size()).boxed(),
            testCase.getStatements().stream(), Pair::with)
        .filter(pair -> pair.getValue1().borrows(this))
        .map(Pair::getValue0)
        .toList();
  }

  public boolean isBorrowableAt(int pos) {
    var consumedPos = consumedAt();
    return consumedPos.map(integer -> pos < integer).orElseGet(() -> position() < pos);
  }

  public VarReference copy(TestCase testCase) {
    var copy = new VarReference(this);
    copy.testCase = testCase;
    return copy;
  }

  public TypeBinding getBinding() {
    return binding;
  }

  public void setBinding(TypeBinding binding) {
    Preconditions.checkNotNull(binding);
    this.binding = binding;
  }

  @Override
  public String toString() {
    return String.format("Var at %d", position());
  }
}
