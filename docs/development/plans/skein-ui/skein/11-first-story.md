# Session skein 11: the first story in Chromium

The meeting point of the browser's sides (decisions.md, 7): the harness
runs the example's server and Chromium in one loop; the face finds the
person's targets through skein's browser kit; the reference shell, thin,
starts the example's view over the inbox, the page part, fetch and the
platform's facts; and the first story runs in the browser suite, from
skein's gate: "a person opens the example and sees its heading and its
words for no notes, under the strict policy, with no trouble". Every
later browser session needs this story at its start, and grows it.

## Read first

- skein's `docs/design/ui-web.md`, sections 3.1, 5, 5.1, 5.2, 10, 14,
  14.1 and 14.2; `examples.md`, section 9.4; `testing.md`, sections 6
  and 7; `browser.md`, sections 5, 7 and 10.
- This plan's `decisions.md`, 7; `code-organization.md`, 1, 3, 4 and 5;
  `findings/audit.md`, 3 (Chromium in tests).
- skein's `testing-strategy.md`, sections 2.9 and 8.

## What the design asks

The harness runs the application's server binary and Chromium as
children of one loop through skein's io, opens the address the binary
prints (found by a prefix), runs each person's script through the face,
fails a test on any trouble the kit reports that the story did not
expect, and writes the snapshot, a screenshot and both stderr tails on
a failure (ui-web.md, 14.2). The face finds by role through the
mapping, names exactly or by words, text by words within one run, and
answers `Pending` while the kit answers (14.2). The reference shell is
the application's start and its turn, about 200 lines, commented to be
copied (3.1, 5.2). The gate builds the wasm32 targets with clippy, the
example's bundle from a dev-profile module, and runs the browser suite
over it, `SKEIN_UI_EXAMPLE_BUNDLE` naming the bundle (testing.md, 6;
ui-web.md, 14.1).

## Choices this plan makes

- **The server binary** is found beside the test binary's directory:
  `tests/ui-web/src/lib.rs::server_binary()` is `current_exe()`'s
  grandparent joined with `skein-ui-example`, which `cargo build -p
  skein-ui-example-server` makes (in `scripts/check.sh` from 11.4, and
  in each browser Proof's command); missing, it fails naming that
  command.
- **The gate's bundle** is written to `target/ui-example-bundle/` by
  `scripts/check.sh`, from `target/wasm32-unknown-unknown/debug/
  skein_ui_example_shell_web.wasm`.
- **The parts this session adds** to `skein-ui-web`: `fetch`
  (exchanges only; the stream is skein 12.4's) and `platform` (the
  facts).

## Rules that bind it

- **The browser suite is a suite of its own:** each test in
  `tests/ui-web/tests/browser_*.rs`, run by `--profile browser` alone;
  a missing Chromium or bundle fails, naming what provides it; never a
  skip.
- **The story fails on any trouble it did not expect,** a policy
  violation among it (ui-web.md, 14.1): `Setup.expected` is empty in
  every story of this plan but skein 14's panic story.
- **Test time,** aims (README, "Test time"): the example's tier aims
  at one minute of the browser suite's two (ui-web.md, 14.1); a browser
  test at 3 s.

## Increments

### 11.1 The face
Scope: ui-web.md 14.2 (the face, its acts).
Needs: skein 07.2 (`Match::Words`, 07.1, and `Press { at }`); skein 09.3
  (`mapping::accessible`).
Files: new `testing/skein-fake-person-browser/{Cargo.toml, src/lib.rs,
  src/face.rs, src/tests.rs}`; skein's `Cargo.toml` (member,
  `[workspace.dependencies]`).
Shape: `BrowserFace` implements `skein_fake_person::Face`: a `Find`
  becomes one kit `Find` per role it may compute (`accessible`), scoped
  `within` the nodes its ancestors found, outermost first, `Exact` or
  `Words` as the target says; text by `Words` on `StaticText`; it
  answers `Pending` until the kit's terminal event, then the first
  match; `allows` refuses a node the kit reports disabled. Its acts as
  kit requests: `Press` a press at the centre; `Input` a select-all and
  the words by `Type`; `Submit` the key Enter; `Go` the path the codec
  prints; a reload `Go::Reload`.
Test shape: `src/tests.rs`, the kit's events scripted as literals:
  `a_find_asks_each_accessible_role_of_the_person_role`,
  `the_face_is_pending_until_the_kit_answers`,
  `text_is_found_by_words_within_one_run`,
  `a_press_becomes_the_kits_press_at_the_centre`. Each under 20 ms.
Proof: unit, `heavy cargo nextest run -p skein-fake-person-browser`
  once; 11.4's story runs it against Chromium.
Don't: send script to the page (browser.md, 4).
Size: about 300 lines.
Removes: nothing.

### 11.2 The harness
Scope: ui-web.md 14.2 (the harness, but stopping and restarting the
  server, skein 12.3's, and pressing a mark, skein 13.4's).
Needs: skein 10.2 (the server binary); 11.1.
Files: `testing/skein-fake-person-browser/src/{harness.rs, tests.rs}`;
  `tests/ui-web/src/lib.rs` (`server_binary`, a fixture bundle);
  new `tests/ui-web/tests/browser_harness.rs`;
  `.config/nextest.toml` (the browser profile's filter takes
  `package(skein-ui-web-tests) and binary(/^browser_/)` beside the
  kit's). The loop's pattern: `tests/browser/tests/browser_real.rs:49-94,
  181-583` (`chromium()`, `spawn`, the `Rig`).
Shape: as the sketch. `Harness::run` starts both children, reads the
  server's stdout to the line starting with `setup.listening`, opens a
  page per player at the address after it, steps each `Play` through
  its `BrowserFace` until each is done or failed, fails on any trouble
  not matching one of `setup.expected` (words in the trouble's text),
  and on a failure puts the snapshot, a screenshot's path and both
  stderr tails into `Outcome::Failed`'s report. `Script` plays a
  person's `Step`s and the harness's own (`harness::Step`), each
  harness step added by the increment that first needs it: none here.
Test shape: `tests/ui-web/tests/browser_harness.rs`:
  `the_harness_fails_a_page_whose_module_throws` (the fixture bundle,
  its glue a literal module that throws at `init`: the outcome names
  the exception); `the_harness_reads_the_listening_line`. About 1 s
  each. Step test in `src/tests.rs`: `the_listening_line_gives_the_address`.
Proof: live (a local Chromium and the real server): `heavy sh -c 'cargo
  build -p skein-ui-example-server && cargo nextest run -p
  skein-ui-web-tests --profile browser'` once. Bound: 2 minutes.
Don't: copy the `Rig` into the harness whole: take what the harness
  needs, about 300 lines.
Size: about 350 lines.
Removes: nothing.

### 11.3 The reference shell, thin, with fetch and the platform's facts
Scope: ui-web.md 3.1, 5 (starting), 5.1 (fetch's exchanges, the
  platform's facts), 5.2 (the turn, but what skein 12 and 13 add);
  examples.md 9.4 (the reference shell).
Needs: skein 09.5 (the page part's platform half); skein 10.1.
Files: new `crates/skein-ui-web/src/{fetch.rs, fetch/web.rs,
  platform.rs, platform/web.rs}`; new `examples/ui/shell-web/{Cargo.toml
  (cdylib, wasm32 only, its code under `#![cfg(target_arch =
  "wasm32")]`), src/lib.rs}`; `examples/ui/view/src/lib.rs` (the
  answered list as a report); skein's `Cargo.toml` (member).
Shape: `fetch::Fetch` (pure: the exchanges in flight by id, each
  answer's body bound; platform: `window.fetch` with an
  `AbortController`, same-origin credentials); `platform::facts() ->
  Facts { now, wall, seed, offset, languages }`; the shell's
  `#[wasm_bindgen(start)]`: finds `#app`, makes the inbox, the page part
  with the look, fetch; starts the state from the facts and the path;
  its turn as ui-web.md 5.2's listing, but history, the small store,
  the timer, title and closing (skein 12, 13); commented to be copied.
Test shape: `crates/skein-ui-web/src/tests/fetch.rs`:
  `an_answer_past_its_bound_fails_the_exchange`,
  `an_abandoned_exchange_reports_nothing`. Each under 20 ms. The shell
  has no host test: it is built for wasm32 only.
Proof: the wasm32 build with the lints: `heavy cargo clippy -p
  skein-ui-example-shell-web -p skein-ui-web --target
  wasm32-unknown-unknown -- -D warnings` once; 11.4 runs it.
Don't: write `unsafe`; inline a script or a style.
Size: about 350 lines (the shell about 150 of them).
Removes: nothing.

### 11.4 The first story, from the gate
Scope: ui-web.md 14.1, 14.3 (its first story); testing.md 6 (the gate).
Needs: skein 08.5 (the bundler's binary); 11.1 to 11.3.
Files: new `tests/ui-web/tests/browser_example.rs`; `scripts/check.sh`;
  `tests/ui-web/src/lib.rs` (`example_bundle()` reads
  `SKEIN_UI_EXAMPLE_BUNDLE`, failing naming `scripts/check.sh` when
  unset or missing; `example_harness() -> Harness`, the example's
  `Setup` with the bundle and `listening: b"skein-ui-example: listening
  at "`; `const SECOND: Duration`).
Shape: `scripts/check.sh` gains, after the host clippy: `cargo check
  --target wasm32-unknown-unknown -p skein-lib -p skein-codec -p
  skein-http`; `cargo clippy --target wasm32-unknown-unknown -p skein-ui
  -p skein-ui-web -p skein-ui-example-view -p skein-ui-example-shell-web
  -- -D warnings`; and before the browser suite: `cargo build --target
  wasm32-unknown-unknown -p skein-ui-example-shell-web`, `cargo run -p
  skein-ui-bundle -- --module … --name skein_ui_example_shell_web
  --prefix /bundle/ --out target/ui-example-bundle --noscript '…'`,
  `cargo build -p skein-ui-example-server`, and `export
  SKEIN_UI_EXAMPLE_BUNDLE`.
Test shape: `browser_example.rs`: `a_person_opens_the_example_and_sees_its_heading`.
  Reference test:
  ```rust
  #[test]
  fn a_person_opens_the_example_and_sees_its_heading() {
      let mut person = Script::new(Box::new([
          harness::Step::Person(Step::See { find: Find::exactly_named(Role::Heading { level: 1 }, b"Notes"), within: SECOND }),
          harness::Step::Person(Step::See { find: Find::text(b"No notes yet."), within: SECOND }),
      ]));
      let outcome = example_harness().run(&mut [&mut person]);
      assert_eq!(outcome, Outcome::Done);
  }
  ```
  About 2 s.
Proof: live, the gate: `heavy --gate scripts/check.sh` in the
  increment's committed worktree, once: the wasm32 builds, the bundle
  and the browser suite with the story pass. Bound: 15 minutes.
Don't: skip the story without a bundle; build the gate's module with
  any profile but dev (not `--profile web`).
Size: about 40 lines of script, 60 of tests.
Removes: nothing.

## Interface sketch

```rust
// testing/skein-fake-person-browser/src/harness.rs: what an application's
// browser tier calls (ui-web.md, 14.2). The test owns the harness; it
// ends when every person is done or one failed.
pub struct Setup<'a> {
    pub server: PathBuf,
    pub args: Vec<OsString>,
    /// The prefix of the server's line that names its address.
    pub listening: &'a [u8],
    pub chromium: PathBuf,
    /// Words of the troubles a story expects; any other fails it.
    pub expected: Vec<Box<[u8]>>,
}
/// One person's script, played through the face.
pub trait Play {
    fn next(&mut self, face: &mut BrowserFace<'_>, now: Time) -> Next<Box<[u8]>>;
}
/// A person's steps, or the harness's own (ui-web.md, 14.2), which later
/// increments add: `Key` (13.3), `PressAt` (13.4), `Compose`, `Back`,
/// `Forward` (14.4).
pub enum Step { Person(skein_fake_person::Step<Box<[u8]>>) }
/// A script of steps; it implements `Play`.
pub struct Script { /* the steps, the person */ }
impl Script { pub fn new(steps: Box<[Step]>) -> Script; }
#[derive(PartialEq, Eq, Debug)]
pub enum Outcome {
    Done,
    /// The step, why, and the failure's report: snapshot, screenshot's
    /// path, both stderr tails.
    Failed { person: u32, step: u32, report: String },
}
pub struct Harness { /* the loop, both children, the kit */ }
impl Harness {
    pub fn new(setup: Setup<'_>) -> Harness;
    pub fn run(&mut self, people: &mut [&mut dyn Play]) -> Outcome;
}
// harness.rs grows `stop_server` and `start_server` in skein 12.3.
```

## Questions to raise when reached

None known.

## Done when

`heavy --gate scripts/check.sh` passes in a worktree of skein at
`origin/main` once, and `git grep -n 'wasm32-unknown-unknown\|profile
browser' origin/main -- scripts/check.sh` prints both.
