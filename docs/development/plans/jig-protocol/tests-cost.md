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
family-level limits check, called once per public `new` and `decode`,
in place of `emit_limit_checks` per type.
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
