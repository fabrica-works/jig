# skein-ui: the code audit at the base

2026-10-15, jig `5efac1af8`, skein `2f282f7`. Read-only audits of the
parts the plan touches, against skein's `ui.md`, `ui-text.md` and
`ui-web.md`. Paths are jig's unless they say skein's; lines are at the
base. Counts are `rg -l <name> --type rust` over the repository.

## 1. jig-view's splits (jig 02)

- **`binding.rs`.** Generic: `Binding` 7-12, `Act` 14-27, `BoundAct`
  29-34, `number` 36-51; `Binding.owner` is jig's `Owner`
  (`tree.rs:15-20`). jig's: `JigMeaning` 53-87 (17 files), `Entry`
  88-92, `JigBindings` 93-229 (95 files), `decode` 230-317 (19 files in
  jig-view, and `examples/ops/view/src/root.rs:261`,
  `testing/jig-test-view/src/root.rs:193`), `jig_bindings_worst_case`
  319-326 (8 files).
- **`limits.rs`.** Generic: `Limits` 11-26, `worst_case` 28-59 but line
  58, which adds `crate::pages::scratch` (`pages.rs:200-205`, jig's page
  scratch, 1 file). jig's: `impl Limits` 61-153 (`of` 6 files,
  `of_words` 20, `compose` 4, `with_phrase_bytes` 1, `envelope` 2).
  Fields against ui.md 15: all present but `scene_marks` and
  `scene_bytes`; none extra. `worst_case` counts no scene.
- **`outline.rs`.** `Fault` 5-10 generic; `Rule` 12-29, `check` 31-54,
  `landmarks` 56-130. Against ui.md 14's six rules: the root's role is
  not checked; "each child of the page a landmark or a dialog" only as
  `InsideLandmark` (24, 118), which accepts main, navigation,
  complementary and content info (100-104) and jig's parts, and refuses
  form, search, a named region and a dialog; `OneMain` (20); at most one
  content info (`OneContentInfo`, 22), a second banner falling into
  `InsideLandmark`; `ShortcutOnce` (28); Escape unchecked (no Escape in
  `Shortcut`, `tree.rs:165-175`). jig's own: `BannerFirst` (16, 57-62),
  `BannerHolds { JigRegion }` (18, 63-74), `AfterPageLast` (26, 87-99,
  116). Their tests: `tests/outline.rs` 35-41, 42-55, 76-86, and every
  test there through `banner()` 16-24; `tests/client/tests/referee.rs`
  447-473 uses `BannerFirst` as its sample fault. No test covers
  `OneContentInfo`.
- **`phrases.rs`.** Generic: `PhraseId` 15-20 (its owner jig's
  `Owner`), `HoleKind` 21-36, `Hole` 37-64, the shapes, rules, forms,
  `Phrase`, `Language`, `PhraseLimits`, `Invalid`, `Capacity` 65-194,
  `Phrasebook` 195-204 but 199, `Saying` 212-217, `load` 220-328 but
  325-326, `say` 329-426, helpers 436-574, `worst_case` 576-597. jig's:
  `pub mod english; pub mod jig;` 8-9; `jig_capacity` field 199, set
  325-326, `jig_capacity()` 342-347 (19 files); `normal_capacity`
  429-435, which reads `jig::ALL` and `crate::owner_words::phrase_bytes`.
  The chip hole is task-shaped: `HoleKind::Chip` 34-35, `Hole::Chip {
  task: u64, title }` 50-51, `ChipSpan { start, end, task }` 205-211,
  `Saying.chips` 216; handled at 61, 363, 376, 414-420, 503, 530, 570,
  591; `ChipSpan` in 2 files, `Hole::Chip|HoleKind::Chip` in 3
  (`phrases.rs`, `tests/phrases.rs`, `tests/owner_history.rs:547`). No
  jig phrase declares a chip hole. `phrases/` is jig's tables:
  `JigPhrase` (69 files), `jig::ALL` (627 phrases), `english.rs`.

## 2. The text shell and the fake person (jig 03)

- **`jig-shell-text`.** `render.rs:2, 55` choose the first line's status
  node by `JigRegion::Link.region()` (`header`, 43-84); `render(tree,
  name, print)` at 86 takes no region. `src/tests.rs:43` also names
  `JigRegion::Link`. `command.rs:47` calls `jig_view::number`.
  `Owner::Jig` at `src/tests/fixtures.rs:17, 176, 181, 198`. `Settings`
  (`terminal.rs:14-20`): `name`, `longest`, `interval`; no status region.
  **No golden fixture tags a region:** both golden first lines are `==
  TITLE == ` with its trailing space (`tests/golden/roles.txt:1`,
  `bytes.txt:1`), so a status region set to none keeps them byte for
  byte. `roles.txt:74` prints `(provisional)` (`render.rs:306, 323`).
  Present against ui-text.md: `[-]`, `(enter sends)`, the six refusals
  with `no page yet`, `closed`, `\r` stripped. Absent: select, option,
  column and row headers, `(required)`, entry kinds, a secret's
  `(hidden)`, descriptions, `(P of C)`, sort.
- **`jig-fake-person`.** `Owner::Jig` at `src/tests.rs:52`,
  `Owner::Application { number: 1 }` at 87. Present: find by role and
  words, exact name, text, within ancestors (`lib.rs:40-55`,
  `tree.rs:36-93`); the eight steps (`lib.rs:76-90`); `Do`, `Wait`,
  `Done`, `Failed` (104-135); unbound and disabled refused
  (`tree.rs:105`); the random person (`random.rs`). Absent: acts checked
  by role (`tree.rs:109, 112`); a press on a check or a choice as
  `Toggle` (`lib.rs:202` always `Press`); options; the random person
  skipping drawings (`random.rs:73-78`); a table test of role names.
- **Users.** `jig-shell-text`: `tests/client` (12 lines),
  `tests/client-protocol` (11). `jig-fake-person`: `tests/client` (25
  lines in 21 files), `tests/client-protocol` (12), `tests/ops` (4).

## 3. skein at the base (skein 01 onwards)

- **Workspace** (`Cargo.toml`): `crates/*` a glob; `testing/*`,
  `tests/*`, `examples/echo/*` listed one by one (5-46), so each new
  crate there adds its line. `skein-codec` and `skein-codegen` have no
  `[workspace.dependencies]` entry. No `wasm-bindgen`, `js-sys`,
  `web-sys` or `wasm-bindgen-cli-support` anywhere; no `web` profile; no
  optimized tool package.
- **`scripts/check.sh`:** fmt (8), clippy (9), `taste.sh` (10), `cargo
  check -p skein-tls` (13), nextest default (16), fuzzy (17), browser
  (18). **No wasm32 build today,** though `testing.md`, 6 says lib,
  codec and HTTP build there (findings/toolchain.md, 2).
- **`.config/nextest.toml`:** default 15 s (slow 3 s), filter `not
  binary(/^fuzzy_/) and not binary(/^browser_/) and not binary(=live)`;
  fuzzy 60 s; browser `package(skein-browser-world) and
  binary(/^browser_/)`, 2 min, 1 thread; measure; live. The browser
  filter does not take `tests/ui-web`'s binaries.
- **`skein-browser`:** a step machine; `Request::{Find, Await, Press,
  Type, Compose, Key, Snapshot, Screenshot}` (`boundary.rs:33-40`).
  **Find matches exactly only** (`Query { role, name, within, boxes }`,
  84-89); **press has no point** (`boundary.rs:35`, the box's centre at
  `browser.rs:1582-1587`). browser.md asks `Press { .., at:
  Option<Point> }` (153), `Query { .., matching: Match }`, `Match {
  Exact, Words }` (176-178), rules 190-207, 221-240, and real-Chromium
  checks (301-311); `browser_real.rs`'s `Scenario` (128-135) has
  neither.
- **HTTP and `drive`:** `skein_http::server` (`Server::new` 441, `up`
  484, `down` 499), the event-stream writer (`sse/writer.rs`,
  `Writer::new` 180); `skein_shell::{Host, drive}` (`drive.rs:12, 53`).
  The echo's `main.rs` and `examples/echo/shell/src/lib.rs:26-79` are
  the binary to copy; its `drain` prints `skein-echo: listening at
  {addr}` once. `tests/browser/src/serve.rs` serves pages with a policy
  header (176).
- **Chromium in tests:** `browser_real.rs:49-94`: `SKEIN_TEST_CHROMIUM`
  or `chromium` on `PATH`, spawned by
  `skein_browser::command::command` (`command.rs:43`) in a hand-rolled
  loop; nothing shared in a library.
