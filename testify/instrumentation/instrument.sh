#!/usr/bin/env bash

#rm -rf /Users/tim/Documents/master-thesis/testify/results/hir.json
#rm -rf /Users/tim/Documents/master-thesis/testify/results/mir.log

RUSTFLAGS="$RUSTFLAGS -A dead_code -A unused_variables -A unused_imports -A unused_mut" cargo +nightly-aarch64-apple-darwin build &&
pushd "/Users/tim/Documents/master-thesis/evaluation/current" && rm -rf target &&
rm -rf "/Users/tim/Documents/master-thesis/testify/results/instrumentation.log" &&
RUSTC_WRAPPER=/Users/tim/Documents/master-thesis/testify/target/debug/instrumentation RUST_BACKTRACE=full cargo +nightly-aarch64-apple-darwin build --all-features &&

#RUSTFLAGS="--testify-stage=analyze" RUSTC_WRAPPER=/Users/tim/Documents/master-thesis/testify/target/debug/instrumentation cargo +nightly-aarch64-apple-darwin build

popd