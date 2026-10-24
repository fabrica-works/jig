# Technical debt

What a first look at jig found while its client domain was being built
(2026-10-10, main at `77c77ec`), for a deeper pass later.
Refreshed 2026-10-18, main at `86971be`.

It is a set of findings and recommendations, not decisions. Each says
what was seen, how, and what would pay it back.

The refresh measured every finding again on `main`, 1,507 commits later
(`git log --oneline 77c77ec..86971be | wc -l`), and takes in two more
sources:

- **What jig's `STATUS.md` listed as the client plan's code debts** until
  2026-10-18: legacy and compatibility paths, worlds' tests in bare
  numbers, `unreachable!` arms, files of 1,600 to 3,000 lines. This file
  alone holds them now, in sections 3, 6 and 7.
- **The debts that the retrospectives of jig-protocol, test-budgets,
  client-composition and skein-ui left for this file**
  (`docs/development/plans/<plan>/retrospective.md`, section 9): in
  sections 4 to 7 where they are of those kinds, and in sections 10 to
  13. Section 14 says what was not taken in.

The client's crates have been renamed since (`docs/design/README.md`,
section 15). `jig-domain-client` is gone: its client half is
`jig-client-domain`, under `jig-client`, and its pages joined
`jig-client-view`, which was `jig-view`. Paths below are today's; a path
that starts with a crate's name is under `crates/`.

**Repeating the measures.** Commands run at the repository's root. Two
shell arrays select production code, the code of `crates/` and
`examples/` without unit tests or generated codecs:

```sh
PROD=(-g '!**/tests/**' -g '!**/tests.rs' -g '!**/tests_*.rs' -g '!**/generated/**')
FPROD=(-not -path '*/tests/*' -not -name tests.rs -not -name 'tests_*.rs' -not -path '*/generated/*')
```

Earlier figures, for `77c77ec` and for `d91a89c` (jig-protocol's end),
come from the same commands run on those trees (`git worktree add
--detach <dir> <commit>`).

## 1. Overall

The first look found the work sound, and the refresh changes little of
that verdict:

- **No shortcuts are marked:** still none of `TODO`, `FIXME`, `todo!`,
  `unimplemented!`, `dbg!` or `#[ignore]` (`rg -c -t rust
  'TODO|FIXME|todo!|unimplemented!|dbg!|#\[ignore'` prints nothing), now
  in about 457,000 hand-written lines: crates 252k, examples 26k, test
  kits 62k, worlds 117k, against 205,000 then (`find <dir> -name '*.rs'
  -not -path '*/generated/*' | xargs cat | wc -l`). Generated codecs add
  234,000 lines (section 12).
- **The step code follows the strict subset** (programming-model.md,
  section 10). The first look sampled its state machines; the refresh
  did not sample again. What a search can see is in section 7.
- **Values are not invented,** with one exception of the opposite kind:
  the clients' component drops a value the engine has, and sends eight
  wire fields that no engine value fills (section 3).
- **The worlds' tests assert behaviour,** and the joint worlds now hold
  the client to the engine (section 2).

The debts below are about cost, accretion and readability. The one
integration the first look found unproven now runs.

## 2. The client and the engine have not yet run together

**Resolved.** The client runs in front of the engine's real domain in
four worlds:

- **the joint world,** `tests/client/src/joint/`: the client before
  jig's core under its test root, through the clients' component's
  translation with no transport between. 257 tests (`rg -c '#\[test\]'
  tests/client/src/joint tests/client/tests/fuzzy_joint.rs`), 126 of them
  in `joint/wire/`, which compare each carrier with its engine source
  field by field: the check against `client/domain/contract.md` that the
  first look recommended;
- **the protocol world,** `tests/client-protocol`: the same over native
  HTTP bytes, 60 tests;
- **`ops`'s joint world,** `tests/ops/src/joint/`: `ops`'s client before
  its whole engine on the conformance world, in
  `tests/ops/tests/fuzzy_joint.rs` (11 tests) and `client_stories.rs`
  (7);
- **the browser tier,** `tests/client-browser`: Chromium against
  `jig-test`, 7 tests.

What stays open of the first look's advice is in section 9: no story
runs against both the scripted engine and the engine.

## 3. Accretion

At `77c77ec`, building the client had kept transitional code so that a
change could land piecemeal: "legacy consumers", compatibility
constructors, adapters supplying `None` for a field their source lacks.

**Today:**

- **Named transitional paths are gone from jig's crates.** `rg -i -n -t
  rust 'legacy|compat'` finds 17 lines: test data, the boundary world's
  own check, comments on version compatibility, and error strings of
  `ops`'s worlds (below). The boundary world now refuses such names:
  `forbidden_name`, in `tests/boundary/src/names.rs`, fails an
  identifier with a `Legacy`, `Typed` or `Untyped` part, or a version
  suffix outside the codecs, and "legacy" in a comment.
- **`ops`'s client keeps two vocabularies for its families.** Numbered
  identities (`Kind::Environment(u64)`, `Act::MoveEnd`, `Ask::MoveEnd`)
  stand beside named ones (`Kind::NativeEnvironment`,
  `Act::NativeMoveEnd`, `Ask::NativeMoveEnd`), in
  `examples/ops/client-domain/src/infrastructure.rs` and
  `infrastructure/documents.rs`. 58 of the 184 records and enums of
  `examples/ops/client-protocol-wire/schema/ops-client-v1.schema` are
  `InfrastructureNative*` (`rg -c '^(record|enum) InfrastructureNative'`
  on it). Each family has a wire module per vocabulary
  (`examples/ops/client-domain/src/wire/`): infrastructure 1,376 and
  1,098 lines, observability 560 and 542. The joint world refuses the
  numbered ones, as "legacy numeric identity is absent from the native
  owner" (`tests/ops/src/joint/front.rs:130`, `infrastructure.rs:398`),
  while the scripted world still keys cards by number
  (`tests/ops/src/client/app.rs:387`).
- **The clients' component leaves wire fields empty**
  (`jig-protocol-engine-clients/src/translate/refusals.rs`):
  - `p::Lack::TaskKind { .. }` drops the engine's `executor` and sends
    `executor: None` (:84);
  - a depth's and a task count's `needed`, and a deadline's `asked` and
    `latest`, are always `None` (:88, :92, :95): the engine's `Lack`
    carries none of them;
  - an ending's `reason` when failed or cancelled, and its `party` and
    `how` when decided, are always `None` (:221, :225, :229).

  No other path fills these fields (`rg -n
  '\b[a-z]+::(LackingDeadline|LackingDepth|LackingTasks|LackingTaskKind|EndingDecidedFields|EndingFailedFields|EndingCancelledFields)Parts
  \{' crates examples "${PROD[@]}"` finds only these lines).

**Recommendation:** the consolidation pass the first look proposed, now
that the joint worlds pass:

- `ops`'s client on one vocabulary, the named one its engine speaks, and
  its scripted world with it;
- the engine's `executor` carried; for each wire field no engine value
  fills, either the engine gives it or the wire drops it;
- a last check that no adapter manufactures a value.

## 4. Test cost

The suites are CPU only. At `77c77ec`, callgrind showed their cost
spent mostly outside what they check. The first look's
recommendations are now rules, `workflow.md`, section 2: fixtures once,
checks on change, traces on failure, test-sized limits, seeds sized to
their assertions, long runs as several tests. Against them:

- **The phrasebook: resolved.** `Phrasebook` is skein's now
  (`skein-ui`'s `phrases.rs`): `load` sorts and checks neighbours for
  duplicates, and `say` and `capacity` look phrases up by binary search.
  The client world builds each book once per process (`phrasebooks`,
  `tests/client/src/test.rs:121`), and the view kits share theirs through
  a `OnceLock`. One place still builds a fresh English book per call:
  `tests/client/src/joint/offered_acts.rs:13`.
- **The tasks world's lifecycle sweep: resolved.** Coverage is typed
  (`Reached` and `MARKERS`, `tests/tasks/src/coverage.rs`). The sweep
  enumerates the 48 story choices and draws 8 seeds of 256, instead of
  running all 256 (`tests/tasks/tests/fuzzy_lifecycle.rs`). The trace is
  off unless `JIG_WORLD_TRACE` is set (`tests/tasks/src/trace.rs`), as
  `workflow.md`, section 3, says; when on, it is still formatted text.
- **Conformance and `ops`'s alert story: in part.** Worst cases are
  computed once per harness (`testing/jig-conformance/src/harness.rs:242`),
  and inputs and outputs are formatted only when tracing. Cuts still
  cost the square of the number of commits:
  - `tests/conformance/tests/fuzzy_cuts.rs` replays the story from the
    start once per commit, for 4 seeds;
  - `fuzzy_snapshot.rs`, for each cut, resumes from a snapshot, replays
    the same cut fresh with tracing on, and compares the two as `Debug`
    dumps of the whole world (`replay::outside`,
    `tests/conformance/src/replay.rs:190`);
  - `fuzzy_scenarios.rs` resumes each cut from a cold snapshot, which
    saves the prefix but runs the rest of the story per cut
    (`replay.rs:124-160`);
  - `ops`'s alert story now takes 6 chosen cuts, one test each, and each
    runs the baseline again (`tests/ops/tests/fuzzy_alert_cuts.rs`).
- **The client harness's view limits: resolved.** They are test-sized:
  `Limits::for_world` with 512 or 1,024 nodes and 256 slots
  (`tests/client/src/test.rs:147`;
  `testing/jig-client-test-view/src/limits.rs`). `ops`'s client takes 512
  nodes (`tests/ops/src/client/app.rs:114-122`).
- **skein's counting allocator** is installed by 44 test binaries (`rg
  -l global_allocator -t rust tests testing crates examples | wc -l`).
  Its bookkeeping is skein's; the refresh did not profile it again.

New of the same kind:

- **The core world's observer** rebuilds its whole snapshot from every
  stored row whenever a commit applied something (`Observer::durable`,
  `tests/core/src/observations.rs:588`).
- **The fleet world formats every event,** request and stimulus as it
  happens, though the trace is read only on failure (`format!` at
  `tests/fleet/src/world.rs:649,657,759` and `turns.rs:200,207`).
- **The focused suite's budget is doubled.** `.config/nextest.toml`
  gives it a `global-timeout` of 30 s (:10), against the 15 s that its
  header (:1-3) and `workflow.md`, section 2, state; a comment (:9) ties
  the doubling to one run, until a test-cost pass. Of the last 50 gated
  landings, 8 ran the focused suite past 15 s, the longest in 18.7 s; the
  fuzzy suite took 15.4 to 34.4 s of its 60 (`git log
  --format='%(trailers:key=Gate-Result,valueonly)' 77c77ec..86971be |
  grep passed | head -50`).

Building the tests at `opt-level=1` made the suites 3 to 8 times faster,
but a cold build took 7.5 minutes against 38 seconds. Builds stay
unoptimized, except the repository's tools (`workflow.md`, section 2).

**Recommendations:**

- cuts resumed from snapshots, compared as typed state, never as `Debug`
  dumps; one fresh replay per story, not per commit;
- the core world's observer updated from each commit's rows;
- the fleet world's trace kept typed and formatted only on failure, or
  switched off as the tasks world's is;
- the focused suite back under 15 s, and its `global-timeout` with it.

## 5. Checks that prove less than they appear to

- **The heap check in the client sweeps: resolved.** Each client's bound
  is its worst case, capped at 16 MiB (`heap_bytes_per_client`,
  `tests/client/src/harness/world.rs:47,190,202`), and two tests show it
  catching a client past a 1,024-byte budget
  (`tests/client/tests/memory.rs:7-19`).
- **Coverage by text search over `Debug` output: in part.** The tasks
  world records coverage as typed facts. Ten assertions in six files
  still find what happened by searching formatted trace lines, such as
  counting a store's loads as the lines containing `"out Load"`:
  `tests/client/src/joint/people_stories.rs:175`,
  `joint/stories/fault_stories.rs:233,340`,
  `tests/notes/tests/simulation.rs:124,130`,
  `tests/core/tests/store.rs:58,256`, `restore_pages.rs:126,132` and
  `restart.rs:130` (`rg -n -t rust
  '\.(contains|starts_with|ends_with|matches)\("[^"]*[A-Za-z]{3,}' tests
  testing | rg -v 'Find::text|b"'`, 80 lines, read one by one). A typed
  record of what was reached says the same thing exactly.
- **The store's drift check sees only what codegen names.**
  `tests/store-codecs/tests/drift.rs:24-31` compares the files codegen
  produces with a golden directory, so a stale file in a directory it
  reads goes unnoticed, and a directory it does not read, such as
  `core/v1`, is never looked at (section 12).

## 6. Readability of the worlds' tests

At `77c77ec`, scenarios were written in bare numbers: request numbers,
task numbers, connectors and kinds, as in `world.call(1, 9201)` or
`ask(&mut world, 43, …)`. A reader had to work to see the story a test
tells.

**Today** the older worlds still read so, and the newer tests name their
numbers, file by file:

- **Bare numbers grew with the worlds.** Calls with two or more bare
  integers, the first of two digits or more: 137 in the worlds, against
  106 then (`rg -o -t rust
  '\b[a-z_]+\((&mut [a-z_]+,\s*)?[0-9]{2,}(_[0-9]+)?,\s*[0-9]+' tests |
  wc -l`). `world.call(1, 202)` and its like are still in
  `tests/core/tests/effects.rs`; `ask(&mut world, <n>, …)` went from 41
  calls to 66, most in `tests/core/tests/stories.rs` (17) and `reads.rs`
  (15) (`rg -c -t rust '\bask\(&mut world,\s*[0-9]+' tests`).
- **Names are per file.** The worlds name 314 integer constants, against
  10 then (`rg -o -t rust '^\s*(pub(\(crate\))? )?const [A-Z_0-9]+:
  (u8|u16|u32|u64|usize|i64)' tests | wc -l`), but each file names its
  own: `PROJECT` is defined in 17 files, `PARTY` in 10, `TASK` and
  `REQUESTER` in 8 (`rg -o --no-filename -t rust -r '$1'
  '^\s*(?:pub(?:\(crate\))? )?const ([A-Z_0-9]+):' tests | sort | uniq -c
  | sort -rn`).
- **Setup outweighs some tests.** In
  `jig-client-view/src/tests/task_proposals.rs`, two tests of 9 and 10
  lines (:379, :389) share 86 lines of fixture (`leaving_plan` and
  `committed_proposal`, :292 to :377).
- **A helper dispatches on strings:** `special`, in
  `tests/client/tests/text_pages.rs:68-114`, picks one of five pages by
  name, with an `unreachable!` arm (:111), where an enum would name them.

**Recommendation:**

- name a world's fixtures once, in the world (the parties, tasks and
  connectors a story uses), not in each file;
- give the common acts helpers named for the design's verbs;
- keep the number only where the number is the point;
- turn groups of tests whose bodies differ only in their data into table
  tests (jig-protocol's retrospective, section 9, lists where many tests
  were added at once).

## 7. Signals in the step code

- **`unreachable!` arms.** 278 in `crates/` and `examples/`, against 218
  then (`rg -o 'unreachable!' crates examples | wc -l`); 228 in
  production code, against 185 (`rg -o 'unreachable!' crates examples
  "${PROD[@]}" | wc -l`). They cluster in
  `jig-domain-host/src/hosted.rs` (20), `jig-domain-fleet/src/attempt.rs`
  (17), `jig-domain-core/src/routing/tasks.rs` (14),
  `examples/ops/domain/src/route.rs` (13), `jig-domain-core/src/effects.rs`
  (12) and `jig-domain-notes/src/read.rs` (10) (`rg -c 'unreachable!'
  crates examples "${PROD[@]}" | sort -t: -k2 -rn`). Most are honest
  invariants of state machines. Some come from matching two values whose
  pairing the types do not encode, such as a read and the document
  answering it (`jig-domain-core/src/reads/rows.rs:168`, `live_page`).
  Others are arms for variants that cannot arrive:
  `jig-protocol-accounts/src/translate.rs` (7, from :40 to :98, for
  variants the engine never meets) and, in a test kit,
  `testing/jig-test-worker/src/domain.rs:285,431`. Those are candidates
  for types that carry the pairing, or that leave the variants out.
- **`panic!` in production code:** 4, all in the native shell, which is
  ordinary Rust: `jig-client-shell-native/src/files.rs:280`,
  `shell.rs:127,179` and `connections.rs:116` (`rg -n 'panic!' crates
  examples "${PROD[@]}"`; 2 at `d91a89c`). Each guards a request or an
  event its caller should never send, which a narrower type would rule
  out.
- **Lint exceptions in production code:** 399 lints named in
  attributes, against 198 then and 359 at `d91a89c` (`rg -o -t rust
  'clippy::[a-z_]+' crates examples "${PROD[@]}" | wc -l`):
  - `manual_map`, 201 of them (28 then): one `#[expect]` per function,
    each giving the subset's reason, no closures. A lint the subset
    always breaks is better allowed once, where the subset applies, with
    that reason;
  - `too_many_lines` and `too_many_arguments`, 150 (141 then; `rg -o
    'too_many_lines|too_many_arguments' crates examples "${PROD[@]}" |
    wc -l`), most in `examples/ops/domain/src/route.rs` (8),
    `jig-domain-core/src/routing/party_controls.rs` (8),
    `party_decisions.rs` (6) and `effects.rs` (6). Several are
    exhaustive seams, kept whole on purpose. The rest should be checked
    for a split;
  - `struct_excessive_bools`, 13, 8 of them in the worlds (`rg -n
    'struct_excessive_bools' tests testing crates examples`; 15 at
    `d91a89c`). A struct of flags is often an enum of states.
- **Long functions:** 404 production functions over 60 lines, against
  250 then and 390 at `d91a89c`. A function runs from its `fn` line to
  the closing brace at its indentation, rustfmt's layout:

  ```sh
  find crates examples -name '*.rs' "${FPROD[@]}" -print0 | xargs -0 awk '
    FNR==1 {n=0}
    !n && /^ *(pub(\([a-z]+\))? )?(const )?fn / && !/[;}] *$/ {
      match($0,/^ */); e=substr($0,1,RLENGTH) "}"; s=FNR; n=1; next }
    n && $0==e { if (FNR-s+1>60) c++; n=0 }
    END {print c}' | awk '{s+=$1} END {print s}'
  ```

  `jig-domain-core` has 99 of them, `jig-client-view` 88,
  `jig-domain-tasks` 48, `jig-protocol-store` 27 and
  `jig-protocol-engine-clients` 23. The longest are phrase tables
  (`jig-client-view/src/phrases/english.rs:12`, `language`, 1,452
  lines; `phrases/jig.rs`, `shape` and `id`, 641 and 610) and
  `jig-domain-core/src/routing/tasks.rs:20`, `tag_tasks`, 1,385 lines.
  The translations jig-protocol's retrospective named are past 100 lines
  still: `jig-protocol-store/src/codec/people/asks.rs`, `encode_ask`
  (170, :297) and `decode_ask` (127, :469);
  `jig-client-protocol-link/src/translate/asks.rs:553`, `ask` (120);
  `jig-protocol-engine-clients/src/translate/asks.rs:8`, `ask` (107).
  Smaller: `show`, in `jig-client-view/src/page/shown.rs`, holds a
  27-line inline `fits` match (:178-204), a function of its own.
- **Long tests:** 637 of 4,341 tests are over 40 lines (613 of 3,814 at
  `d91a89c`), counting a `fn` within four lines below a `#[test]`:

  ```sh
  find crates examples testing tests -name '*.rs' -not -path '*/generated/*' -print0 | xargs -0 awk '
    FNR==1 {n=0; t=-9}
    /#\[test\]/ {t=FNR}
    !n && FNR-t<=4 && /^ *(pub(\([a-z]+\))? )?fn / {
      match($0,/^ */); e=substr($0,1,RLENGTH) "}"; s=FNR; n=1; next }
    n && $0==e { k++; if (FNR-s+1>40) c++; n=0 }
    END {print c, k}' | awk '{c+=$1; k+=$2} END {print c " of " k}'
  ```

  The longest:
  `jig-domain-core/src/tests/activity.rs:50` (189 lines),
  `tests/client/src/joint/engine_stories.rs:129` (156),
  `tests/core/tests/activity.rs:31` (148). Of those jig-protocol's
  retrospective named, `tests/client/src/joint/stories/fault_stories.rs`
  now has five tests over 40 lines.
- **Large files:** 66 hand-written files over 800 lines, against 48 then
  and 64 at `d91a89c`; 12 of 1,600 lines or more, against 10 (`find
  crates examples testing tests -name '*.rs' -not -path '*/generated/*'
  -exec wc -l {} + | awk '$2 != "total" && $1 >= 1600' | wc -l`). The
  largest:
  - `jig-domain-core/src/tests.rs`: 2,974 lines;
  - `jig-client-view/src/tests/domain/pages.rs`: 2,927;
  - `jig-client-view/src/phrases/jig.rs`: 2,902 (2,821 then);
  - `testing/jig-client-scripted-engine/src/tests.rs`: 2,257;
  - `jig-client-view/src/tests/cards_objects.rs`: 2,042;
  - `tests/core/tests/stories.rs`: 1,871;
  - `jig-domain-tasks/src/boundary.rs`: 1,857 (1,776);
  - `jig-domain-people/src/domain.rs`: 1,818 (1,873);
  - `examples/ops/client-view/src/tests.rs`: 1,760;
  - `jig-domain-core/src/reads/rows.rs`: 1,654 (1,643);
  - `tests/client/src/joint/session.rs`: 1,624;
  - `jig-domain-people/src/tests.rs`: 1,605.

  The files past 800 lines that the retrospectives saw grow kept growing:
  `jig-domain-core/src/routing/tasks.rs` 1,586 (1,449 at `d91a89c`),
  `tests/client/src/joint/session.rs` 1,624 (1,555 at skein-ui's end),
  `jig-client-view/src/sections/project/owner.rs` 1,212 (1,202),
  `tests/ops/src/conformance/mod.rs` 1,104 (1,079),
  `tests/client/src/harness/world.rs` 978 (955),
  `tests/client/src/stories.rs` 975 (943). `examples/ops/domain/src/route.rs`
  shrank to 1,596 (1,777 then), and `jig-domain-client/src/domain.rs`
  went with its crate. Size alone is not debt. These files are worth a
  look in the deeper pass.
- **Code cites plans, not the design:** 8 comments cite a plan's rulings
  or decisions (`rg -n -i --hidden '\bruling|decisions\.md|\bdecision
  [0-9]' --glob '!docs/**' --glob '!.git/**' .`):
  `jig-domain-tasks/src/index.rs:167`,
  `jig-protocol-config/src/llm.rs:354`,
  `tests/client-protocol/src/native/world.rs:95`, four comments of
  `jig-client-protocol-wire/schema/client-v11-part1.schema` and
  `.config/nextest.toml:9`. A reader finds the rule in the design
  section a ruling applies, not in the ruling.

## 8. Process cost

**Resolved.** Building the client, the evidence each merge carried cost
more machine time than the tests themselves: three serial runs of the
touched binaries, the workspace's serial totals, and the gate. The
evidence is now the gate alone, which skein's `land` runs and records in
the landing's `Gate-Result` trailer (skein's development.md, section 9).
The last gated landing, `4e1ab92b`, ran 4,250 focused tests in 15.8 s,
111 fuzzy ones in 23.2 s and one browser test in 1.3 s, 68 s in all
(`git log -1 --format=%B 4e1ab92b | grep Gate-Result`). If a later plan
reaches for heavier evidence again, profile first. The cost is usually
in one place.

## 9. Integration came last

The client plan built the client against the scripted engine, and met
the engine only near its end: the drift came at once (sections 2 and 3).
skein's development.md, section 5.3 ("The meeting point comes first"),
now asks every plan for what this section recommended: one thin story
end to end as soon as the far side can answer, a translation exhaustive
both ways from the first types, and a fake's scenarios run against the
real side.

**What remains:**

- **`client/domain/testing.md` still orders them as before:** "The
  client's world comes first", and "The joint world is the check"
  (section 1).
- **The scripted engine is checked against nothing.**
  `testing/jig-client-scripted-engine` is 15,559 lines (`find
  testing/jig-client-scripted-engine -name '*.rs' | xargs cat | wc -l`).
  No story runs against both it and the engine: the joint world's
  stories are its own (`tests/client/src/joint/stories.rs` and
  `stories/`), and the two fronts share only the native transport's
  input (`tests/client/src/test/native.rs`). `ops`'s scripted world and
  its joint world speak different vocabularies (section 3), and the
  client world's fronts handle a sign-out unlike the link (section 13).

**Recommendation:**

- In `client/domain/testing.md`, the joint world runs from the start,
  and the scripted engine is checked against the engine.
- The client world's stories run on both fronts, the scripted engine's
  and the joint world's, so that the fake cannot agree with the client
  where the engine does not.

## 10. Duplicated code

- **The piece-decoding loop,** which splits a document's
  length-prefixed pieces, bounds their sum and decodes each row, is
  written 50 times in 22 production files, against 50 in 20 at
  `d91a89c` (`rg -c 'piece::split\(&mut reader\)' crates examples
  "${PROD[@]}"`); 48 of them in `jig-client-protocol-link/src/translate/`.
  That directory is split by number to keep files small:
  `documents_part0` to `8`, `pages_part0` to `8` and `facts_part0` to `2`
  hold 6,097 of its 10,048 lines (`cat
  crates/jig-client-protocol-link/src/translate/*_part*.rs | wc -l`), and
  the link's test fixtures add `tests/fixtures/reads_part0` to `7`. One
  reader generic over the row would replace the copies; jig's step code
  defines no generic items, so it belongs in skein, or in what skein's
  generator emits with each record. The numbered files then shrink with
  it.
- **`skein_ui::Limits` written out by hand:** 34 literals in 31 files,
  each with `scene_marks: 0` and `scene_bytes: 0`, and no constructor
  (`rg -U -c '(skein_ui::)?Limits \{[^{}]*?scene_marks: 0' -g '*.rs'
  crates tests testing examples`); for example
  `jig-client-view/src/limits.rs:75`, `owner_deployment.rs:677`,
  `owner_result.rs:121` and `tests/client/src/model.rs:177`. One
  constructor for a tree without scenes would replace them.
- **skein-ui's node accounting, again:** `text_bytes`, in
  `jig-client-view/src/tests.rs:70-85`, re-implements skein-ui's private
  accounting for five test files (`rg -n 'fn text_bytes|tests::text_bytes'
  crates/jig-client-view/src`). It drifts when skein-ui's does; skein-ui
  exposing what it counts would end it.
- **The section context, a hidden parameter.** `Ui`'s `section_context`
  (`jig-client-view/src/ui.rs:72`) is set before a call and reset to
  `u32::MAX` after it, in `jig-client-view/src/page/shown.rs`: around
  `open_section` (:239-241), around `task::document` (:246-249), and
  before `routing::refuse` (:223). The loop that sets its bits is
  written three times: `context_subject` (:330), `context_section`
  (:339) and `context_read`'s project arm (:389) (`rg -n
  'section_context|context_read|context_subject|context_section'
  crates/`). Passed as an argument, and built by one function, it would
  say what each call reads.

## 11. Fixtures that move with the code

- **Four copies of smith's goldens:** the transcript files
  `jig-protocol-transcript/tests/data/record_turn_full.bin` (146 bytes)
  and `record_turn_smallest.bin` (42); `FULL_TURN`, in
  `testing/jig-client-test-records/src/activity_reads.rs:4`; and
  `SMALLEST`, smith's smallest charter, in
  `testing/jig-test-worker/src/tests/fixtures.rs:62`. Literals derive
  from their sizes in `tests/client/src/joint/wire/turns.rs:24,57,60`,
  `activity.rs:381` and `jig-client-protocol-link/src/tests/pages.rs:242`.
  They equal smith's goldens at the pinned `a277e922` today (`cmp`
  against smith's `crates/smith-transcript/golden/v5/` and
  `crates/smith-charter/golden/v2/` in cargo's checkout of that
  revision), but nothing checks it, and the copies' README names
  `a277e922`. Every repin of smith refreshes them by hand. A drift test
  against the pinned crate's goldens, or reading them from it, would pay
  it back.
- **The one-process world's power-loss cut is placed by a count:**
  `crash(skein_world::Cut::PowerLoss, 128)`, the 128th submission
  (`tests/one-process/tests/one_process.rs:38`); its fuzzy variant draws
  75 to 133 (`fuzzy_one_process.rs:9`). Any change in how many steps a
  run takes moves it. Placed on an event, the first committed `Turn`
  with a live worker, as the signal stories already place theirs
  (`tests/one-process/src/referee.rs:280-293`), it would stay where its
  story needs it.

## 12. Generated and unread files

- **Generated volume:** 233,576 lines in 11 files (`find . -path ./target
  -prune -o -path '*/generated/*' -name '*.rs' -print | xargs wc -l`),
  106,267 of them `jig-client-protocol-wire/src/generated/v11.rs`, which
  was 99,451 lines as v7 at `d91a89c`. The goldens are 7,921 files (`find
  . -path ./target -prune -o -path '*/golden/*' -type f -print | wc -l`).
  Their build cost has not been measured: `cargo build -p
  jig-client-protocol-wire --timings`, with kache bypassed
  (`CARGO_BUILD_RUSTC_WRAPPER=`), would.
- **The fake store** (`testing/jig-fake-store`, 329 lines) stays for
  `ops` alone: `testing/jig-conformance` implements its `Durable` for it
  (`src/harness.rs:27`), and `tests/ops` uses it. `ops`'s worlds on the
  real store would retire it.

## 13. Behaviour against the design

Where the code does less than its design says, or the worlds' fronts
less than the code they stand for:

- **The composer** is built only in the confirmation's dialog
  (`jig-client-view/src/confirmation.rs:241-247`, the `ActKind::Message`
  arm), where `client/domain/pages.md`, 12.2, places it in an agent's
  activity, below the thread.
- **The project's overview** opens an inbox watch and sets the inbox's
  filter, but asks no inbox read (`jig-client-view/src/page/opening.rs:63-71`).
  Only the inbox's section asks one (`start_group`,
  `page/conversations.rs:312-325`).
- **Drawings are acted on in their place only:** every owner's binding
  maps `Act::Drawing` to `None` (`jig-client-view/src/binding.rs:206-269`,
  `examples/ops/client-view/src/bindings.rs:82,95,108`,
  `testing/jig-client-test-view/src/bindings.rs:84,97` and
  `root.rs:222`), against `client/domain/view.md`, sections 5 and 16.
- **`pages::owed`,** the owed cards' function in `pages.md`, 12.1, and
  `view.md`, 4.1, does not exist: `after_page` builds them inline
  (`jig-client-view/src/pages.rs:180-203`).
- **An owner's number under other names:** `Part`'s `connector`, in
  every variant (`jig-client-domain/src/contract/common.rs:185-207`;
  `OwnerPart`, `contract/owner_part.rs:41`; the wire's `client-v11.schema`,
  :62-73 and :107), and the `family` fields of
  `jig-client-domain/src/boundary.rs` (:7, :12, :36, :41) and
  `jig-client-protocol-link/src/component.rs` (:210, :249). They read
  wrong as soon as an owner that is no connector names a part.
- **Recovery after a power loss between a worker's clean exit and its
  `Finished` commit** is checked by no test: the one-process world's
  referee counts only finished and lost claims
  (`tests/one-process/src/referee.rs:116-143`), while the core maps the
  fleet's `Refused`, `NotStarted` and `Withdrawn` alike to a refused
  ending (`jig-domain-core/src/routing/children.rs:167-183`).
  `domain/engine.md`, section 6, names claims settled as lost or as not
  started, not this case.
- **The client world's fronts sign out unlike the link.** The link drops
  a sign-out answer of an older sign-in
  (`jig-client-protocol-link/src/sign_in.rs:85`). The scripted front
  answers a sign-out with no sign-in, and takes the party's number for
  the sign-in (`tests/client/src/scripted.rs:240,508-521,530`). The
  joint world's transport signs the client out on any signed-out watch
  end or keyed answer, whatever sign-in its route carried
  (`tests/client/src/joint/packets.rs:216-218,346-348`; only
  `Reply::SignOut` compares, :322), and records no sign-in for keyed
  replies or watches (`joint/transport.rs:32,58-62`). A newer sign-in is
  then forgotten.

## 14. Not taken in

From the retrospectives' sections 9:

- **What is not jig's code:** skein's, smith's and the kit's rows, the
  workspace's pins, and the runs' directories; and two commits whose
  trailers git cannot read, `2497c8299` and `cdfce095a`, which stay so.
- **Resolved since:**
  - `Conversations { project }`, once ignored when the page opened, now
    opens the project's chats (`jig-client-view/src/page/opening.rs:26-33`);
  - the native shell's sign-out answer: signing moved into the link,
    which keeps an answer to its sign-in (section 13);
  - jig's browser layer, its frame parts and cards as primitives: they
    are public functions of `jig-client-view`'s `frame.rs` and
    `cards.rs`, `pages::owed` aside (section 13).
- **One test's time under load** (client-composition's terminal story,
  0.21 to 0.62 s): the focused profile's 500 ms slow period flags it
  when it is past its aim.
- **The steps that added many tests:** in section 6's last
  recommendation, without the steps.
