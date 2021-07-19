## Current edge cases that are not handled

- To run a generated test case, one must add the instrumented module to lib.rs / main.rs, but what happens if the instrumented file is not in the same package, but in a subdirectory?