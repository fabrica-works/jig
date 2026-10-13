# Session 02: skein's suite back within 15 seconds

skein's focused suite runs 9.5 to 12.5 s on carbon-forge, under the 30 s
timeout jig-protocol's ruling 249 doubled it to (skein `886a7d4`). This
session makes it and what it hands its consumers cheaper, then restores
the 15 s timeout: nextest without its double spawn, for every gate and
every command the queue runs; the codec generator's worst cases checking
a family's limits once, and its golden tables in fewer tests; the
suite's costliest tests sized to what they assert; and lib's search for
one byte by a plain scan, which smith pays for. Five increments, the
last the budget's.

## Read first

- skein's `testing-strategy.md`, section 8 (two suites; "A test is a
  process"; "What a test costs the suite"; "A tool is not the code under
  test"), as this plan's design landing revised it (skein's design
  commit in README, "Inputs").
- skein's `codec.md`, 6 (generated code and its tests) and 4 ("a call
  costs what its value holds, not the family's size").
- `findings/cost-skein.md` (the audit: each test, its cause, the change
  tried and its time) and `findings/cost-jig-units.md`, 1, 2 and 4 (the
  double spawn, the generated worst cases, the golden tables).
- jig's `jig-protocol/tests-cost.md`, steps 2, 7 and 8 (what the codec
  generator's tests became).

## What the design asks

- "The suite takes at most **15 seconds**" (testing-strategy.md, 8); "A
  change that breaks a budget is fixed by making tests cheaper";
  "Raising a budget is an explicit decision, not a fix."
- "**A test is a process.** nextest runs each test in a process of its
  own ... The gate, and every command the machine's queue runs, starts
  nextest without its double spawn (`NEXTEST_DOUBLE_SPAWN=0`, set by each
  repository's `scripts/check.sh` and by `heavy`)" (testing-strategy.md,
  8, as revised).
- "A test is cheaper by its algorithm first: a fixture built once per
  test, not per case; a lookup by key, not a scan" (testing-strategy.md,
  8).

## Choices this plan makes

- **Double spawn off in two places:** each repository's
  `scripts/check.sh` exports `NEXTEST_DOUBLE_SPAWN=0` before its first
  `cargo nextest`, so a gate run by hand costs what `land`'s does; and
  skein's kit, `scripts/dev/heavy`, exports it for every command it
  queues, so a measurement through the queue costs what the gate's does.
  smith's and jig's `check.sh` are their sessions' (smith 02.1, jig
  01.1). cargo's `[env]` does not reach `cargo nextest`
  (`findings/cost-jig-units.md`, 1).
- **The generator's worst cases:** each public `worst_case_bytes` and
  `worst_case_heap` checks the family's limits once, then calls a
  generated inner function that does not, and recurses only through
  inner functions (`emit_worst.rs:60-97`, the type-level functions, and
  `:107-123`, the family-level ones). Results are not
  remembered: what that would save is about 0.3 ms per store open
  (`findings/cost-jig-units.md`, 2).
- **Golden tables of up to 1,024 cases** (`emit_tests.rs:261`,
  `cases.chunks(64)`): each case decodes and encodes one value of under a
  millisecond, so a table of 1,024 stays under the unit test's 20 ms
  ceiling; jig's 113 golden tests become about 10. Bound tables keep 64.
- **The request table's test runs its largest capacity for one type
  pair:** `RequestTable` is used by nothing outside `skein-lib`, so its
  operations' scans of `order` (O(N) each, a fill and drain O(N²)) go to
  `later.md`, and the memory test keeps capacity 300 for `<[u64; 8],
  [u64; 8]>` and caps the other five pairs at 64: the maps' types and keys
  are the same for every pair, and the slab is covered at 300 by
  `a_slab_holds_exactly_its_worst_case` (`findings/cost-skein.md`, 6).
- **Downstream regenerates:** smith's three families and jig's families
  under `crates/*/src/generated/` and `testing/*/src/generated/`
  regenerate when they take 02.2 (smith 01.1, jig 05.1), golden `.bin`
  files unchanged.
- **The 15 s timeout comes back when the suite's median is at most
  12 s** (decisions.md, 2): three runs of the focused suite in the gate
  lane on carbon-forge (02.5).

## Rules that bind it

- No increment "drops an assertion, or changes an expected value, except
  a size its session file names with the reason the assertion holds at
  it" (code-organization.md, section 4).
- No increment "builds optimized a crate that ships, a world or a kit
  (testing-strategy.md, 8, "A tool is not the code under test")"
  (code-organization.md, section 4).
- "**smith's reliability plan lands in smith and skein from
  debian-16gb-hel1-1** ... `land` rebases each landing on its landings"
  (README, "Rules"): its machine uses the same kit, `scripts/dev/`.

## Increments

### 02.1 nextest without its double spawn, in skein's gate and the queue
Scope: testing-strategy.md, 8 ("The gate runs nextest without its double
  spawn").
Needs: nothing.
Files: `scripts/check.sh`; `scripts/dev/heavy` (the environment of the
  command it queues).
Shape: `export NEXTEST_DOUBLE_SPAWN=0` in `scripts/check.sh`, before its
  first `cargo nextest`, with a comment naming testing-strategy.md, 8;
  `heavy` passes `NEXTEST_DOUBLE_SPAWN=0` in the queued command's
  environment unless the caller set it.
Test shape: none new; `heavy test-plan` (the kit's own tests, development.md,
  3.6) passes.
Proof: process, the focused time in `land`'s `Gate-Result`, against
  11.405 s (skein `b5d3a2c`'s); nothing runs before `land` but `heavy
  test-plan` once.
Don't: change a profile's `global-timeout` here (02.5's); set it in
  `.config/nextest.toml` or `~/.cargo/config.toml`.
Size: at most 10 lines.
Removes: nothing.

### 02.2 The generator's worst cases check a family's limits once; golden tables of 1,024
Scope: codec.md, 4 ("a call costs what its value holds, not the
  family's size") for the worst-case functions; testing-strategy.md, 8
  ("cases of one behaviour ... are rows of one table test").
Needs: nothing.
Files: `testing/skein-codegen/src/emit_worst.rs` (`emit_record` `:60-74`,
  `emit_enum` `:76-97`, the field expressions `:40` and `:56` that call
  `{name}::worst_case_bytes(limits)?` and `worst_case_heap`, and the
  family-level `worst_case_bytes` and `worst_case_heap` `:107-123`, which
  call each top-level type's), `testing/skein-codegen/src/emit_tests.rs`
  (`tables` `:260-271`: the golden tables' chunk),
  `testing/skein-codegen/src/tests.rs`; `tests/codec/src/generated/`,
  `src/secrets/`, `src/scalars/` and `src/lints/` (skein's four sample
  families, regenerated by their drift test's own environment variable,
  `SKEIN_CODEGEN_WRITE=1`; ruling 101) and `tests/codec/tests/worst.rs`.
Shape: per type, `pub fn worst_case_bytes(limits: &Limits) ->
  Option<u64>` and `worst_case_heap` keep their signatures and docs: each
  returns `None` if `check_limits(limits)` refuses, else its inner
  `fn worst_case_bytes_within(limits: &Limits) -> Option<u64>` (and
  `worst_case_heap_within`), `pub(crate)`, which recurse into other
  types' `_within` functions and never call `check_limits`. The family's
  own `worst_case_bytes` and `worst_case_heap` check once, then call the
  top-level types' `_within` functions. Golden tables chunk by 1,024;
  bound tables keep 64.
Test shape: in `testing/skein-codegen/src/tests.rs`:
  `a_worst_case_checks_its_familys_limits_once` (the generated source of
  `small_bound_family()` (`:40`) holds one `check_limits` call per public
  worst-case function and none in a `_within` function);
  and, in `tests/codec/tests/worst.rs`,
  `a_worst_case_past_a_ceiling_is_none` (`tests/codec`'s generated
  family: its top type's worst cases are `None` with one limit past its
  ceiling, and equal at `CEILINGS` to the old functions' values, taken
  from a run on `main` before the change and written as literals).
  `tests/codec`'s golden `.bin` files are unchanged.
Proof: unit, `heavy cargo nextest run -p skein-codegen -p
  skein-codec-tests` once, every test passing, with the golden files
  unchanged (`git diff --stat origin/main -- tests/codec` names only
  generated `.rs` files and `tests/worst.rs`).
Don't: change a generated encoding, a golden value, a public signature
  or `check_limits`'s refusals; remember results across calls.
Size: at most 60 lines of generator code.
Removes: the per-node `check_limits` calls.

### 02.3 The suite's costliest tests sized to what they assert
Scope: jig's workflow.md, 2 ("Test-sized limits"); testing-strategy.md,
  8 ("a fixture built
  once").
Needs: nothing.
Files (each change as `findings/cost-skein.md` gives it, tried in the
  audit's scratch tree): `tests/http/tests/memory.rs:455,457`;
  `tests/llm/tests/memory.rs:719-720,778,785` and
  `tests/llm/tests/selective.rs:30,43`; `tests/json/tests/memory.rs:237,350,376`;
  `crates/skein-llm/src/openai/tests.rs:1390,1395`; `tests/ring/src/lib.rs:218-224`
  (`pattern`) and `tests/ring/tests/ring.rs:26,39,48`;
  `tests/http/transcripts/hostile-event-never-ends.*`,
  `hostile-event-line-too-long.*`, `hostile-comments-without-end.*`;
  `tests/lib/tests/memory.rs:600-611`.
Shape:
  - the megabyte SSE event: `1 << 14` bytes, the test renamed
    `an_event_far_longer_than_its_chunk_costs_the_same_reader_memory_as_a_small_one`
    (its worst case depends only on `chunk` and `field`, `sse.rs:100-107`);
  - the selective LLM tests: `1 << 18` bytes, the expected `Oversize`
    bytes `262_144` (still past every limit but `skip`, the answer cap
    `receiving`, 123,436, among them), the comment at `:719-720` updated;
  - the JSON megabyte tests: `1 << 12` bytes, two renamed so their names
    no longer say megabytes (every limit but `length` and `skip` is 32
    bytes or less);
  - the native-arguments test: `131_072` bytes (past the projection's
    limits of 128 and 512, `tests.rs:1311-1313`);
  - `pattern(n)` fills one period of 65,536 bytes and repeats it (the
    formula's period: the same bytes); the large transfer takes its
    buffer back from the `Send`'s completion, as receives do (`ring.rs:55`),
    instead of cloning 32 MiB per send;
  - the three hostile transcripts state smaller SSE limits (`sse event
    2048`; `sse line 512 event 8192`) and carry bodies just past them
    (2,200, 2,080 and 2,000 bytes), `Content-Length` and framing
    updated; their `.expect` files keep their failures.
  - the request table's memory test (`tests/lib/tests/memory.rs:600-611`):
    capacity 300 for `<[u64; 8], [u64; 8]>` only, up to 64 for the others.
Test shape: the tests named, each keeping its assertions; no new test.
  Ceilings: each under 0.25 s alone.
Proof: unit and world, `heavy --gate cargo nextest run -p
  skein-http-world -p skein-llm-world -p skein-json-world -p
  skein-lib-tests -p skein-ring-tests -p skein-llm --profile measure -j 1
  -E 'binary(memory) | binary(selective) | binary(transcripts) |
  binary(ring) | binary(files) | test(/native_function_arguments/)'`
  once: each named test's time against `findings/cost-skein.md`'s
  "before", in the report; then `heavy cargo nextest run -p
  skein-http-world -p skein-json-world --profile fuzzy` once, passing
  (the transcripts are shared with the fuzzy suite).
Don't: change a limit the code under test ships with, a reference
  reader, or an expected failure.
Size: no production code; about 100 lines of tests changed, besides the
  transcripts' filler removed.
Removes: about 1,700 lines of transcript filler.

### 02.4 lib's search for one byte is a plain scan
Scope: lib.md, 6 (`bytes::find`, `find_from`, `count`: unchanged in what
  they return); testing-strategy.md, 8 ("a test is cheaper by its
  algorithm first").
Needs: nothing.
Files: `crates/skein-lib/src/bytes.rs` (`find_from` `:40-48`, `count`
  `:76-92`), `crates/skein-lib/src/tests/bytes.rs`.
Shape: in `find_from` and `count`, a needle of one byte is found by a
  loop over the haystack comparing each byte, without building a
  `TwoWay`; longer needles keep the two-way search.
Test shape: `crates/skein-lib/src/tests/bytes.rs`:
  `a_one_byte_needle_is_found_as_the_two_way_search_finds_it` (every
  position of a 64-byte haystack of two symbols, `find_from` and `count`
  with caps 1, 2 and 64, against a naive reference in the test); the
  existing tests unchanged.
Proof: unit, `heavy cargo nextest run -p skein-lib -E 'test(/bytes/)'`
  once, passing; smith's gain arrives with its repin (the audit
  estimates about 2.4 s of smith's summed time,
  `findings/cost-smith.md`, 4, finding 7).
Don't: change what `find`, `find_from` or `count` return: the named
  test is the check.
Size: at most 15 lines of production code.
Removes: nothing.

### 02.5 The 15 s timeout back
Scope: testing-strategy.md, 8 ("at most 15 seconds"); jig-protocol's
  ruling 249 ("then its budget goes back").
Needs: 02.1 to 02.4 landed; skein 01 landed (its tests are in the
  suite measured).
Files: `.config/nextest.toml` (the default profile's `global-timeout`,
  and its header comment, "within 30 seconds").
Shape: `global-timeout = "15s"`, the header back to "within 15 seconds".
Test shape: none.
Proof: process, on the branch, rebased on `origin/main`: `heavy --gate
  cargo nextest run --workspace` three times, one after another; the
  three `Summary` times in the report, and their median in the commit
  body as a line `Median: <seconds> s`, at most 12.0. Past 12.0 s, it
  lands nothing and reports the three times, the twenty slowest tests of
  the median run and each binary's summed time (decisions.md, 5 decides
  what follows).
Don't: change a slow period, the fuzzy profile, or a test.
Size: two lines.
Removes: the doubled timeout.

## Done when

- `grep -n 'global-timeout = "15s"' ~/src/rust/skein/.config/nextest.toml`
  prints the default profile's line.
- `grep -n NEXTEST_DOUBLE_SPAWN ~/src/rust/skein/scripts/check.sh
  ~/src/rust/skein/scripts/dev/heavy` prints a line in each.
- `git -C ~/src/rust/skein log -1 --grep='^Step: skein-02.5$'
  --format='%(trailers:key=Gate-Result,valueonly)' origin/main` names
  `scripts/check.sh passed`.
