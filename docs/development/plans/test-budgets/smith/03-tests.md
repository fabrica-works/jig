# Session 03: smith's costliest tests and their fixtures

After smith 02, the audit projects smith's focused suite at about 120 s
summed and 16.7 s of wall; the aim is 12 s. This session takes the
costs it found in the tests and their fixtures, and the one in
production code: a log line that formats a 32 MB text, a render bound
checked 256 times on the same outcome, an empty JSON object parsed once
per tool at every product start, focused sweeps of hundreds of worlds
where a handful cover every state, and derivations repeated per record
in the process worlds. Four increments; the audit measured or estimated
about 14 s summed.

## Read first

- smith's `docs/development/workflow.md`, section 2; skein's
  `testing-strategy.md`, section 8 ("Focused tests ... A scenario that
  needs randomness runs the few seeds that show its behaviour"; "Fuzzy
  tests look for what no scenario names: sweeps of many random worlds").
- `findings/cost-smith.md`, section 4, findings 2 to 6 and 8.

## What the design asks

- "A test is cheaper by its algorithm first: a fixture built once per
  test, not per case; a lookup by key, not a scan" (testing-strategy.md,
  8); jig's `workflow.md`, 2 puts the same for traces: "**Traces on
  failure.** Keep typed observations and format them only on failure",
  which this session follows in smith.
- "A scenario that needs randomness runs the few seeds that show its
  behaviour" (testing-strategy.md, 8); sweeps belong to the fuzzy suite.

## Choices this plan makes

- **A sweep's covering seeds stay focused:** for each focused sweep the
  audit names, the seeds that reach every state the test asserts reached
  (7 and 5 of the run world's 315 and 117, 2 of the session world's 200)
  are listed in the test as constants, and the sweep over all its seeds
  moves to the world's `fuzzy_simulation.rs`, beside its other sweeps,
  with the same assertions.
- **The render bound is checked once per drawn cap, at the outcome that
  needs the most:** `random_outcomes_at_random_caps_fit_their_derived_render_bound`
  renders the one Read outcome once at the smallest drawn bound, where it
  is tightest, instead of 256 times.
- **`framing.rs` parses the empty object once** and clones it per tool
  (`crates/smith-protocol-llm/src/framing.rs:16-24`): production, the
  same values.

## Rules that bind it

- No increment "drops an assertion, or changes an expected value, except
  a size its session file names with the reason the assertion holds at
  it" (code-organization.md, section 4).
- No increment "raises a `global-timeout`, a slow period or a per-test
  ceiling, or moves a focused test to the fuzzy suite to fit the budget,
  unless its session file says so" (code-organization.md, section 4):
  this one says so for the sweeps above, and for no other test.
- "**Every smith landing that changes code runs `smoke-tools` live**
  ..., test code included" (README, "Rules").

## Increments

### 03.1 Traces and logs formatted only when they are read
Scope: jig's workflow.md, 2 ("Traces on failure"), in smith's worlds.
Needs: smith 02.1 (smith's gate passes here).
Files: `tests/agent/src/world.rs:1004` (the log line that formats a
  33,554,433-byte text with `{:?}`). The run world's `Debug` traces
  (`tests/run/src/world.rs:897,1009`) stay as they are (ruling 103;
  `later.md`).
Shape: the log line names the transcript by its exact length and a
  fingerprint, not its bytes: a hash (the world's existing hasher) of
  at most its first 512 and its last 512 bytes, labelled as such (ruling
  105). A payload of at most 1 KiB is formatted as today.
Test shape: the worlds' tests, unchanged;
  `every_transient_history_refusal_is_exact_and_starts_no_provider_or_tool_effect`
  (`smith-agent-world::messages`) under 0.1 s alone (the audit: 1.25 s
  to 0.02 s).
Proof: world, `heavy --gate cargo nextest run -p smith-agent-world -p
  smith-run-world --profile measure -j 1` once: summed times in the
  report (at this plan's base, alone: `smith-agent-world::messages` 1.45 s,
  `smith-run-world::simulation` 1.09 s, which this increment does not
  change); `smoke-tools` once before `land`.
Don't: drop a line a replay or a referee compares.
Size: test kit only, at most 30 lines.
Removes: the formatting of unread traces.

### 03.2 The render bound checked once per cap; the framing's object parsed once
Scope: `findings/cost-smith.md`, 4, findings 3 and 4.
Needs: smith 02.1 (smith's gate passes here).
Files: `crates/smith-protocol-llm/src/tests.rs:74-140`;
  `crates/smith-protocol-llm/src/framing.rs:16-24`.
Shape: as "Choices".
Test shape: `random_outcomes_at_random_caps_fit_their_derived_render_bound`
  keeps its name and its seeds; one new test in
  `crates/smith-protocol-llm/src/tests.rs`,
  `each_tools_framing_starts_from_the_same_empty_object`, comparing each
  tool's framing with a literal.
Proof: unit, `heavy --gate cargo nextest run -p smith-protocol-llm -p
  smith-agent-shell --profile measure -j 1` once: summed times in the
  report (the audit: 1.74 s to 0.27 s for the render test; the
  product's start 18.4 ms to 14.3 ms); `smoke-tools` once before
  `land`.
Don't: change what a framing holds, or a drawn cap.
Size: at most 15 lines of production code.
Removes: the repeated parses.

### 03.3 Focused sweeps keep their covering seeds; the sweeps go fuzzy
Scope: testing-strategy.md, 8.
Needs: smith 02.1 (smith's gate passes here).
Files: `tests/run/tests/simulation.rs:655-819`, `tests/run/tests/fuzzy_simulation.rs`;
  `tests/session/tests/simulation.rs:516-538`, `tests/session/tests/fuzzy_simulation.rs`.
Shape: as "Choices": each focused test runs its listed covering seeds
  and asserts every state reached, as now; each sweep, by its old name
  with `_over_many_seeds` appended, runs in the fuzzy file.
Test shape: as Shape.
Proof: world, `heavy --gate cargo nextest run -p smith-run-world -p
  smith-session-world --profile measure` once, and the same with
  `--profile fuzzy` once: every test passing, both `Summary` lines in the
  report; `smoke-tools` once before `land`.
Don't: lose a reached-state assertion; pick seeds without checking they
  reach every state the test names.
Size: test code only, at most 40 lines.
Removes: about 600 focused worlds.

### 03.4 The process worlds and the agent service's tests derive once
Scope: `findings/cost-smith.md`, 4, findings 6 and 8.
Needs: smith 02.1 (smith's gate passes here).
Files: `tests/agent-process/src/lib.rs:150` (`limits()`, called per
  record and per line at `tests/agent-process/src/world.rs:1208-1219` and
  `events.rs:157`); `tests/local-process/src/process/configuration.rs`
  (`lower_configuration` `:12`, `specialize` `:19`,
  `lower_configuration_with` `:243`); `tests/agent-process/src/referee.rs:668`
  (a verdict formatted whatever happens);
  the test modules of `crates/smith-agent-service/src/sinks.rs`
  (`:1316`, `:1432`) and `crates/smith-agent-service/src/calls.rs`
  (`:489-492`); `tests/local-process/tests/fuzzy_store.rs`
  (`configured` `:206`: one line that obtains `lower_configuration()`
  before the world's metered processes start, ruling 113).
Shape: `limits()` and `lower_configuration()` each computed once per
  process (`OnceLock`) and cloned where a caller changes its copy; the
  verdict formatted on failure only; the sinks' tests whose loops run
  10,000 times (`:1316`, `:1362`, `:1403`, `:1432`) run them N times, N
  the smallest power of two past what the test's sink holds under its
  limits (`limits(1)` or `limits(2)`, `:1309`, `:1429`), so its bound is
  still crossed as with 10,000; the totals the tests assert (`:1440-1441`)
  become N, a size this session names. Where N would pass 64, that test
  keeps 10,000 and the report says so. `calls.rs` derives once per test.
Test shape: the tests, unchanged in name and assertion.
Proof: world and unit, `heavy --gate cargo nextest run -p
  smith-agent-process-world -p smith-agent-service -p
  smith-local-process-world --profile measure -j 1` once: summed times in
  the report (at this plan's base, alone: `smith-agent-service` 2.84 s,
  `smith-agent-process-world::stories` 1.09 s); `smoke-tools` once
  before `land`.
Don't: share a fixture a test changes; change a limit the product ships.
Size: test code only, at most 50 lines.
Removes: the repeated derivations.

## Done when

- `heavy --gate cargo nextest run --workspace --profile measure` on
  smith's `main`: every test passes; its wall and summed times in the
  session's last report.
