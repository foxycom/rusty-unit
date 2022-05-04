#!/usr/bin/env bash

PROFDATA_FILE="$1"

cargo +nightly-aarch64-apple-darwin cov -- export \
    $( \
      for file in \
        $( \
          RUSTFLAGS="-Z instrument-coverage" \
            cargo +nightly-aarch64-apple-darwin test rusty_tests --no-run --message-format=json \
              | jq -r "select(.profile.test == true) | .filenames[]" \
              | grep -v dSYM - \
        ); \
      do \
        printf "%s %s " -object $file; \
      done \
    ) \
  --instr-profile="$PROFDATA_FILE" --summary-only \
  --ignore-filename-regex='/.cargo/registry' \
  --ignore-filename-regex='rusty_monitor.rs' \
  --ignore-filename-regex='/rustc' \
  --Xdemangler=rustfilt