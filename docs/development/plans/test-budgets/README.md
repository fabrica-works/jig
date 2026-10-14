# test-budgets: the focused suites back within 15 seconds, and a read of what has arrived

2026-10-13. Start commits: skein `2df5b58` (this plan's design landing,
on `2068257`), smith `0305f2fd`, jig `d91a89c45` (each repository's
`origin/main` when the plan was written; each session starts from its
repository's `origin/main`).

The last plan, jig-protocol, ended with jig's and skein's focused suites
under a doubled timeout of 30 s and smith's landing on carbon-forge with
none (its ruling 249), each to get a test-cost pass and its budget back
once the plan ended. This plan does that, measuring every cost first and
fixing it at its cause: nextest's double spawn, which doubles what a
test's process costs; checks run on every iteration whatever changed
(the channel world's referee formatted 22 KB of unchanged state each
time); worst cases and documents computed again for every member, node
and heartbeat; byte-at-a-time searches, decoders and fills in
unoptimized code; sweeps of hundreds of worlds in the focused suite; a
smith pin that went backwards. It also builds skein's read of what has
arrived (the user's option (a), 2026-10-13): a reader of bytes nothing
frames pays one delivery for what arrived together, not one iteration
of its loop per byte. Each suite's aim is 12 s on carbon-forge, under
its 15 s timeout (decisions.md, 2).

## Inputs

- **skein's design** on skein's `main`, with this plan's design landing
  (skein `2df5b58`, "Add a read of what has arrived; run nextest without
  its double spawn"): `lib.md`, section 7; `io.md`, 2 and 3.3; `http.md`,
  3.3, 4 and 6; `tls.md`, 3.1; `programming-model.md`, 4.3;
  `testing-strategy.md`, 2.4 and 8; `codec.md`, 4 and 6.
- **The repositories' workflows:** skein's `docs/design/testing.md`,
  section 6; smith's and jig's `docs/development/workflow.md`, sections 1,
  2 and (smith) 5.
- **skein's foundation:** `development.md` (this plan follows it),
  `testing-strategy.md`.
- **`decisions.md`:** what this plan settled, and the user's choices.
- **`findings/`:** `measurements.md` (the suites at this plan's base);
  `reads.md` (the code the new read touches); `cost-skein.md`,
  `cost-smith.md`, `cost-jig-units.md`, `cost-jig-channel.md`,
  `cost-jig-client.md` (each suite's audit, every cost measured); `tried/`
  (the audits' scratch changes, as evidence).
- **jig's `jig-protocol/`:** `decisions.md` (rulings 120, 121, 249, 268;
  "For the user"), `tests-cost.md` (the focused suite's first two passes)
  and `smith-settle.md` (skein-02.s8, smith's settle test, the raw reader).

## The sessions

A session waits for what "Needs at start" names; an increment that needs
more says so in its row and its file. Each increment also builds on the
ones before it in its session. Dependencies are by capability.

### skein

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `skein/01-arrived.md` | skein 01: a read of what has arrived | nothing | |
| `skein/02-suite.md` | skein 02: skein's suite back within 15 s | nothing | 02.5 (the timeout): 02.1 to 02.4, and skein 01 |

### smith

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `smith/01-arrived.md` | smith 01: smith reads what has arrived | skein 01's read and readers (01.1, 01.3); skein 02's generated worst cases (02.2); smith 02.1 (the failures here) | |
| `smith/02-gate.md` | smith 02: smith's gate passes here, its real tier in parallel | nothing | 02.2 to 02.4: 02.1 |
| `smith/03-tests.md` | smith 03: smith's costliest tests and fixtures | smith 02.1 (the failures here) | |
| `smith/04-budget.md` | smith 04: smith's 15 s timeout back, here | smith 01, 02, 03 and `smith-repin`; skein 02.1 (`heavy`) | |

### jig

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `jig/01-gate.md` | jig 01: the gate, and what every world pays | nothing | |
| `jig/02-channel.md` | jig 02: the channel world | nothing | |
| `jig/03-client.md` | jig 03: the client's worlds and the view | nothing | |
| `jig/04-rows.md` | jig 04: cases as rows, and the walks over the tree | nothing | |
| `jig/05-arrived.md` | jig 05: jig takes skein's and smith's changes, and reads what has arrived | smith 01.1; skein 02.2 | |
| `jig/06-budget.md` | jig 06: jig's 15 s timeout back | jig 01 to 05 and `jig-repin`; skein 02.1 (`heavy`) | |

### Repins (development.md, 5.7)

- **`smith-repin`:** after every skein step (skein 01 and 02, 02.5
  included) and smith 01 to 03: takes skein's `main` (`heavy cargo update
  -p skein-lib`), and regenerates smith's families as smith 01.1 does if
  smith's drift tests (`tests/codecs/tests/drift.rs`) then fail.
- **`jig-repin`:** after `smith-repin`, smith 04 and jig 01 to 05: takes
  smith's `main` and the skein revision smith's `Cargo.lock` then holds,
  as jig 05.1 does, and regenerates jig's families as jig 05.1 does if
  jig's drift tests then fail.
- Nothing downstream of jig (decisions.md, 12).

### How many at once, and in what order

- **At the start:** skein 01 and 02; smith 02; jig 01, 02, 03 and 04:
  seven sessions, as the machine's slots allow (development.md, 7.2 and
  11.4). smith 02.1 lands first among smith's: smith's gate here runs
  every focused test, and seven fail at the base (decisions.md, 10).
- **Then:** smith 03 once smith 02.1 has landed; smith 01 once skein
  01.1, 01.3 and 02.2 and smith 02.1 have; jig 05 once smith 01.1 has;
  at the end, in this order: skein 02.5, `smith-repin`, smith 04,
  `jig-repin`, jig 06.
- **When slots are short, start first** the sessions others wait on, in
  this order:
  1. smith 02, skein 01, skein 02, jig 01;
  2. smith 01, jig 02, jig 03, smith 03;
  3. jig 05, jig 04;
  4. skein 02.5; `smith-repin`; smith 04; `jig-repin`; jig 06.
- **The longest chain:** skein 01 → smith 01 → jig 05, then skein 02.5 →
  `smith-repin` → smith 04 → `jig-repin` → jig 06.

## Rules of this plan

- **Every session reads,** besides its "Read first": its repository's
  `AGENTS.md` and workflow, this README and `code-organization.md`.
- **Times are measured as each Proof says, through `heavy`, once, on
  the branch,** and reported with the command and the log's path; the
  "before" is the number the session file or `findings/` gives, never
  measured again. Nothing else is measured, and no measurement is
  repeated unless its Proof says so (development.md, 4.5).
- **Until skein 02.1 lands,** each measuring command in a Proof runs as
  `NEXTEST_DOUBLE_SPAWN=0 heavy --gate ...`, as the gates will.
- **A Proof's time is a guide:** past it, the increment lands and its
  report gives the time; only a failing assertion stops it
  (development.md, 8.8), or a test past twice its ceiling (8.2).
- **A summed time** is the sum of the `PASS` times in the nextest log
  `heavy` names: `awk -F'[][]' '/PASS \[/ {s += $2} END {printf "%.1f\n", s}'
  <log>`; a wall time is the log's `Summary` line.
- **Each increment's report names the suite's focused time** from its
  landing's `Gate-Result` (its first `Summary`), so the supervisor's
  hourly check follows the suites toward 12 s.
- **A commit's last paragraph is its trailers** (ruling 102): `Proof:`,
  `Plan:` and `Step:`, then `Co-Authored-By:` if any, one per line, with
  no blank line between them and no other line among them. Other
  evidence (logs, counts, clippy) goes in the body above, never as a
  `Key: value` line whose key has a space. Before `land`, `git log -1
  --format='%(trailers:key=Step,valueonly)' HEAD` must print the step;
  if it prints nothing, amend the unlanded commit. A landed commit is
  never rewritten.
- **The coordinator's check of a landed report** (ruling 104): an
  implementer goes on after a landing (development.md, 8.6: "After a
  landing, send the report and go on"), so its worktree may already hold
  its next increment. Run `git -C <worktree> rev-parse --abbrev-ref HEAD`
  first. If it prints the landed step's branch, `git status --porcelain`
  must print nothing, as development.md, 7.3 says; if it prints another
  branch, `git status --porcelain` is not run for that landing. The
  other two checks of 7.3 (`merge-base --is-ancestor`, the `Step`
  trailer) stand for every landing.
- **smith's window story may be landed again once** (ruling 108), until
  the step that fixes it lands: when a smith gate fails only on
  `smith::end_to_end a_window_size_change_reaches_the_status_line`, with
  widths `[85, 20]` against `[80, 20]`, `land` again once without
  asking, as development.md, 8.5 allows for a timeout, and name the
  failed gate's log in the commit body. A second failure on the same
  tree, or any other failure, is a question as 8.5 says.
- **smith's gate on carbon-forge** is ruling 121's (decisions.md, 10),
  with `NEXTEST_DOUBLE_SPAWN=0`, until smith 04.1: `land --gate 'cargo
  fmt --all --check && cargo clippy --workspace --all-targets -- -D
  warnings && NEXTEST_DOUBLE_SPAWN=0 cargo nextest run --workspace
  --profile measure && NEXTEST_DOUBLE_SPAWN=0 cargo nextest run
  --workspace --profile fuzzy'`.
- **Every smith landing that changes code runs `smoke-tools` live**
  (smith's workflow.md, 5; "Live services" below), test code included:
  once through `heavy --gate` before `land`, its run identifier and
  verdict in the commit body.
- **smith's reliability plan lands in smith and skein from
  debian-16gb-hel1-1** (decisions.md, 11). `land` rebases each landing on
  its landings; a conflict in a file both changed is resolved by keeping
  both changes, and one that cannot be is a question. Its sessions edit
  smith's `tests/end-to-end`, `tests/local-process` and `crates/smith-local-*`,
  and skein's `crates/skein-io/`, `crates/skein-shell/` and the kit,
  `scripts/dev/`. skein 01.1's new variant breaks smith's two exhaustive
  matches on `Read` (`smith-events/src/read.rs`,
  `smith-protocol-machine/src/search.rs`) until smith 01.1: when skein
  01.1 lands, the supervisor tells the user, so that the reliability run
  can add those two arms if it repins skein before smith 01.1 lands.
- **A repin never moves a pin back:** before a `cargo update --precise`,
  the step checks that the commit is a descendant of the one the lock
  holds (`git -C ~/src/rust/<repository> merge-base --is-ancestor <held>
  <new>` exits 0); otherwise it skips the move and says so.
- **The aim and the timeout** (decisions.md, 2 and 5): each repository's
  last increment (skein 02.5, smith 04.1, jig 06.1) measures three runs
  and restores the 15 s timeout when the median is at most 12.0 s; past
  it, it lands nothing, and decisions.md, 5 applies.

## Live services

- **smith's `smoke-tools`,** for every smith landing that changes code
  (smith 01, 02, 03 and `smith-repin`), as smith's `docs/development/workflow.md`,
  section 5 requires: the cheapest model and effort, once through `heavy
  --gate` before `land`, once more only if it fails; about 10,000 tokens a
  run, borrowing the user's login as smith's benchmarks do, read only.
  At most 30 runs in the plan.
- **Nothing else is live:** skein's fake peers serve every other proof.

## Repin

Every repository downstream of what the plan changes is repinned
(development.md, 5.7): smith by smith 01.1 and `smith-repin`, jig by jig
05.1 and `jig-repin`. Nothing downstream of jig: decisions.md, 12.

## Pulse

```
words: legacy|compat|hack|workaround
types: ^(Legacy|Old|Compat)[A-Z]
file-lines: 800
budgets: 15 60
```

## Done when

1. Every step is landed or dropped: `plan status` lists none waiting,
   ready, running or blocked.
2. Each focused suite has its 15 s timeout: `grep -n 'global-timeout =
   "15s"' ~/src/rust/skein/.config/nextest.toml
   ~/src/rust/jig/.config/nextest.toml ~/src/rust/smith/.config/nextest.toml`
   prints the default profile's line in each.
3. Each budget's landing passed its repository's own gate here: for
   skein 02.5, smith 04.1 and jig 06.1, `git -C ~/src/rust/<repository>
   log -1 --grep='^Step: <step>$' --format='%(trailers:key=Gate-Result,valueonly)'
   origin/main` names `scripts/check.sh passed` and a focused time at
   most 15 s.
4. Each suite met the aim: the same three commits' bodies each hold a
   line `Median: <seconds> s` at most 12.0 (`git -C
   ~/src/rust/<repository> log -1 --grep='^Step: <step>$' --format=%B
   origin/main | grep '^Median:'`).
5. The double spawn is off in every gate and in the queue: `grep -n
   NEXTEST_DOUBLE_SPAWN ~/src/rust/{skein,smith,jig}/scripts/check.sh
   ~/src/rust/skein/scripts/dev/heavy` prints a line in each.
6. skein's read of what has arrived is built and used:
   `heavy cargo nextest run -p skein-io-world -E
   'test(a_megabyte_on_a_real_pipe_arrives_in_few_deliveries)'` in skein,
   and `heavy cargo nextest run -p smith-machine-world -E
   'test(a_megabyte_of_output_takes_few_deliveries)'` in smith, each pass
   once on `main`.
7. The aims stated above are these items; none other is checked.
