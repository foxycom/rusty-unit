package de.unipassau.rustyunit.test_case.visitor;

import de.unipassau.rustyunit.test_case.type.TypeBinding;

public interface TypeBindingVisitor {

  String visit(TypeBinding typeBinding);
}
