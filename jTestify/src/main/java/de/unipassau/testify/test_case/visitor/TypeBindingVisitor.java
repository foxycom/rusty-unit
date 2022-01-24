package de.unipassau.testify.test_case.visitor;

import de.unipassau.testify.test_case.type.TypeBinding;

public interface TypeBindingVisitor {

  String visit(TypeBinding typeBinding);
}
