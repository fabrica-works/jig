#!/bin/sh
# The gate (docs/development/workflow.md, section 1): formatting, lints as
# errors, then both test suites in full: the default one, then the fuzzy
# one. skein's `land` runs it through the machine's queue.
set -eu

cd "$(dirname "$0")/.."

cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
# skein's testing-strategy.md, section 8: a test is a process.
export NEXTEST_DOUBLE_SPAWN=0
cargo nextest run --workspace
cargo nextest run --workspace --profile fuzzy
