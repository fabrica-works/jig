# jig-protocol: the focused suite's cost

Six steps not in the plan (decisions.md, ruling 169), from the designer's answer of
2026-10-12 to the supervisor's question "The focused suite's cost" (skein `778ce715`:
`docs/design/codec.md`, 6; `docs/foundation/testing-strategy.md`, 8). The text of each
is the designer's. Order: 1 and 2 at once; then 3; then jig 19.1 and 30.1 regenerate
and land; 4, 5 and 6 when lanes allow.

## 1. jig-tests.s1: The boundary's vocabulary scan

Shape: `tests/boundary/src/tests/vocabulary.rs` skips files under
`src/generated/` and scans each family's `schema/*.schema` instead
(the generated names come from them); the parts split the sorted files
by bytes, each about 1/16 of the total (16 parts); `names.rs`'s scan
finds each forbidden word with `str::match_indices` (std's searcher,
built optimized) and checks the word's boundaries around each match,
in place of the per-character loop at :201-205; the unused `_lower`
goes.
Test shape: the boundary's tests keep their rules and failure text; a
fixture test plants a forbidden word in a schema and in a source past
a part's byte split, each reported with its path.
Proof: `heavy cargo nextest run -p jig-boundary` once: every part under
0.3 s, the vocabulary tests under 2 s together.
Don't: drop a word or a rule; read a file twice in one test; change
`forbidden_words`.

## 2. skein-02.s4: skein-codegen's table tests, small-limit bounds and linear generator

Shape: `emit_tests.rs` writes one generic `golden` and one `bound`
helper per family and table tests of at most 64 cases each, collecting
failures and asserting once, naming each failing case (and its
`Path`); a bound test sets that field's limit to `bound.min(3)` with
every other at `CEILINGS`, and decodes at it and one past it;
`generate()` (`lib.rs:136`) computes once a map from name to
declaration, each type's `has_bounds` and the bound paths, and passes
them down (`bound_names` and the declaration scans go); one
family-level limits check, public, in place of `emit_limit_checks` per
type, which the family's owner calls once where it makes its limits;
`new` and `decode` check values only, each against its limit no higher
than its ceiling, as they do (skein's codec.md, 4; the designer,
2026-10-13, skein `b68a17a9`). As landed, 02.s4 calls the check in every
public `new` and `decode`, which doubled a smith test's time; skein-02.s6
removes those calls.
Test shape: the generator's own tests on a small schema: its table
tests name a planted failing case; a bound test's limit is the small
one; generating a 400-declaration schema takes under 0.5 s unoptimised.
Proof: skein's gate; then, once, smith's and jig's families regenerate
with unchanged golden `.bin` files.
Don't: change an encoding, a golden value or a public type.

## 3. jig-repin.s3: jig repins skein-codegen and regenerates

Shape: `skein-codegen` alone moves in `Cargo.lock`; every family under
`crates/*/src/generated/` regenerates (the store's five, and
worker-wire if landed); the drift tests' ceilings (ruling 153) go back
to the world's.
Test shape: none new; the golden `.bin` files are unchanged.
Proof: the gate; the focused suite's test count falls by about 1,300.
Don't: move another skein package; edit generated code by hand.

## 4. jig-tests.s2: The view's sizes and fixtures

Shape: every loop over `phrases::jig::ALL` and an owner vocabulary for
the largest phrase (`owner_chip.rs:196-202` and the 13 like it) becomes
`words.book.jig_capacity()?`; `Builder::text` opens its node with room
for one run, keeping `text_per_node` for `open(Role::Text)`
(`tree.rs:229,237`, `builder.rs:122-123`); `tests/cards.rs`'s `words()`
returns a shared `&'static Words`, cloned only by tests that change it;
`owner_person_task.rs:309-465`'s storage tests build 3 rows, each
dimension still refused one under; `family_terminals.rs:88-92` builds
its words and limits once, outside the loop.
Test shape: the existing tests, unchanged in name and expectation; one
unit test that a built text node holds room for one run.
Proof: `heavy cargo nextest run -p jig-view -p jig-test-view` once: the
slowest under 0.15 s.
Don't: change a phrase, a limit's value or a worst case's result.

## 5. jig-tests.s3: The worlds' fixtures and checks

Shape: `tests/client/src/test.rs`'s `worst_case` and `view::Limits` are
computed once per limits and borrowed, with the books borrowed, not
cloned; `World::new`'s default heap bound is computed when first asked;
`route_room_checked` (`routing/mod.rs:275`) no longer recomputes
`tasks::worst_case`, which `tasks::Domain`'s construction checks; the
conformance referee compares a policy generation, not the policy
(`harness.rs:382`); `Peers::tick` keeps per-party marks instead of
scanning the store (`peers.rs:165,443`); the engine's trace formats only
under `JIG_WORLD_TRACE` (`effects.rs:519,527`).
Test shape: the existing stories, unchanged; the conformance referee's
test of a policy change still fails a planted breach.
Proof: the gate; joint stories each under 0.3 s.
Don't: change a story's steps, a referee's rule, or a trace's content
when tracing is on.

## 6. skein-tests.s1: skein-heap's peak

Shape: `Highs` keeps its peaks with one pass when full instead of a
merge per push (`testing/skein-heap/src/lib.rs:130,192-213`).
Test shape: its tests, unchanged; one of a burst of rising peaks.
Proof: skein's gate; jig's `ops` client stories under 0.3 s each.
Don't: change what the counting allocator reports.

## 7. skein-02.s5: generated tests in the strict subset

A step not in the plan (decisions.md, ruling 175), after smith's repin
found it: the table tests skein-02.s4 generates (skein `d135339c`) use
`Vec`, `String`, `format!`, `Result::map`/`map_err` closures and
`slice::join`, which smith's and jig's strict configurations refuse in
the crates that hold generated code (jig's root `clippy.toml`,
`disallowed-types`, `-macros`, `-methods`).
Scope: codec.md, 6: generated files "pass the workspace's lints with no
  allowances".
Needs: skein-02.s4.
Files: `testing/skein-codegen/src/emit_tests.rs` and what it calls; a
  crate or module of skein's that compiles one generated family under
  skein's root (strict) `clippy.toml`.
Shape: the generated golden and bound table tests use only what the
  strict configuration allows: the cases a `const` array; a failing case
  named by the assertion's message from the case's `&'static str` name,
  without building a string; matches in place of closure combinators.
  Everything else of 02.s4 stays: about 64 cases a test, small-limit
  bounds, the linear generator.
Test shape: skein's gate's clippy checks one generated family under the
  root `clippy.toml`; a planted failing case is named in its failure.
Proof: skein's gate; then, once, `heavy cargo clippy -p
  jig-protocol-store --all-targets -- -D warnings` on jig's regenerated
  families and the same on smith's three, both exiting 0.
Don't: add an allowance; change an encoding, a golden value or a public
  type.
Size: at most 120 lines of production code.

## 8. skein-02.s6: a family's limits checked where they are made

A step not in the plan (decisions.md, ruling 179), from the designer's
answer (skein `b68a17a9`, `docs/design/codec.md`, 4 and 6: "Constructors
and decoders check values only, never the limits ... a call costs what
its value holds, not the family's size"); its text is the designer's.
Shape: the generator drops `check_limits` from every `new` and `decode`
  (`testing/skein-codegen/src/emit_codec.rs:224,332`) and makes the
  family's check public, naming the first limit past its ceiling;
  `limits_valid` and `check_limits` become one.
Test shape: a generated `new` and `decode` given limits past a ceiling
  still refuse a value past that ceiling.
Proof: skein's gate; then, once, smith's
  `smith_check_json_lists_the_effective_limits_by_name` near its 1,010 ms
  on smith's `main`, within its 2,000 ms bound, with smith's families
  regenerated (`heavy --gate cargo nextest run -p smith --profile measure
  -E 'test(smith_check_json_lists_the_effective_limits_by_name)'` in a
  worktree under `wt/`).
Don't: change an encoding, a golden value, or the `min` clamp.
Size: at most 60 lines of production code.

## 9. jig-repin.s4: jig's families regenerated after skein-02.s6

As jig-repin.s3 (ruling 147: cargo's move of the skein source, no smith
entry), with the later of skein-02.s6's and 02.s7's commits (ruling 182); every family under
`crates/*/src/generated/` regenerates, golden `.bin` files unchanged; each
`codec::<family>::limits` still calls its family's check once. It lands
with `scripts/check.sh`.

## The second pass (2026-10-13)

From the designer's answer to "The focused suite's cost, a second pass"
(skein `cb3cefbf`, `docs/foundation/testing-strategy.md`, 8: "What a
test costs the suite", "A tool is not the code under test"). Measured on
carbon-forge, each crate's tests in one process on one thread against
their test time in the gate of `logs/20261013T051804-83.log`:
`jig-view` 301 in 4.0 s against 13.6 s; `jig-client-world` 267 in 1.4 s
against 10.8 s; `jig-protocol-store` 419 in 0.18 s against 6.1 s;
`jig-domain-core` 210 in 0.36 s against 3.2 s; `jig-domain-client` 205
in 0.21 s against 2.7 s; `jig-protocol-client-link` 133 in 0.22 s
against 1.9 s; the snapshot world 24 in 0.69 s against 3.4 s. A process
costs about 5 ms alone; 2,152 of the 3,508 tests take 15 ms or less in
the gate, 27.2 s together.

## 10. jig-tests.s4: the repository's tools built optimized

Shape: jig's `Cargo.toml` gains `[profile.dev.package.jig-boundary]`
and `[profile.dev.package.skein-codegen]`, each `opt-level = 3`, which
the test profile inherits; `docs/development/workflow.md`, 2's "with
unoptimized test builds" names the exception (testing-strategy.md, 8, "A
tool is not the code under test").
Test shape: none new; the boundary's and the drift tests' names and
failures unchanged.
Proof: `heavy cargo nextest run -p jig-boundary -p jig-store-codecs-world
-p jig-client-wire-world` once: each vocabulary part under 0.1 s, each
drift test under 0.3 s; the next gate's focused time in its `Gate-Result`.
Expected: about 8 s of the gate's 97 s of test time (the boundary's
7.7 s, the drifts' 1.6 s), about 1 s of wall time.
Don't: optimize a crate whose code ships, a world, or a kit.

## 11. jig-tests.s5: one behaviour's cases as rows, where they cost less than their process

Shape: in the unit tests of `jig-protocol-store`, `jig-domain-core`,
`jig-domain-client`, `jig-scripted-engine`, `jig-protocol-client-link`
and `jig-protocol-config`, tests that check one behaviour over cases,
each under 1 ms of work alone (the same assertion over a list of inputs,
names differing only by the case), become rows of one table test per
behaviour that checks every row and names each that fails
(testing-strategy.md, 8, "A test is a process"). Each table keeps the
behaviour's name; its rows keep the old tests' names as labels.
Test shape: no assertion dropped; a planted failing row is named in the
failure, once per table helper.
Proof: the gate; the focused suite's test count and time before and
after, in the report.
Expected: about 12 ms of gate test time per test folded; folding 600
into 60 saves about 7 s.
Don't: fold tests of different behaviours, or a test past 1 ms of work;
change an expected value.

## 12. The steps left (31 to 35, 18, 25 to 27, 33)

Each new test costs about 12 ms of the gate's total before its own work
(testing-strategy.md, 8). So each step's tests are one per behaviour,
with a behaviour's cases as rows; its fixtures are constants or built
once in a test; its limits test-sized. A step's Proof names its tests'
count. jig 34 and 35 write each document's wire record once: a test
kit under `testing/` holds the records, a dev-dependency of both, which
jig 34's encode tests expect and jig 35's decode tests take, each side
keeping its own literal of its own value. jig 34's encoder does not
replace jig 35's twins: a decoder tested against the encoder passes
whatever mistake the two share; their round trip is jig 33's protocol
world's.
