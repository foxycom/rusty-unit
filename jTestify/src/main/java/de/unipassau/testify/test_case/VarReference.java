package de.unipassau.testify.test_case;

import com.google.common.collect.Streams;
import de.unipassau.testify.test_case.statement.Statement;
import de.unipassau.testify.test_case.type.Type;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.UUID;
import java.util.stream.IntStream;
import org.javatuples.Pair;

public class VarReference {

  private TestCase testCase;
  private final Type type;
  private UUID id;

  public VarReference(TestCase testCase, Type type) {
    this.id = UUID.randomUUID();
    this.testCase = testCase;
    this.type = type;
  }

  public VarReference(VarReference other) {
    this.id = other.id;
    this.testCase = other.testCase;
    this.type = other.type.copy();
  }

  public Type type() {
    return type;
  }

  public int position() {
    return Streams.zip(IntStream.range(0, testCase.size()).boxed(),
            testCase.getStatements().stream(), Pair::with)
        .filter(pair -> pair.getValue1().returnValue().isPresent()
            && pair.getValue1().returnValue().get() == this)
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
    return testCase.equals(that.testCase) && type.equals(that.type) && id.equals(that.id);
  }

  @Override
  public int hashCode() {
    return Objects.hash(testCase, type, id);
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
}