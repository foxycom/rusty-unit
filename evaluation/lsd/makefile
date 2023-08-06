SHELL:=/bin/bash
INSTRUMENTER := ${RU_BIN}/instrumentation
ANALYZER := ${RU_BIN}/analysis
JAR := ${RU_BIN}/rusty-unit.jar
MONITOR := ${RU_MONITOR}
FEATURES := ""
CRATE_ROOT := $(shell realpath .)
OUTPUT := $(CRATE_ROOT)/rusty-unit
CRATE_NAME := lsd
MAIN_FILE := src/main.rs
RUN := 0

clean : clean-build clean-coverage clean-logs
	$(RM) -r analysis

clean-logs :
	$(RM) -f tests.error
	$(RM) -f tests.log

clean-coverage :
	$(RM) -r coverage

clean-build :
	$(RM) -r target

src/rusty_monitor.rs :
	$(CP) $(MONITOR) ./src/rusty_monitor.rs

analysis/hir/hir.json : src/rusty_monitor.rs
	@export RUST_LOG=debug && \
	export RUST_BACKTRACE=full && \
	export RU_CRATE_NAME=$(CRATE_NAME) && \
	export RU_CRATE_ROOT=$(CRATE_ROOT) && \
	export RU_RUN=$(RUN) && \
	export RUSTC_WRAPPER=$(ANALYZER) && \
	export RUST_BACKTRACE=1 && \
	cargo build --features $(FEATURES)

coverage/rusty_tests.profdata :
	@export RUSTFLAGS="-C instrument-coverage -Awarnings" && \
	export LLVM_PROFILE_FILE="coverage/rusty-test-%m.profraw" && \
	cargo test rusty_tests --features $(FEATURES) && \
	cargo profdata -- merge -sparse "coverage/"rusty-test*.profraw -o "coverage/rusty_tests.profdata"

coverage/data.json : coverage/rusty_tests.profdata
	mkdir -p coverage
	$(shell ru export-cov "coverage/rusty_tests.profdata" "coverage/data.json" $(FEATURES))

coverage/report.html : coverage/rusty_tests.profdata
	mkdir -p coverage
	$(shell ru show-cov "coverage/rusty_tests.profdata" "coverage/report.html" $(FEATURES))

coverage : coverage/data.json

coverage-html : coverage/report.html

analyze : analysis/hir/hir.json

random-search : analysis/hir/hir.json
	java -jar $(JAR) --algo random \
	-c $(CRATE_ROOT) \
	-m $(MAIN_FILE) \
	-n $(CRATE_NAME) \
	--o $(OUTPUT)-$(RUN) \
	--run $(RUN) \
	--hir $(CRATE_ROOT)/analysis/hir/hir.json \
	--mir $(CRATE_ROOT)/analysis/mir/original \
	--instrumenter $(INSTRUMENTER) \
	--features $(FEATURES) \
	--parse-traits

seeded-random-search: analysis/hir/hir.json
	java -jar $(JAR) --algo random \
	-c $(CRATE_ROOT) \
	-m $(MAIN_FILE) \
	-n $(CRATE_NAME) \
	--o $(OUTPUT)-$(RUN) \
	--run $(RUN) \
	--hir $(CRATE_ROOT)/analysis/hir/hir.json \
	--mir $(CRATE_ROOT)/analysis/mir/original \
	--instrumenter $(INSTRUMENTER) \
	--features $(FEATURES) \
	--seed-all-methods \
	--seed-random-population \
	--seed-constant-pool \
	--parse-traits

seeded-dynamosa : analysis/hir/hir.json
	java -jar $(JAR) --algo dynamosa \
	-c $(CRATE_ROOT) \
	-m $(MAIN_FILE) \
	-n $(CRATE_NAME) \
	--o $(OUTPUT)-$(RUN) \
	--run $(RUN) \
	--hir $(CRATE_ROOT)/analysis/hir/hir.json \
	--mir $(CRATE_ROOT)/analysis/mir/original \
	--instrumenter $(INSTRUMENTER) \
	--features $(FEATURES) \
	--seed-all-methods \
	--seed-random-population \
	--seed-constant-pool \
	--parse-traits

dynamosa : analysis/hir/hir.json
	java -jar $(JAR) --algo dynamosa \
	-c $(CRATE_ROOT) \
	-m $(MAIN_FILE) \
	-n $(CRATE_NAME) \
	--o $(OUTPUT)-$(RUN) \
	--run $(RUN) \
	--hir $(CRATE_ROOT)/analysis/hir/hir.json \
	--mir $(CRATE_ROOT)/analysis/mir/original \
	--instrumenter $(INSTRUMENTER) \
	--features $(FEATURES) \
	--parse-traits

execute :
	@export RUSTC_WRAPPER=$(INSTRUMENTER) && \
	export RU_STAGE=instrumentation && \
	export RU_CRATE_NAME=$(CRATE_NAME) && \
	export RU_CRATE_ROOT=$(CRATE_ROOT) && \
	export RU_RUN=$(RUN) && \
	cargo test rusty_tests
