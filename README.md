[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.6604714.svg)](https://doi.org/10.5281/zenodo.6604714)


  
# How To

## Prerequisites
To use RustyUnit, you need a few things to be installed on your machine:
- PostgreSQL 14+ (change DB properties in `rusty-unit/src/main/resources/config.properties`)
- Redis (RustyUnit assumes default configuration, i.e., port `6379` on `localhost`)
- Rust / Cargo
- Java 17+
- Gradle 8

Start the PostgreSQL and Redis servers before proceeding. RustyUnit will create and initialize on itself Redis data and PostgreSQL tables that it needs. In the properties file, you can also set other parameters before building. If you intend to run the Java code from an IDE, make sure to install the [Lombok plugin](https://projectlombok.org).

## Clone
The build process expects the presence of the source code of the Rust compiler, which is included as submodule in the repository. To clone the repository together with the submodule use:

`git clone --recurse-submodules https://github.com/foxycom/rusty-unit.git`

## Build RustyUnit
First of all, you need to build RustyUnit's binaries. Run in root:
```
make build
```

The command automatically installs the needed Rust compiler toolchain and produces a `bin` folder with three binaries: `analysis`, `instrumentation`, and `rusty-unit.jar`. 

**Note:** We tested the code with the Rust compiler version `1.61.0-nightly`, which is also included as a Git submodule in `compiler/rust` and linked into the final binary of RustyUnit. We need a nightly compiler due to features RustyUnit exploits. Since nightly versions often include breaking changes, RustyUnit might not work with other versions. 


## Analysis
To use them with the case study subjects, set the required environment variables (use absolute paths):
```
export RU_BIN=<path>/bin
export RU_MONITOR=<path>/compiler/src/monitor.rs
```

Now, go to one of the crates within the `evaluation` directory. RustyUnit first needs to analyze the structure of the crate. In the root of a crate, run:
```
make analyze
```
This creates the `analysis` directory, which contains HIR and MIR analysis files. The HIR JSON file contains available types and functions along with the constant pool.

The MIR directory contains two JSON files per function, pre- and post-instrumentation. Each file contains among others the CDG of the function, its DOT representation, locals, and basic blocks.

## Generation
From a crate's root, select one of the algorithms to run:

```
// Random search
make random-search
// Unseeded DynaMOSA
make dynamosa-poor
// Seeded DynaMOSA
make dynamosa
```

The commands start the search process with the parameters set in `rusty-unit/src/main/resources/config.properties` and the respective algorithm. At the, a copy of the crate with the generated test is created in the directory `rusty-unit-0`. Add `RUN=<n>` to change the run number and put the result into `rusty-unit-<n>`:

```
make dynamosa RUN=<n>
```

To rerue final test suite, switch into the generated `rusty-unit-<n>` directory and execute:
```
make execute
```
## Coverage
You can also produce the coverage data of a generated test suite with:
```
make coverage
```
This creates a `coverage` directory with a `data.json` file, which is the coverage output of Rust's [instrument-coverage](https://doc.rust-lang.org/rustc/instrument-coverage.html), which only consists of line and region coverage. Luckily, RustyUnit stores the basic block coverage for each run and generation in the PostgreSQL database. You can find it in the table for the respective algorithm:
* `experiments_random` for random search
* `experiments_dynamosa` for unseeded DynaMOSA
* `experiments_seeded_dynamosa` for seeded DynaMOSA

## Evaluation Results
You can download the evaluation case study subjects described in the master's thesis from [Zenodo](https://doi.org/10.5281/zenodo.6604714). Each crate we evaluated features a `rusty-unit` directory, which contains 30 run results for each of the algorithms, i.e., `rusty-unit-0` to `rusty-unit-29`. You can run the final test suites and compute instrument-coverage coverage values the same way as described above. You can also find the extracted results for each run and generation that we extracted from PostgreSQL in `experiments/data`.