package de.unipassau.rustyunit.test_case.metadata;

public interface MetaData {
    void setFails(boolean value);
    boolean fails();
    void setFilePath(String filePath);
    String filePath();
    int id();
}
