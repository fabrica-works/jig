# test-budgets: decisions

The decisions this plan rests on, numbered; the supervisor's rulings
during the run follow them, from 101; the choices left to the user close
the file.

## The plan's own

1. **Scope** (the user, 2026-10-13: "the jig-protocol plan has
   completed, with few later items, among which: Test-cost passes on
   every suite whose budget was doubled (jig's quick suite, smith's),
   then the budgets restored; skein's raw-reader plan, option (a) ...
   which I'd both like to tackle now"). The focused suites of jig, smith
   and skein (whose timeout ruling 249 doubled too, skein `886a7d4`) made
   cheaper and their 15 s timeouts restored; skein's read of what has
   arrived, taken by smith's and jig's readers. Not in it: the fuzzy
   suites, which fit their 60 s; the repository downstream of jig
   (decision 12); the live suites.
2. **The aim: at most 12 s in a gate on carbon-forge** (the user,
   2026-10-13, "≤ 12 s (80%)"), each focused suite, with its
   `global-timeout` back at 15 s. The margin is for the next plans' tests
   and for load outside the queue (one measure of jig's suite on this
   plan's base took 20.1 s at a load of 8.7, against 15.0 s in its last
   gate). Each repository's last increment measures its suite three times
   in the gate lane and restores the timeout when the median is at most
   12.0 s.
3. **Measured on carbon-forge,** the slowest machine (the user,
   2026-10-13, jig's `jig-protocol/decisions.md`, "For the user": "15 s
   focused and 60 s fuzzy hold on carbon-forge"). Every time in this plan
   is carbon-forge's, through `heavy`, unoptimized as the gate builds.
4. **Algorithms first** (the user, 2026-10-13: "exploring how to make
   test runs more efficient (e.g. detecting+converting O(N) code to O(1)
   and things like that)"). The planner's audit (`findings/cost-*.md`)
   found each increment's cause by measuring it; no increment drops an
   assertion or a behaviour a test checks, and where production code
   changes, its behaviour stays the same. Where a test is made smaller,
   its session says why its assertion holds at the smaller size.
5. **The one-process fallback runs only if needed** (the user,
   2026-10-13: "Conditional last step"; earlier, "not now; only when it is
   needed to keep the suite within its budget"). If, after a repository's
   cost sessions, its timeout increment (skein 02.5, smith 04.1, jig
   06.1) measures a median past 12.0 s, it lands nothing and reports;
   the supervisor then asks the designer (development.md, 10.4) to change
   testing-strategy.md, 8 so that each crate's unit tests run in one
   process (libtest's threads) and only the worlds' tests one process
   each, and adds the step `<repository>-tests.one-process` that makes
   the change in that repository's gate. Measured on this plan's base:
   `jig-protocol-store`'s 417 unit tests run in 0.28 s in one process
   against 2.7 s as 417 nextest processes, alone.
6. **nextest without its double spawn** (`findings/cost-jig-units.md`,
   1; the designer, in this plan's design landing, testing-strategy.md,
   8). nextest starts each test through a second process of its own,
   which guards only a terminal's job control; turned off, a test's
   process costs about half. Measured on this plan's base, paired in the
   gate lane: 992 of jig's unit tests in 1.117 s and 0.951 s with it,
   0.493 s and 0.501 s without (`findings/measurements.md`, 3). Each
   repository's `scripts/check.sh` and skein's `heavy` set
   `NEXTEST_DOUBLE_SPAWN=0`; until skein 02.1 lands, each Proof's
   measuring command sets it in front of `heavy` (README, "Rules"), so
   every increment is measured as the gate will run it.
7. **The read of what has arrived** (the user, 2026-10-13: option (a)
   of "A raw reader over io takes a byte per iteration", jig's
   `jig-protocol/decisions.md`, "For the user"). Designed in skein's
   `lib.md`, 7, `io.md`, 2 and 3.3, `http.md`, 3.3, 4 and 6, `tls.md`,
   3.1, `programming-model.md`, 4.3 and `testing-strategy.md`, 2.4, landed
   with this plan. `Read::Arrived { max }`: met as soon as the side below
   holds a byte, by all it holds then, up to `max`.
8. **Which readers move** (`findings/reads.md`, 4): every reader of
   bytes nothing frames whose step already takes a whole delivery: in
   skein, the LLM client's error body, the OAuth token response, the
   fake peers' bodies, the tests' bodies; in smith, a tool's and git's
   output, a host's standard error, the headless prompt, the scripted
   person; in jig, the client link's event data, the native shell's
   bodies of unknown length, the engine's chunked request bodies. Exact
   reads stay where the bytes are framed or the step takes one event per
   byte: the JSON tokenizer, the HTTP body near a delimiter, the channel's
   header, an event's first data byte, the terminal's keys.
9. **smith's readers are proven by tests, not a benchmark probe**: what
   a model sees does not change, only the loop's iterations, which smith
   01's stories count. smith's `workflow.md`, 5 asks a probe of "an
   improvement (prompt caching, budgets, deadlines, tool round trips,
   limits sized for real work)"; a read's cost is none of those, and no
   probe measures a loop's iterations. Every smith landing that changes
   code still runs `smoke-tools` once. Listed for the user's veto.
10. **smith's gate here** (jig's `jig-protocol/decisions.md`, ruling 121):
    smith lands on carbon-forge without the focused suite's timeout, with
    `land --gate 'cargo fmt --all --check && cargo clippy --workspace
    --all-targets -- -D warnings && NEXTEST_DOUBLE_SPAWN=0 cargo nextest
    run --workspace --profile measure && NEXTEST_DOUBLE_SPAWN=0 cargo
    nextest run --workspace --profile fuzzy'`, until smith 04.1 restores
    its timeout; from then smith lands here with `scripts/check.sh`.
11. **smith's reliability plan lands in smith and skein meanwhile,** from
    debian-16gb-hel1-1. This plan's smith sessions rebase on it through
    `land`; neither waits for the other. Its sessions add tests to the
    suite this plan makes cheaper, measured on a machine about 1.7 to 2
    times faster (smith's focused suite: 9.2 to 9.8 s in that machine's
    gates; here 16.5 s in the `measure` profile on smith `6449a197`, 21.0
    s on `0305f2fd`, `findings/measurements.md`, 1).
12. **Nothing downstream of jig is repinned** (jig's
    `jig-protocol/decisions.md`, rulings 281 and 282): the one repository
    built on jig does not build on jig's `main`, and its reconciliation
    is a plan of its own. Its repin appears in no file of this plan
    (development.md, 5.7, "Names").
13. **The codec generator's changes reach smith and jig by regenerating**
    (as jig's `jig-protocol/tests-cost.md`, steps 3 and 9 did): a family's
    worst-case functions check its limits once, and its golden tables hold
    up to 1,024 cases; smith 01.1 and jig 05.1 take skein's codegen and
    regenerate every family, golden `.bin` files unchanged.

## Rulings

(The supervisor's, from 101.)

101. **skein 02.2 regenerates all four of `tests/codec`'s families**
     (plan defect, skein 02's block of 2026-10-13T23:15:24Z). 02.2's Files
     named only `tests/codec/src/generated/`, but `tests/codec/tests/drift.rs`
     checks four schemas against their generated code (`src/secrets/v1.rs`,
     `src/generated/v1.rs`, `src/scalars/v1.rs`, `src/lints/v1.rs`), so the
     generator's change fails three drift tests unless all four regenerate.
     They regenerate, once, with the drift test's own variable
     (`SKEIN_CODEGEN_WRITE=1 cargo nextest run -p skein-codec-tests --test
     drift`, through `heavy`); nothing in them is edited by hand, and the
     golden `.bin` files stay unchanged. 02.2's Files now names the four
     families. Generated code counts toward neither the Size nor the
     20-line allowance outside Files (development.md, 8.2).

## For the user

- **No benchmark probe for smith's readers** (decision 9): smith's
  workflow asks a probe of an improvement; the planner reads a loop's
  iterations as outside what a probe measures, and proves smith 01 by
  stories that count deliveries. Veto, and smith 01 gains a probe (which
  `benchmarks.md` would first have to design).

- **smith's suite here and the reliability run there** (decision 11):
  after smith 04.1 restores smith's 15 s timeout on carbon-forge, the
  reliability run's landings, gated on debian-16gb-hel1-1, can carry the
  suite past it here unseen. Options: (a) the reliability run's
  supervisor holds smith's focused suite to about 7 s on its machine
  (12 s here over the 1.7 ratio), as a rule of that plan; (b) a gate
  that bites on carbon-forge is worked on when it bites, as ruling 249
  did; (c) budgets per machine. The planner recommends (a). Nothing in
  this run waits on it.
