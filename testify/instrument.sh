#!/usr/bin/env bash

#rm -rf /Users/tim/Documents/master-thesis/testify/results/hir.json
#rm -rf /Users/tim/Documents/master-thesis/testify/results/mir.log

RUSTFLAGS="$RUSTFLAGS -A dead_code -A unused_variables -A unused_imports -A unused_mut" cargo +nightly-aarch64-apple-darwin build --bin instrumentation --features "instrumentation" &&
pushd "$RU_CRATE_ROOT" &&
rm -rf target &&
RUSTC_WRAPPER=/Users/tim/Documents/master-thesis/testify/target/debug/instrumentation RUST_BACKTRACE=1 cargo +nightly-aarch64-apple-darwin build --all-features &&
(popd || exit)