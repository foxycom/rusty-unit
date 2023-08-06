SHELL:=/bin/bash

bin/analysis :
	@export RUSTFLAGS="-A dead_code -A unused_variables -A unused_imports -A unused_mut" && \
	cd compiler && \
	cargo build \
	--release \
	--bin analysis  \
	--features "analysis instrumentation file_writer" && \
	cd .. && \
	mkdir -p ./bin && \
	mv -v ./compiler/target/release/analysis ./bin/analysis

bin/instrumentation :
	@export RUSTFLAGS="-A dead_code -A unused_variables -A unused_imports -A unused_mut" && \
	cd compiler && \
	cargo build \
	--release \
	--bin instrumentation  \
	--features "instrumentation" && \
	cd .. && \
	mkdir -p ./bin && \
	mv -v ./compiler/target/release/instrumentation ./bin/instrumentation

bin/rusty-unit.jar :
	pushd rusty-unit && \
	gradle shadowJar && \
	popd && \
	mv -v ./rusty-unit/build/libs/rusty-unit.jar ./bin/rusty-unit.jar

build : bin/analysis bin/instrumentation bin/rusty-unit.jar
