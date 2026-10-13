# Session 01: the gate, and what every world pays

Three costs every jig test or world pays, whatever it checks: nextest's
double spawn, about half of each test's process (3,813 processes in the
focused suite); a configuration document's overrides, each of which
recomputes the whole domain's worst case on the same limits (36 times
for the channel world's engine, about 4 ms a document); and the
reference service parsing its document two or three times at each start.
Two increments, neither needing anything outside jig.

## Read first

- skein's `testing-strategy.md`, section 8 ("A test is a process", as
  this plan's design landing revised it: "The gate, and every command the
  machine's queue runs, starts nextest without its double spawn").
- jig's `docs/development/workflow.md`, sections 1 and 2.
- `findings/cost-jig-units.md`, 1 (the double spawn, measured) and 3
  (the overrides); `findings/cost-jig-channel.md`, 3 and 5;
  `findings/tried/jig-overrides-once.md`.

## What the design asks

- "The gate, and every command the machine's queue runs, starts nextest
  without its double spawn (`NEXTEST_DOUBLE_SPAWN=0`, set by each
  repository's `scripts/check.sh` and by `heavy`)" (testing-strategy.md,
  8).
- "**Fixtures once.** Build a phrasebook, configuration or worst-case
  size once per test process or world when it is a fixture, then share
  it" (workflow.md, 2).

## Choices this plan makes

- **The overrides' check computes each family's worst case at most once
  per document:** `members` (`crates/jig-protocol-config/src/overrides.rs:515-563`)
  carries a `Fits { hub: Option<bool>, core: Option<bool> }` that `valid`
  (`:565-577`) fills on first need; the problems pushed, and their order,
  stay the same.
- **The reference service parses its document once at startup:**
  `startup` (`testing/jig-test-service/src/startup.rs:533-545`) calls
  `config::document` once and hands the parsed document to
  `worker_specs_in` and `parse_in`, new functions holding the bodies of
  `worker_specs` (`:432`, which also parses at `:448`) and `parse`
  (`:104`). `parse(input: &[u8])` stays, a wrapper that parses and calls
  `parse_in`, for its other callers (`startup.rs:261,624`;
  `src/tests/fixtures.rs:164,642`; `src/tests/first_period.rs:11`).

## Rules that bind it

- No increment "drops an assertion, or changes an expected value, except
  a size its session file names with the reason the assertion holds at
  it" (code-organization.md, section 4).
- "`NEXTEST_DOUBLE_SPAWN=0` is set in each repository's
  `scripts/check.sh` and in skein's `heavy`, and nowhere else in code"
  (code-organization.md, section 1).
- Shared files: jig 04.2 folds the cases of
  `crates/jig-protocol-config/src/tests/overrides.rs` after 01.2 adds
  its test there; jig 02.1 edits `testing/jig-test-service` too, but not
  `startup.rs`.

## Increments

### 01.1 nextest without its double spawn in jig's gate
Scope: testing-strategy.md, 8, as revised.
Needs: nothing.
Files: `scripts/check.sh`; `docs/development/workflow.md`, section 2
  (the gate's command and the `measure` command each named with the
  variable).
Shape: `export NEXTEST_DOUBLE_SPAWN=0` before the first `cargo nextest`
  in `scripts/check.sh`, with a comment naming testing-strategy.md, 8.
Test shape: none.
Proof: process, the focused time in `land`'s `Gate-Result`, against
  15.041 s (jig `2048f39b7`'s); nothing runs before `land`.
Don't: change `.config/nextest.toml` (jig 06.1's).
Size: at most 5 lines.
Removes: nothing.

### 01.2 A document's worst cases once; the reference service's document parsed once
Scope: workflow.md, 2 ("Fixtures once"); jig's
  `protocol/configuration.md`, unchanged in behaviour.
Needs: nothing.
Files: `crates/jig-protocol-config/src/overrides.rs` (`members`
  `:515-563`, `valid` `:565-577`), `crates/jig-protocol-config/src/tests/overrides.rs`;
  `testing/jig-test-service/src/startup.rs` (`startup` `:533-545`,
  `parse` `:104-112`, `worker_specs` `:432-450`).
Shape:
  ```rust
  // crates/jig-protocol-config/src/overrides.rs
  /// Each family's worst case over the overridden limits, computed once when first asked.
  struct Fits {
      hub: Option<bool>,
      core: Option<bool>,
  }
  fn valid(limits: &Limits, name: &[u8], fits: &mut Fits) -> bool;

  // testing/jig-test-service/src/startup.rs
  pub(super) fn parse(input: &[u8]) -> Result<Parsed, Problems>; // parses, then parse_in
  fn parse_in(document: &config::Document) -> Result<Parsed, Problems>;
  fn worker_specs_in(document: &config::Document) -> Result<(bool, List<WorkerSpec>), Problems>;
  ```
  `startup` maps a document that does not parse to the problem `parse`
  maps it to today (`problem(found.path, found.what)`), and
  `worker_specs_in` takes the engine's listener from `parse_in` on the
  same document.
Test shape: `crates/jig-protocol-config`'s tests unchanged; one test in
  `src/tests/overrides.rs`, `overrides_past_a_familys_bound_are_each_reported_once`:
  a document with three core overrides that together pass the core's
  bound gives three `What::Override` problems, at the three members'
  paths, in their order. The reference service's tests unchanged.
Proof: unit and world, `heavy --gate cargo nextest run -p
  jig-protocol-config -p jig-test-service -p jig-one-process-world
  --profile measure -j 1` once: every test passes; the one-process
  world's summed time in the report (the audit: its power-loss test 0.81
  s to 0.67 s).
Don't: change a problem, its path, or its order.
Size: at most 30 lines of production code.
Removes: the repeated worst cases and the extra parses.

## Done when

- `grep -n NEXTEST_DOUBLE_SPAWN ~/src/rust/jig/scripts/check.sh` prints
  the export.
- `git -C ~/src/rust/jig grep -c 'config::document(' -- testing/jig-test-service/src/startup.rs`
  prints `testing/jig-test-service/src/startup.rs:2` (the wrapper's and
  `startup`'s).
