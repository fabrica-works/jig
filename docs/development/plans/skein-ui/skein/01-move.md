# Session skein 01: skein-ui, skein-ui-text and skein-fake-person, moved

jig's `jig_view::ui`, `jig-shell-text` and `jig-fake-person` arrive in
skein as `skein-ui`, `skein-ui-text` and `skein-fake-person`, by a copy
and a rename of paths, with their tests. Nothing changes behaviour.
skein lands first; jig takes the crates in jig 04.

## Read first

- skein's `docs/design/ui.md`, section 2; `ui-text.md`, section 2;
  `testing.md`, section 6 (the layout).
- skein's `AGENTS.md` (the code's rules).
- This plan's `decisions.md`, 2 and 5; `code-organization.md`, 1, 5
  and 6.
- skein's `programming-model.md`, section 10.

## What the design asks

The crates of ui.md 2, at their paths: `crates/skein-ui` (step code,
lib only), `crates/skein-ui-text` (lib, io, ui),
`testing/skein-fake-person` (lib, ui).

## Choices this plan makes

- Each crate is copied from jig's `origin/main` at its increment's
  start (the source commit named in each commit's body), and its paths
  renamed: `crate::ui::` → `crate::`, `jig_view::ui::` → `skein_ui::`,
  `jig_shell_text` → `skein_ui_text`, `jig_fake_person` →
  `skein_fake_person`. Doc comments citing jig's documents cite skein's
  (`ui.md`, `ui-text.md`): at jig `f15431139`, `ui/tree.rs:146` (region
  names) cites ui.md 8, `:152` (pictures) ui.md 3.5, `:158` (keys) ui.md
  6.1; the first lines of jig-shell-text's `lib.rs`, `render.rs` and
  `command.rs` cite ui-text.md 3, 5 and 6. A further citation of a jig
  document: cite skein's section that holds it, and list it in the
  commit body.
- Each increment runs `cargo fmt` after the rename (`skein_ui` sorts
  after `skein_lib`, so `use` lines reorder), and says so in its commit
  body.
- The text shell's golden files move from `tests/golden/` to
  `src/tests/golden/` (ui-text.md, 8).

## Rules that bind it

- **No rewrite of a moved part** (code-organization.md, 6): only paths,
  names and citations change; a test that fails after the copy is a
  question, not a fix.
- **No name of jig in skein's code:** `rg -n -i 'jig' crates/skein-ui
  crates/skein-ui-text testing/skein-fake-person` prints nothing at
  each increment's end.
- **skein's lints hold:** skein's workspace lints may be stricter than
  jig's; a lint the copy trips is fixed in the same increment, its fix
  named in the commit body.

## Increments

### 01.1 `skein-ui`
Scope: ui.md 2.
Needs: jig 02.5 (`jig_view::ui` names nothing outside itself) and jig
  03.1 (the builder's role assertion) on jig's `origin/main`.
Files: new `crates/skein-ui/{Cargo.toml, src/lib.rs, src/*.rs,
  src/tests/*}` from jig's `crates/jig-view/src/ui/`; skein's
  `Cargo.toml` (`[workspace.dependencies] skein-ui`).
Shape: `lib.rs` is jig's `crates/jig-view/src/ui.rs` with `#![cfg_attr(not(test),
  no_std)]`, `#![forbid(unsafe_code)]`, `extern crate alloc;` and the
  crate doc citing `ui.md`; depends on `skein-lib` only.
Test shape: the moved tests; none new.
Proof: unit, and the wasm32 build: `heavy cargo nextest run -p
  skein-ui` and `heavy cargo build -p skein-ui --target
  wasm32-unknown-unknown` pass once (the crate builds for wasm32 as
  lib, codec and HTTP do: findings/toolchain.md, 2). Nothing above
  calls it in skein until skein 09.1 and 10.1; jig 04 connects it.
Don't: add the crate to `scripts/check.sh` (skein 11.4).
Size: about 3,950 lines moved, 30 new.
Removes: nothing in skein.

### 01.2 `skein-ui-text`
Scope: ui-text.md 2 and 3.
Needs: 01.1; jig 03.2 (the status region a setting) on jig's
  `origin/main`.
Files: new `crates/skein-ui-text/` from jig's `crates/jig-shell-text/`,
  its own `Cargo.toml` with `skein-sim` as a dev-dependency, as jig's;
  skein's `Cargo.toml` (`[workspace.dependencies] skein-ui-text`; the
  member comes with `crates/*`).
Shape: as jig's at the copy, its `Settings` with `status`.
Test shape: the moved tests and goldens (`src/tests/golden/roles.txt`,
  `bytes.txt`), byte for byte.
Proof: unit: `heavy cargo nextest run -p skein-ui-text` passes once,
  the goldens unchanged. The terminal's test on skein-sim is the
  highest layer in skein until skein 05.1 moves it to `tests/ui-text`.
Don't: change a golden.
Size: about 1,700 lines moved.
Removes: nothing in skein.

### 01.3 `skein-fake-person`
Scope: ui.md 17.1.
Needs: 01.1; jig 03.3 (the fake person naming only `jig_view::ui`)
  on jig's `origin/main`.
Files: new `testing/skein-fake-person/` from jig's
  `testing/jig-fake-person/`; skein's `Cargo.toml` (member line, and
  `[workspace.dependencies]`); its `[lints]` as jig's crate sets them,
  but for the three `disallowed_{types,methods,macros} = "allow"` lines
  and their comment, which go; a new `testing/skein-fake-person/
  clippy.toml`, a copy of skein's `tests/clippy.toml`, as
  `testing/skein-fake-channel/` has.
Shape: as jig's at the copy.
Test shape: the moved tests.
Proof: unit: `heavy cargo nextest run -p skein-fake-person` passes once.
  Its first caller in skein is the example's view tests (skein 10.3).
Don't: change its behaviour (skein 06).
Size: about 1,255 lines moved.
Removes: nothing in skein.

## Questions to raise when reached

- A skein lint the copied code trips that needs a change of behaviour
  to fix.

## Done when

`heavy cargo nextest run -p skein-ui -p skein-ui-text -p
skein-fake-person` passes, and `rg -n -i 'jig' crates/skein-ui
crates/skein-ui-text testing/skein-fake-person` prints nothing.
