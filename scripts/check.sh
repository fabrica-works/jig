#!/bin/sh
# The gate (docs/development/workflow.md, section 1): formatting, lints as
# errors, then three test suites in full: the default one, then the fuzzy
# one, then the browser tier. agents' `land` runs it through the machine's queue.
set -eu

cd "$(dirname "$0")/.."

cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
# The browser's step crates (protocol/browser.md, 15).
cargo clippy --target wasm32-unknown-unknown \
    -p jig-client-protocol-wire -p jig-client-protocol-link \
    -p jig-client-protocol-http -p jig-client-domain -p jig-client \
    -p jig-client-view -p jig-client-shell-web -p jig-client-test-protocol-wire \
    -p jig-client-test-domain -p jig-client-test-view -p jig-client-test-shell-web \
    -p jig-ops-client-protocol-wire -p jig-ops-client-domain \
    -p jig-ops-client-view -- -D warnings
# skein's testing-strategy.md, section 8: a test is a process.
export NEXTEST_DOUBLE_SPAWN=0
cargo nextest run --workspace
cargo nextest run --workspace --profile fuzzy

# The browser tier (protocol/browser.md, 15).
cargo build --target wasm32-unknown-unknown -p jig-client-test-shell-web
cargo build -p skein-ui-bundle
target/debug/skein-ui-bundle \
    --module target/wasm32-unknown-unknown/debug/jig_client_test_shell_web.wasm \
    --name jig_client_test_shell_web --prefix /bundle/ \
    --stylesheet crates/jig-client-shell-web/jig.css \
    --stylesheet testing/jig-client-test-shell-web/jig_client_test_shell_web.css \
    --pictures jig=crates/jig-client-shell-web/pictures \
    --pictures jig-test=testing/jig-client-test-shell-web/pictures \
    --files testing/jig-client-test-shell-web/files \
    --noscript 'jig needs WebAssembly and scripts.' \
    --out target/jig-test-bundle
cargo build -p jig-test
JIG_TEST_BUNDLE="$(pwd)/target/jig-test-bundle"
export JIG_TEST_BUNDLE
cargo nextest run --workspace --profile browser
