package de.unipassau.rustyunit.algorithm;

public enum Algorithm {
    RANDOM, DYNA_MOSA, MOSA;

    public static Algorithm from(String name) {
        return switch (name) {
            case "dynamosa" -> DYNA_MOSA;
            case "mosa" -> MOSA;
            case "random" -> RANDOM;
            default -> throw new RuntimeException("Not implemented");
        };
    }
}
