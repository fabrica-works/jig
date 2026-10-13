# Session 04: smith's 15 s timeout back, here

smith's `.config/nextest.toml` has kept its 15 s timeout, but on
carbon-forge smith has landed without it since jig-protocol's ruling
121: its gate here ran `cargo nextest run --workspace --profile measure`,
which has none (decisions.md, 10). Once smith's other sessions have
landed, this session measures the suite three times in the gate lane
here and, when the median is at most 12 s, lands with smith's own
`scripts/check.sh`, the timeout included. One increment.

## Read first

- skein's `testing-strategy.md`, section 8; smith's `docs/development/workflow.md`,
  sections 1 and 2.
- jig's `jig-protocol/decisions.md`, ruling 121 ("Restored by `land` with
  smith's `scripts/check.sh`, once smith's focused suite fits 15 s
  here").
- decisions.md, 2, 5, 10 and 11.

## What the design asks

- The focused suite "takes at most **15 seconds**", enforced by its
  profile's `global-timeout` (smith's workflow.md, 2).

## Choices this plan makes

- **Three runs, the median,** as jig 06 and skein 02.5.
- **The increment is a landing, not a change:** its commit records the
  measurement and is the first smith landing here through `land` with no
  `--gate`; from it on, the README's smith gate is `scripts/check.sh`.

## Rules that bind it

- "**No increment raises a `global-timeout`, a slow period or a per-test
  ceiling, or moves a focused test to the fuzzy suite to fit the
  budget**" (code-organization.md, section 4).

## Increments

### 04.1 smith lands here with its own gate
Scope: smith's workflow.md, 2; ruling 121's end.
Needs: smith 01, 02 and 03 and `smith-repin` landed; skein 02.1 (`heavy`
  sets `NEXTEST_DOUBLE_SPAWN=0`).
Files: `docs/development/workflow.md`, section 2 (the sentence "The
  budgets are wall time on the development machine (4 cores, 8 threads,
  ...").
Shape: the sentence names the machine: the budgets are wall time on the
  slowest development machine, carbon-forge (an i5-8265U, 4 cores, 8
  threads), with nothing else building (decisions.md, 3). The commit is
  Markdown only and lands with `land`, not `land --docs`, so that its
  gate, `scripts/check.sh`, runs here.
Test shape: none.
Proof: process, on the branch, rebased on `origin/main`: `heavy --gate
  cargo nextest run --workspace` three times, one after another; the
  three `Summary` times in the report, and their median in the commit
  body as a line `Median: <seconds> s`, at most 12.0; then `land` (no
  `--gate`), whose `Gate-Result` names `scripts/check.sh passed`. Past 12.0 s, it lands nothing and reports the three times,
  the twenty slowest tests of the median run and each binary's summed
  time: decisions.md, 5 then applies.
Don't: change a test, a profile or a timeout.
Size: none.
Removes: ruling 121's gate here.

## Done when

- `git -C ~/src/rust/smith log -1 --grep='^Step: smith-04.1$'
  --format='%(trailers:key=Gate-Result,valueonly)' origin/main` names
  `scripts/check.sh passed` and a focused time at most 15 s.
