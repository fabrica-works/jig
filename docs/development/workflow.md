# Development workflow

How work reaches main. Work happens on local branches; main moves only by
merging a branch that passed every check below, or that changes only
documentation.

## 1. Before merging to main

The gate is `scripts/check.sh`, run on the branch's tip rebased on main, so
that what is checked is what main becomes:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --target wasm32-unknown-unknown \
    -p jig-client-protocol-wire -p jig-client-protocol-link \
    -p jig-client-protocol-http -p jig-client-domain -p jig-client \
    -p jig-client-view -p jig-client-test-protocol-wire \
    -p jig-client-test-domain -p jig-client-test-view \
    -p jig-ops-client-protocol-wire -p jig-ops-client-domain \
    -p jig-ops-client-view -- -D warnings
NEXTEST_DOUBLE_SPAWN=0 cargo nextest run --workspace
NEXTEST_DOUBLE_SPAWN=0 cargo nextest run --workspace --profile fuzzy
```

skein's `land` (skein's `docs/foundation/development.md`, section 9) does
it all from a clean worktree on the branch: it rebases on `origin/main`,
runs the gate through the machine's queue, records its result in the tip
commit's trailers, and pushes the tip to `origin`'s `main`, which only
fast-forwards. The local `main` follows.

A branch that changes only Markdown files (the documents under `docs/`,
`AGENTS.md`) skips the gate, since nothing it touches is built or tested,
and lands with `land --docs`. Doc comments in `.rs` files are not
documentation in this sense: they are code, and clippy checks them.

## 2. The two test suites

The suites are those of skein's `docs/foundation/testing-strategy.md`
(section 8); `docs/design/domain/testing.md` (section 9) says what they
hold in jig.

- **The default suite** is every crate's unit tests and each world's
  focused tests (`tests/<name>/tests/*.rs`): focused tests of
  expected behaviour (scenarios, referee tests, replay, facts changing
  nothing, memory against the worst case), and a cheap random world as a
  smoke test where one helps. It is what `NEXTEST_DOUBLE_SPAWN=0 cargo nextest run --workspace`
  runs, and it takes at most **15 seconds**.
- **The fuzzy suite** is the worlds' `tests/fuzzy_*.rs`: randomized tests,
  such as sweeps of random worlds and domains driven at random, over many
  seeds. It is not run by default, only with `--profile fuzzy`, as the
  gate before merging to main, and it takes at most **1 minute**.

Tests cost what they check, with unoptimized test builds except for the
repository's tools, `jig-boundary` and `skein-codegen`, which are built
optimized (testing-strategy.md, 8, "A tool is not the code under test"):

- **Fixtures once.** Build a phrasebook, configuration or worst-case size
  once per test process or world when it is a fixture, then share it.
- **Checks on change.** Check what changed when it changes. Check the whole
  state at the end, unless waiting would lose the bug the check guards.
- **Traces on failure.** Keep typed observations and format them only on
  failure, or replay the failing seed with tracing enabled. Record coverage
  as typed facts; do not build or search `Debug` dumps for it.
- **Test-sized limits.** Use limits as small as the assertions allow, so
  preallocation is proportionate. A checked memory bound can fail: a test
  shows it catching a regression.
- **Seeds sized to their assertions.** Enumerate combinations that can be
  enumerated, then draw seeds for the remaining faults and timing choices.
  Coverage markers prove every reach the sweep claims.
- **Long runs are several tests.** Aim for each focused test to take at most
  about 0.5 s, and each fuzzy test about 5 s, unoptimized on the idle
  development machine. Split longer sweeps so the gate can share them
  across threads, retaining their assertions and coverage. The nextest slow
  periods flag these times; they do not fail a test.

The budgets are enforced, not advisory: `.config/nextest.toml` gives each
profile a `global-timeout`, so a suite that runs past its budget fails,
naming the tests it stopped, and a slow period past which a test is
flagged SLOW as it runs. A change that breaks a budget is fixed by making
tests cheaper, or by moving randomized ones to a world's fuzzy tests.
Raising a budget is a decision to take explicitly, not a fix. If a gate
exceeds a suite budget, retry its unchanged source once; the second run
decides. Record source, pre-run load, and the slowest focused and fuzzy
names with their times. Serial totals and targets are informational, not
additional merge gates. To see what each test costs on its own, the
`measure` profile has no suite budget:

```sh
NEXTEST_DOUBLE_SPAWN=0 cargo nextest run --workspace --profile measure -j 1
```

The budgets are wall time on the development machine (4 cores, 8 threads,
which nextest uses all of), with nothing else building. Run the checks on
an idle machine.

## 3. When a fuzzy seed fails

A failure names its seed, and a seed replays to the same run. Fix the bug,
then keep its seed: as a scenario among its world's focused tests if it
shows behaviour worth naming, or among the sweep's pinned seeds. A finding
that cannot be fixed yet joins its world's `FINDINGS`, which an ignored
test replays until it is fixed.

The tasks, host, conformance and ops harnesses normally keep tracing off in
sweeps. Unset `JIG_WORLD_TRACE` for that mode: they keep no diagnostic lines
and do no diagnostic formatting or event copies. Typed coverage remains
active. The variable's presence enables the complete trace, including reply
identities and referee payloads; even `JIG_WORLD_TRACE=0` enables it. Each
harness reads it once per process. Focused replay tests can instead select
tracing explicitly through their existing constructors.

To diagnose a reported seed, replay the same deterministic test with tracing
on. For example, this replays the lifecycle's enumerated choices and drawn
seeds, with whole boundary diagnostics available if a run fails:

```sh
JIG_WORLD_TRACE=1 cargo nextest run -p jig-tasks-world --profile measure \
  --ignore-default-filter -j 1 \
  -E 'test(random_faults_and_restart_cuts_reach_every_retained_ending)'
```

The variable selects tracing, not seeds. To replay one seed alone, pin the
reported seed in its world's focused replay test using the existing
trace-enabled constructor. For tasks, `run_story_traced(seed, true, true)`
replays the ordinary seeded story; an enumerated lifecycle failure also
names its complete `Story` choices, which `run_story_choices(seed, choices)`
replays with the environment switch on. Preserve the whole-trace and
complete-state comparisons in those tests. A failing conformance or ops
scenario names its scenario, seed and cut: replay that same tuple through
its existing scenario runner with tracing enabled. After fixing it, retain
the seed as described above, and unset the switch for normal sweeps.
