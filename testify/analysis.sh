#!/usr/bin/env bash

#rm -rf /Users/tim/Documents/master-thesis/testify/results/hir.json
#rm -rf /Users/tim/Documents/master-thesis/testify/results/mir.log

RUSTFLAGS="$RUSTFLAGS -A dead_code -A unused_variables -A unused_imports -A unused_mut" cargo +nightly-aarch64-apple-darwin build --bin analysis --features "analysis file_writer" &&
pushd "$RU_CRATE_ROOT" &&
rm -rf target &&
rm -rf "/Users/tim/Documents/master-thesis/testify/results/instrumentation.log" &&
RUSTC_WRAPPER=/Users/tim/Documents/master-thesis/testify/target/debug/analysis RUST_BACKTRACE=1 cargo +nightly-aarch64-apple-darwin build --all-features   &&
(popd || exit)