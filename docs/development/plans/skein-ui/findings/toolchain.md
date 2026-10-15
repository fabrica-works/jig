# skein-ui: the toolchain, checked at the base

2026-10-15, carbon-forge: rustc 1.95.0, cargo-nextest 0.9.138, Chromium
147.0.7727.101 at `/usr/bin/chromium`, target `wasm32-unknown-unknown`
installed.

1. **The glue's versions.** A scratch workspace
   (`~/src/tmp/planning/skein-ui/wasm-probe`) with `wasm-bindgen =
   "=0.2.126"`, `js-sys = "=0.3.103"`, `web-sys = "=0.3.103"` (features
   `Window`, `Document`, `Element`, `HtmlElement`) in a `cdylib` with
   `#![forbid(unsafe_code)]`, a `#[wasm_bindgen(start)]` function and one
   `#[wasm_bindgen(module = "/snippet.js")]` import, built with `heavy
   cargo build -p probe-module --target wasm32-unknown-unknown`; and a
   binary on `wasm-bindgen-cli-support = "=0.2.126"` (resolving `walrus
   0.26.5`) whose `Bindgen::new().input_path(..).web(true)` generated
   `probe_module.js`, `probe_module_bg.wasm` (115 KB, dev) and
   `snippets/probe-module-<hash>/snippet.js`. Both passed, 58 s. The
   newest on crates.io was 0.2.129; the plan pins 0.2.126, which the
   cache holds. `forbid(unsafe_code)` holds over the macros' output.
2. **lib, codec and HTTP for wasm32.** `heavy cargo check --target
   wasm32-unknown-unknown -p skein-lib -p skein-codec -p skein-http`
   passed in a worktree of skein at `2f282f7`; skein's `check.sh` builds
   none of them for wasm32 today (findings/audit.md, 3), which skein
   11.4 adds with skein-ui's.
3. **jig on skein's main, smith unmoved.** In a worktree of jig at
   `5efac1af8`: `heavy sh -c 'cargo update -p skein-lib && cargo check
   --workspace --all-targets'` moved skein `74b7fdaf3` → `2f282f7`, left
   smith at `29fa97c9`, and passed (88 s). skein's only code change in
   between is `testing/skein-fake-llm-domain/src/cache.rs` (smith's
   parity).
