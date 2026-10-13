# Session 04: one behaviour's cases as rows, and the walks over the tree

About 330 of jig's focused tests are cases of one behaviour, each in a
process of its own: refusals per limit, round trips per record, a
contract value per page, a problem per configuration field. Each costs
its process before it checks anything, about 6 ms of the gate's summed
time once jig 01.1 turns the double spawn off, against well under 1 ms of
work. The snapshot world runs each scenario's baseline three times, once
per cut. And jig's boundary tests walk all of jig's 10,126 files, 8,000
of them golden `.bin` records, once per test. Four increments, each
needing nothing outside jig.

## Read first

- skein's `testing-strategy.md`, section 8 ("A test is a process";
  "cases of one behaviour that each cost less than that ... are rows of
  one table test, which checks every row and names each that fails").
- jig's `docs/development/workflow.md`, section 2.
- `findings/cost-jig-units.md`, 4, 6 and 7 (the groups, with
  `file:line`; the snapshot world; the boundary's walks).
- jig `38068705f` (jig-tests.s5): the table helper each crate's tests
  copy, its planted-row check, and what it folded.

## What the design asks

- "So cases of one behaviour that each cost less than that (a golden
  value per type, a refusal per limit, a check per source file) are rows
  of one table test, which checks every row and names each that fails; a
  group of rows takes a test of its own only to share the work across
  threads" (testing-strategy.md, 8).

## Choices this plan makes

- **The table helper is jig-tests.s5's,** copied into each test file
  that needs it as s5 did (`fn table<const N: usize>(cases: [Case;
  N])`, for example `crates/jig-protocol-config/src/tests/overrides.rs:318`):
  every row runs, each failure is caught and named by its row's label,
  and the test fails once at the end naming them all.
- **A row keeps its old test's name as its label,** its inputs and its
  expected value as literals.
- **Only rows of one behaviour fold,** each under 1 ms of work alone; the
  groups below are the audit's. A group the implementer finds is two
  behaviours stays two tests, and its report says so.
- **The generated golden tables are not this session's:** skein 02.2
  makes them 1,024 cases, and jig 05.1 regenerates.
- **The boundary's walk skips `golden` directories,** as it skips
  `target` and hidden ones (`tests/boundary/src/tests.rs:20-22`): they
  hold records, no source, manifest or schema. At the root it skips
  every directory whose name starts with `target`, not only `target`, so
  a second target directory (`target-*`) is not walked.

## Rules that bind it

- "**A test that checks several behaviours under one name**" is on the
  plan's "Don't" list (code-organization.md, section 4).
- No increment "drops an assertion, or changes an expected value, except
  a size its session file names with the reason the assertion holds at
  it" (code-organization.md, section 4).
- "**Cases of one behaviour become rows** ... a planted failing row
  shows, once per crate, that a failure names its row"
  (code-organization.md, section 3).

## Increments

### 04.1 The store's and the wire's cases as rows
Scope: testing-strategy.md, 8.
Needs: nothing.
Files: `crates/jig-protocol-store/src/tests/` (the read-back round
  trips, for example `codec/people_outcomes.rs:6-132`, 17 tests of one
  `people_round_trip`; the refusals per limit, `format.rs:68-125`, 8
  tests of `limits.X = n; check() == Err(...)`); `crates/jig-protocol-client-wire/src/tests.rs`
  (`:23-353`: 12 round trips, 3 bound refusals).
Shape: per behaviour, one `#[test]` named for it (for example
  `people_outcomes_read_back_as_written`), whose `cases` array holds one
  row per old test, labelled with the old test's name.
Test shape: as Shape; one planted failing row per crate, run once, then
  removed (the report names the test and the label the failure
  printed).
Proof: unit, `heavy --gate cargo nextest run -p jig-protocol-store -p
  jig-protocol-client-wire --profile measure` once: the test count and
  summed time in the report, against 502 tests at this plan's base (the
  audit: about 125 of the store's tests become about 15).
Don't: fold tests of different behaviours, or a case past 1 ms of work
  alone; touch generated code.
Size: no production code.
Removes: about 110 test functions.

### 04.2 The link's, the configuration's and the engine's clients' cases as rows
Scope: testing-strategy.md, 8.
Needs: jig 01.2 (it adds a test to `crates/jig-protocol-config/src/tests/overrides.rs`).
Files: `crates/jig-protocol-client-link/src/tests/pages.rs:4-213` (21
  tests of "a record becomes its literal contract value") and
  `tests/documents.rs:24-` (11 of "keeps every supplied field");
  `crates/jig-protocol-config/src/tests/{domain,worker,overrides,llm,charters,hosts}.rs`
  (the `refuses()` and `problem()` cases, 40 or more);
  `crates/jig-protocol-engine-clients/src/tests/documents.rs:5-` (10).
Shape: as 04.1.
Test shape: as 04.1.
Proof: unit, `heavy --gate cargo nextest run -p jig-protocol-client-link
  -p jig-protocol-config -p jig-protocol-engine-clients --profile measure`
  once: the test count and summed time in the report, against 341 tests
  at this plan's base.
Don't: as 04.1.
Size: no production code.
Removes: about 75 test functions.

### 04.3 The snapshot world: one test per scenario, over its three cuts
Scope: testing-strategy.md, 8; jig's conformance world.
Needs: nothing.
Files: `tests/conformance/tests/snapshot.rs` (`compare` `:14-34`, the 24
  tests `:36-155`).
Shape: `compare(scenario, seed)` runs one baseline through
  `replay::chosen_cuts_traced` (`tests/conformance/src/replay.rs:111`)
  with the cuts `[1, 2, 3]`, compares each resumed snapshot with a fresh
  run cut there, as now, its message naming the cut, and asserts that
  the cuts reached are `[1, 2, 3]` and the baseline made at least three
  commits. One test per scenario, named `<scenario>_snapshots_match_complete_fresh_state`,
  with the seed its three tests used.
Test shape: as Shape; `a_forked_snapshot_replays_to_the_durable_rows`
  unchanged. 25 tests become 9.
Proof: world, `heavy --gate cargo nextest run -p jig-conformance-world
  --profile measure -E 'binary(snapshot)'` once: the test count (9) and
  summed time in the report (the audit: 2.54 s to 1.79 s summed at 8
  threads).
Don't: change a scenario, a seed, a cut, or what is compared.
Size: no production code; about 40 lines of tests.
Removes: 16 tests.

### 04.4 The boundary's walk skips records and other target directories
Scope: testing-strategy.md, 8 ("a lookup by key, not a scan").
Needs: nothing.
Files: `tests/boundary/src/tests.rs` (`walk_directory` `:20-22`,
  `walk_fixture_skips_build_and_hidden_directories` `:202`).
Shape: `walk_directory(name, at_root)` is false for a name starting with
  `.`, for `golden`, and at the root for a name starting with `target`.
Test shape: `walk_fixture_skips_build_and_hidden_directories` gains rows
  for `golden` at any depth and `target-x` at the root, and keeps
  `targets` below the root walked.
Proof: unit, `heavy --gate cargo nextest run -p jig-boundary --profile
  measure -j 1` once: its summed time in the report, against 1.42 s at
  this plan's base (`findings/measurements.md`, 5).
Don't: skip a directory other than `golden` and the root's `target*`;
  change a rule or a forbidden word. (`git ls-files '*/golden/*' | grep
  -v '\.bin$'` prints nothing at this plan's base: a `golden` directory
  holds records only.)
Size: at most 5 lines.
Removes: nothing.

## Done when

- The test counts the reports of 04.1, 04.2 and 04.3 give fall by at
  least 180 together (the audit's groups: about 110, 75 and 16).
