# Session 06: jig's 15 s timeout back

jig's focused suite ran under a 30 s timeout for the end of jig-protocol
(ruling 249, jig `1acbf0794`), "until jig-protocol ends". Once jig's
other sessions have landed, this session measures the suite three times
in the gate lane on carbon-forge and, when the median is at most 12 s,
puts the 15 s timeout back. One increment.

## Read first

- skein's `testing-strategy.md`, section 8 ("The budgets ... are
  enforced"; "Raising a budget is an explicit decision, not a fix").
- jig's `docs/development/workflow.md`, section 2.
- decisions.md, 2 (the aim) and 5 (what follows if it is missed).

## What the design asks

- "It takes at most **15 seconds**" (workflow.md, 2); "`.config/nextest.toml`
  gives each profile a `global-timeout`, so a suite that runs past its
  budget fails".

## Choices this plan makes

- **The aim is 12 s, the timeout 15 s** (decisions.md, 2).
- **Three runs, the median,** each the focused suite alone in the gate
  lane: one run on carbon-forge varies by 10 % or more with what runs
  beside the queue.

## Rules that bind it

- "**No increment raises a `global-timeout`, a slow period or a per-test
  ceiling, or moves a focused test to the fuzzy suite to fit the
  budget**" (code-organization.md, section 4).

## Increments

### 06.1 The 15 s timeout back
Scope: workflow.md, 2; jig-protocol's ruling 249 ("then its budget goes
  back").
Needs: jig 01, 02, 03, 04 and 05 and `jig-repin` landed; skein 02.1
  (`heavy` sets `NEXTEST_DOUBLE_SPAWN=0`).
Files: `.config/nextest.toml` (the default profile's `global-timeout`).
Shape: `global-timeout = "15s"`; the header comment already says "within
  15 seconds".
Test shape: none.
Proof: process, on the branch, rebased on `origin/main`: `heavy --gate
  cargo nextest run --workspace` three times, one after another
  (`scripts/check.sh` sets `NEXTEST_DOUBLE_SPAWN=0`; these runs get it
  from `heavy`); the three `Summary` times in the report, and their
  median in the commit body as a line `Median: <seconds> s`, at most
  12.0. Past 12.0 s, it lands nothing and reports the three
  times, the twenty slowest tests of the median run with their times,
  and each binary's summed time: decisions.md, 5 then applies.
Don't: change a slow period, the fuzzy profile, a test or a fixture.
Size: one line.
Removes: the doubled timeout.

## Done when

- `grep -n 'global-timeout = "15s"' ~/src/rust/jig/.config/nextest.toml`
  prints the default profile's line.
- `git -C ~/src/rust/jig log -1 --grep='^Step: jig-06.1$'
  --format='%(trailers:key=Gate-Result,valueonly)' origin/main` names
  `scripts/check.sh passed` and a focused time at most 15 s.
