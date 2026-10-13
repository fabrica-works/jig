#!/usr/bin/env bash
set -euo pipefail
cargo build -p jig-test --bin jig-test
cargo nextest run -p jig-test --profile process
