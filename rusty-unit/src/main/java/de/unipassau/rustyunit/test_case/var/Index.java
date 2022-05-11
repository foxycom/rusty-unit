package de.unipassau.rustyunit.test_case.var;

import de.unipassau.rustyunit.test_case.TestCase;
import de.unipassau.rustyunit.test_case.type.prim.UInt.USize;

public class Index extends VarReference {

    private int value;

    public Index(TestCase testCase, int value) {
        super(testCase, USize.INSTANCE);
        this.value = value;
    }

    public int value() {
        return value;
    }
}
