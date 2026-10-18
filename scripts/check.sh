#!/bin/sh
# The gate (docs/development/workflow.md, section 1): formatting, lints as
# errors, then both test suites in full: the default one, then the fuzzy
# one. skein's `land` runs it through the machine's queue.
set -eu

cd "$(dirname "$0")/.."

cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
# The browser's step crates (protocol/browser.md, 15).
cargo clippy --target wasm32-unknown-unknown \
    -p jig-client-protocol-wire -p jig-client-protocol-link \
    -p jig-client-protocol-http -p jig-client-domain -p jig-client \
    -p jig-client-view -p jig-client-shell-web -p jig-client-test-protocol-wire \
    -p jig-client-test-domain -p jig-client-test-view \
    -p jig-ops-client-protocol-wire -p jig-ops-client-domain \
    -p jig-ops-client-view -- -D warnings
# skein's testing-strategy.md, section 8: a test is a process.
export NEXTEST_DOUBLE_SPAWN=0
cargo nextest run --workspace
cargo nextest run --workspace --profile fuzzy
