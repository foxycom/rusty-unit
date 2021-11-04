#!/usr/bin/env bash


cargo +nightly-aarch64-apple-darwin build

pushd "/Users/tim/Documents/master-thesis/testify/benchmarks" && rm -rf target
rm -rf "/Users/tim/Documents/master-thesis/testify/results/instrumentation.log"
#RUSTFLAGS="--testify-stage=analyze" RUSTC_WRAPPER=/Users/tim/Documents/master-thesis/testify/target/debug/instrumentation cargo +nightly-aarch64-apple-darwin build
RUSTC_WRAPPER=/Users/tim/Documents/master-thesis/testify/target/debug/instrumentation RUST_BACKTRACE=full cargo +nightly-aarch64-apple-darwin rustc -- -Z verbose -Awarnings --testify-stage=analyze
popd