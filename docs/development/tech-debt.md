# Technical debt

What a first look at jig found while its client domain was being built
(2026-10-10, main at `77c77ec`), for a deeper pass later. It is a set of
findings and recommendations, not decisions. Each says what was seen,
how, and what would pay it back.

## 1. Overall

The work is sound:

- **No shortcuts are marked:** none of `TODO`, `FIXME`, `todo!`,
  `unimplemented!`, `dbg!` or `#[ignore]` in about 205,000 lines (crates
  114k, examples 18k, test kits 26k, worlds 47k).
- **The step code follows the strict subset** (programming-model.md,
  section 10). The state machines sampled (the fleet's attempts, the
  views' run text, the reads' pages) are exhaustive, and each state that
  cannot occur is named with its reason.
- **Values are not invented.** Where a value had no faithful source (a
  call's number, a proposal's maker, a first-seen clock), the work
  stopped and the gap went to the design, instead of being filled with
  a guess.
- **The worlds' tests assert behaviour.** They check outcomes,
  committed records and refusals across restarts and faults, mirror the
  design's stories, and replay their seeds.

The debts below are about cost, accretion, readability, and one
integration that is still unproven.

## 2. The client and the engine have not yet run together

The client has been tested against the scripted engine only. The joint
world, the client in front of the real engine, is the first meeting of
the two vocabularies. Its translation has already found several places
where the client departed from the contract:

- predicates the engine's reads do not have;
- a proposal's maker reduced to a number;
- resource names flattened to text;
- the views dropping a run's text while nobody watched it.

Expect another round of mismatches when its stories run. Until they
pass, there is no evidence that the client works against the engine.

**Recommendation:** treat the joint worlds' stories (the test
application's and `ops`'s) as the client's acceptance. Then check every
client carrier against `client/domain/contract.md` once more, field by
field.

## 3. Accretion

Building the client took 58 supplementary increments on top of the 97
planned ones, as the translation found gaps. Many of them kept
transitional code so that a change could land piecemeal: "legacy
consumers", compatibility constructors, adapters supplying `None` for a
field their source lacks. None of it is wrong today, but it piles up.

**Recommendation:** a consolidation pass once the joint worlds pass:

- remove the legacy and compatibility paths;
- collapse carriers that ended up duplicated;
- check that no adapter still manufactures a value.

## 4. Test cost

The suites are CPU only: each test runs on one thread, with no I/O
beyond loading the binary (measured with `strace` and `/proc`). Their
cost is spent mostly outside what they check. callgrind, unoptimized,
whole runs of single tests:

- **The client world's sweeps.** In
  `jig-client-world::fuzzy_client random_people_in_goal_starts`:
  - 79 % creating clients, 67 % of it rebuilding the English phrasebook
    for every client, tab, device and reload;
  - `Phrasebook::load` checks duplicates quadratically, and looks
    phrases up by scanning: `shape`, `phrase` and `capacity` in
    `crates/jig-client-view/src/phrases.rs`, and
    `Limits::of_words`/`compose`. The scans also run in the shipped
    client, on every `say`;
  - skein's counting allocator spends 34 % on bookkeeping
    (`skein_heap::Highs::merge` on every allocation).
- **The tasks world's lifecycle sweep.** 67 % is a coverage check that
  searches `Debug` dumps for text markers, on all 256 seeds, for markers
  already found. 10–15 % more formats those dumps as events happen,
  though they are read only on failure.
- **Conformance and `ops`'s alert story** replay whole stories once per
  cut, so their cost grows with the square of the number of commits.
  They recompute worst-case sizes on every iteration, and format every
  input and output.
- **The client harness's view limits** are envelope-sized, not
  test-sized. Every callback reserves gigabytes of virtual memory
  (about 8,500 large `mmap`s per 4-second test).
- **Seed counts exceed their assertions:** the lifecycle sweep runs 256
  seeds over 48 branch combinations.

Building the tests at `opt-level=1` made the suites 3 to 8 times faster,
but a cold build took 7.5 minutes against 38 seconds. Builds stay
unoptimized: slow tests are improved, and the budgets (15 s focused,
60 s fuzzy) stay.

**Recommendations** (being taken up by the client plan's session jig 47,
`~/src/plans/jig-client-implementation/jig/47-test-cost.md`):

- the phrasebook looked up by id, and built once in tests;
- checks run when what they check changed;
- traces kept typed and formatted only on failure, with coverage
  recorded as typed facts;
- worst-case sizes computed once per harness;
- test-sized limits;
- seeds sized to their assertions;
- long sweeps split into tests of about 5 s each, so the gate's threads
  share them.

These become rules in `workflow.md`.

## 5. Checks that prove less than they appear to

- **The heap check in the client sweeps cannot fail.** With
  envelope-sized view limits its bound is near 10¹¹ bytes. A memory
  bound that is checked should come with a test showing it catching a
  regression.
- **Coverage by text search over `Debug` output** passes as long as some
  formatted field happens to contain the marker. A typed record of what
  was reached says the same thing exactly.

## 6. Readability of the worlds' tests

The worlds' tests are correct but dense. Scenarios are written in bare
numbers: request numbers, task numbers, connectors and kinds, as in
`world.call(1, 9201)` or `ask(&mut world, 43, …)`. A reader has to work
to see the story the test tells.

**Recommendation:**

- name a world's fixtures (the parties, tasks and connectors a story
  uses);
- give the common acts helpers named for the design's verbs;
- keep the number only where the number is the point.

## 7. Signals in the step code

- **218 `unreachable!` arms in `crates/`,** clustered in
  `jig-domain-host/src/hosted.rs` (21), `jig-domain-fleet/src/attempt.rs` (17),
  `jig-domain-core/src/routing/tasks.rs` and `effects.rs` (11 each), and
  `jig-domain-notes/src/read.rs` (10). Most are honest invariants of state
  machines. Some come from matching two values whose pairing the types
  do not encode, such as a read and the document answering it
  (`jig-domain-core/src/reads/rows.rs`, `live_page`). Those are candidates for
  types that carry the pairing.
- **147 functions are excused from `too_many_lines` or
  `too_many_arguments`** in `crates/` and `examples/`. Several are
  exhaustive seams, kept whole on purpose. The rest should be checked
  for a split.
- **Large files:**
  - `jig-client-view/src/phrases/jig.rs`: 2,821 lines;
  - `jig-domain-people/src/domain.rs`: 1,873;
  - `examples/ops/domain/src/route.rs`: 1,777;
  - `jig-domain-tasks/src/boundary.rs`: 1,776;
  - `jig-domain-core/src/reads/rows.rs`: 1,643.

  Size alone is not debt. These files are worth a look in the deeper
  pass.

## 8. Process cost

Building the client, the evidence each merge carried cost more machine
time than the tests themselves:

- three serial runs of the touched binaries;
- the workspace's serial totals;
- the gate.

That evidence is now the gate alone, with the workspace's serial totals
once per session, a slowest-test report, and a warning past four fifths
of a budget. If a later plan reaches for heavier evidence again, profile
first. The cost is usually in one place.

## 9. Integration came last

The client plan built the client against the scripted engine. It made
the joint world's translation the only place where the client's and the
engine's vocabularies meet, and scheduled it near the end, after the
client's world and its view. That let both sides go on in parallel while
the engine's watches, reads and people's requests were still being
built. `client/domain/testing.md` points the same way: the client's
world "comes first", and the joint world "is the check".

The cost:

- **The scripted engine agrees with the client by construction.** It is
  written in the client's vocabulary.
- **Nothing showed drift on either side** (section 2) until the
  translation was written. Then it came at once, as a burst of design
  questions and supplementary increments (section 3).

A full joint world needed both sides far along, but two things did not:

- **the translation,** exhaustive both ways, started as soon as both
  sides had types. Every increment that adds a variant on either side
  extends it, so drift is a build error the day it happens;
- **one thin story end to end,** such as signing in and starting a
  chat, as soon as the engine can answer it, and extended by each later
  increment.

The fake also needs keeping honest: **the scripted engine's scenarios
run against the real engine too.** That way the fake cannot agree with
the client where the engine does not.

**Recommendation:**

- Integrate a client from its first increment: jig's protocol layer,
  `ops`'s, and temper's client.
- In `client/domain/testing.md`, the joint world runs from the start,
  and the scripted engine is checked against the engine.
