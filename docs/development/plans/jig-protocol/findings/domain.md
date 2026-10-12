# Code audit for jig-protocol: the domain items

Read-only audit (skein's development.md, 5.4), 2026-10-12. jig `main` = `origin/main` 2fae240a; smith at 2a800ea2 (the revision in jig's `Cargo.lock`, `/srv/data/git/runner/src/smith` HEAD); skein pinned at 30dc6f7. Nothing was edited, committed or checked out. Paths are as built (before item 12's renames), relative to jig unless prefixed `smith`/`skein`. "Users" are counted with `grep -rn` over `crates testing tests examples`. Estimates are production lines (step code, roots and kits), tests excluded.

## Summary

| Item | State today | Production lines (est.) | Lands after | Biggest files touched (>800) |
|---|---|---|---|---|
| 1 The engine stopping | not built (only `views::End::Stopping`, never sent) | 320–420 | — | people/domain.rs 1873, ops route.rs 1585 |
| 2 Several charters, chats' charter, default budget, one account per charter | partly: tasks child lists charters; `core::Settings` has one `charter`/`run`/`account` | 220–320 | — | documents/task_tree.rs 1018, objects.rs 933, inbox_deployment.rs 830 |
| 3 The reserve before a run is placed | partly: fields only (0e952778); `check_run` ignores them | ~40 | 2 (or against `settings.run` first) | authority/check.rs 1006 |
| 4 Accounts on skein's driver; grants pushed and resent | not on the driver's terms; grants never pushed or resent | child rewritten (~400) + ~450 | 2; 1 for `Close`; together with 6.2 | fleet/attempt.rs 1110, hosted.rs 1340 |
| 5 An application's credentials | not built | ~200 | 4 | jig-test-domain routes.rs 1005 |
| 6.1 The hello with its agent's bounds | partly: fleet `Hello` has no bounds; the hub has no hello | ~530 | 2, 3, 6.4 | fleet tests.rs, phrases/jig.rs 2994 |
| 6.2 Notices | half: fleet has them, hub not; core `unreachable!` | ~120 | together with 4 | hosted.rs |
| 6.3 Refusals with turns/answers, the unread fence | not built | ~300 | — (with 6(f)) | hosted.rs, tasks/boundary.rs 1776 |
| 6.4 The start's messages; brief without inbox | partly: ops replays the whole inbox after the assignment | ~240 | with 7, 4, 5 ("the assignment whole") | tasks/domain.rs 1251, brief/orchestrated.rs 848 |
| 6.5 Refused/unread stay and wake the task | not built; `Bounced` is a no-op | ~60 | 6.3, 6.4 | — |
| 6(f) Windows, resume from the current window | not built | 100–130 | 7 | — |
| 6(g) Facts as typed activity, long operations | partly: fleet's typed `Activity`; no root produces it | 250–300 | 6.1 | hosted.rs |
| 6(h) smith's payloads by hand-out | hub carries bytes (as designed); ops root encodes/decodes itself | 80–120 (+~60 moved) | 9 | ops route.rs |
| 7 The conversation's first attempt on the assignment | partly, with two defects | 60–80 | — | routing/tasks.rs 1414, hosted.rs |
| 8 Text by retry, a gap marked cut | retry built; gap dropped; text bypasses hub and fleet | 120–160 | gap rule: —; transport: 6(g), 6.1 | — |
| 9 Host tools decoded outside the root | not built: ops root decodes calls, keys, results | 300–400 new, 900–1,100 moved | 12 | ops translate.rs 1310 |
| 10 People's and tasks' refusals, signing out | 8 sub-items: 2 built (10.7, 10.8), 3 partly, 3 not | 205–375 | 10.3 with 11.6 | people/domain.rs, phrases/jig.rs, client/domain.rs 1238 |
| 11 The store's domain debts | 7 sub-items: only "never erased singly" and the people/core key orders built | 1,250–2,130 | 11.1 first; 11.4 after 6(f), 7; 11.6 after 11.3, 10 | reads/rows.rs 1744, tasks/boundary.rs 1776 |
| 12 The crates' renames | not built | 0 new; ~3,300 mechanical edits | — (land first) | boundary tests.rs 1009 |

In all: about 5,200–6,800 new production lines for items 1–11, ~1,000 moved out of `ops`'s root (item 9), and the rename.

## Order and dependencies

```
12 rename ─────────────────────────────► 9 host tools ──► 6(h) payloads
7 first attempt ──► 6(f) windows ─┬────► 11.4 turns by window
                   6.3 refusals ──┘ (one increment: jig_host::Turn, Payload::Turn, TurnPayload)
                   6.3 + 6.4 ──────────► 6.5 refused/unread wake
2 charters ──► 3 reserve ──► 6.1 hello (also needs 6.4's start count) ──► 6(g) facts ──► 8 transport
2 ──► 4 accounts (+1 for Close, +6.2 notices) ──► 5 credentials
7 + 6.4 + 4 + 5 + 6(f): "the assignment whole" (host::Assignment / RunAssignment / Request::Start / Now::RunPrepared)
11.1 key order ──► 11.3 indexes (11.2 inbox census) ──► 11.6 retention (+10.3 chats forgotten, +11.4 drop-whole write)
independent: 1, 8's gap rule, 10.1, 10.2, 10.4–10.7, 11.5, 11.7 (after 11.1 if both edit rows.rs)
```

- **Land 12 first.** It edits 540 `.rs` files and 37 manifests; every concurrent increment would conflict, and `jig-protocol-*` crates (item 9's `jig-protocol-tools`, the reader in `jig-protocol-charter`) take their names at once.
- **The assignment whole.** Items 4, 5, 6.4, 6(f) and 7 each add a field to the same values: `jig_host::RunAssignment` (7 constructions), `jig_host::Assignment` (9 literals), `Request::Start`/`ToAgent::Start` (3 full destructures, 9 `..` matches), `core::Now::RunPrepared` (producer `run_preparation.rs:131`; consumers ops `route.rs:1245`, `jig-test-domain/routes.rs:421`), `jig_fake_workers::Assignment`, ops `assemble::assignment`. Batch the fields in one increment, or sequence them so each literal is edited once.
- **One turn, one increment.** 6.3 (refusals) and 6(f) (`opens_window`) both change `jig_host::Turn` (5 literals), ops `Payload::Turn` (7 sites), `core::Event::TurnPayload`/`AcceptedTurn`, `TurnRecord`; 11.4 then keys the store on the flag.
- **`crates/jig-core/src/routing/children.rs:284-290` is where 4 items meet.** `fleet::Request::Bounced { .. } => continue` (6.5) and `fleet::Request::Grant | Rejected | Exhausted | Told => unreachable!("account and diagnostic host routes are handled separately")`: nothing handles them separately, so the first root that routes a `fleet::Event::Grant` (item 4), `Rejected`/`Exhausted` (6.2) or `Told` (6(g)) through `core::Event::Fleet` (`vocabulary.rs:423`) crashes the core. Item 4's first increment must replace the arm, not only add the sender.
- **`core::Settings` is singular** (`crates/jig-core/src/lib.rs:107-136`): item 2 replaces `charter`/`run`/`account` with a charter table; items 3, 4 (configured accounts), 6.1 (the fleet's requirements per charter) read from it. `lib.rs` is 707 lines, so the table needs its own module.

## Defects found on the way (not in STATUS.md)

- **Item 7:** the first claim of a resuming charter records `transcript_from = 0` (`crates/jig-core/src/run.rs:138-141`, `routing/run_preparation.rs:89-92`) for the conversation's whole life; three claim-failure paths (`run_preparation.rs:389`, `routing/tasks.rs:229`, `:1120`) erase the task's in-memory `RunProof` though the store keeps the previous one, so the next resume reads every turn of the task from 0 until a restart.
- **10.1:** `Core::signing_in` (`lib.rs:267`) is set at `routing/people.rs:80` and never cleared: every later `Held::PeopleReply` names the last candidate sign-in (`people.rs:131`, `:169`), and `Core::quiescent` (`lib.rs:636`) is false forever after the first sign-in.
- **11.1:** closed funding periods never leave the live table (`crates/jig-core-tasks/src/funders.rs:631-686` only sets `closed`; `restore_funding` :594-619 admits them back); the table is bounded by `Limits::funders`, so once full `open` refuses new periods `Busy` (`funders.rs:418-432`).
- **Item 4:** an unusable account at the second admission check holds the task `Hold::Effects` (`run_preparation.rs:68`) instead of waiting; a missing grant fails the preparation (`:77-81`).
- **6(h):** ops reports every hub answer to the fleet as `fleet::Answer::Ended` (`examples/ops/domain/src/route.rs:1474`), refusals and failures included, after decoding the result itself (`assemble.rs:244-284`).
- **6(f):** smith's `decode_transcript` (smith `crates/smith-protocol-channel/src/transcript.rs:40-52`) refuses a transcript whose first turn is `Ordinary` or that has a later `Window` turn; jig resumes from the conversation's first attempt (`run.rs:137-229`), so a real smith agent would refuse any resume spanning two windows, and the ops world's scripted agent writes `TurnKind::Ordinary` at place 1 (`tests/ops/src/conformance/scripts.rs:441-443`).
- **Tests asserting the opposite of the design** (to be rewritten, not kept green): `a_bounced_message_stays_unread_for_a_later_read` (`tests/core/tests/runs.rs:214-229`, 6.5); `report(&mut domain, 1, 21, b"gap").is_empty()` (`crates/jig-core-views/src/tests/completion_retry.rs:102`) and the views referee's "the scripted host writes consecutive offsets" (`tests/views/src/referee.rs:155`, item 8); `a_chat_answers_parks_and_resumes` resending the whole two-attempt conversation (`tests/core/tests/runs.rs:23-28`, 6(f)); `tests/accounts/tests/simulation.rs` (9 tests of jig's own refresh, keeping and numbering, item 4).
- **Item 3's fixtures:** with the floor `(reserve_turns + 1) * most_costly`, the shared fixture (`tests/core/src/world.rs:298-320`: window 8192, output 1024, prices 1, `price_unit` 1000, `reserve_turns` 1) makes every run need 20; `tests/core/src/faults.rs:110` (10), `tests/ops/src/conformance/config.rs:240` (19) and `tests/core/src/stories/closing_chat.rs:59` (10) would hold every run. Raising `price_unit` at `world.rs:317` and `crates/jig-core/src/tests.rs:279` keeps every budget.

## Note on 10.8 and 11.6 (people timers)

The core world does fire `Timer::People`: `World::fire_due` (`tests/core/src/effects.rs:390-407`) fires every due `core_deadlines()` owner, People included (since 187eb86d; used by `tests/core/tests/joint_clock.rs:94-140`). What never fires it: `World::advance` (`effects.rs:410-417`, Fleet and Tasks only), the conformance world (`tests/conformance/src/lib.rs:390-397`), the ops world (`tests/ops/src/conformance/mod.rs:225-228`), and `ops`'s root, which projects no core deadlines. Item 11.6 below is stated accordingly.

## Item 1. The engine stopping

**Design.**
- `domain/engine.md` 6.1: "From then on it takes nothing that would make a commit: a party's request is answered not ready ... a run's turn or call is answered busy, and its answer left unacknowledged ... a connector's events and outcomes stay with the connector, as when the journal is full; no run is placed, and no procedure stepped." / "It ends every watch saying the engine is stopping ... and says once when nothing it holds waits for a commit".
- `protocol/service.md` 5 (shutdown drains, steps 1–5): "The core hears that the engine is stopping ... What is in flight finishes ... until nothing waits for a commit, which the core says." A second signal aborts.
- `domain/engine.md` 8: "a call the fleet cannot pass up is answered at once, unserved, or busy while the engine stops (section 6.1)".
- `client/domain/contract.md` 7.2 (line 389–391): a watch is "Ended, each saying why: ... the engine is stopping."

**Today: not built** (only the vocabulary for the watch ending exists).
- No core input for stopping: `core::Event` (crates/jig-core/src/routing/vocabulary.rs:331–487) has no `Stop`; `step_one` (routing/mod.rs:402–510), `room` (routing/mod.rs:301–363) and `fire` (routing/mod.rs:512–567) have no stopping gate.
- The core's lifecycle state is `restart::Restart { Cold, Waiting, Open, Failed }` (crates/jig-core/src/restart.rs:33–38): no `Stopping`. The children's own phases: people `Phase { Restoring, Ready, Failed }` with `ready()` (crates/jig-core-people/src/domain.rs:383–388), whose `!ready()` already answers `Again::NotReady` at 8 sites (people/domain.rs:261, 292, 308, 370, 438, 468, 554, 703, 788, 1359); tasks `Startup { Restoring, Ready, Failed }` (crates/jig-core-tasks/src/domain.rs:29–33; refusal `NotReady` at tasks admission.rs:55, 132).
- Watch ending: `views::End::Stopping` exists (crates/jig-core-views/src/boundary.rs:33) but nothing in the core ever sends it (no `End::Stopping` outside the vocabulary; the core's ending sweep `end_invalid`, routing/watch_marks.rs:376–394, only yields `SignedOut`/visibility ends). Downstream consumers already map it: tests/ops/src/joint/front.rs:782, jig-client `WatchEnd::Stopping` (crates/jig-client/src/contract/watches.rs:138; domain.rs:1006, 1096), jig-scripted-engine faults.rs:203–204.
- "Nothing waits for a commit": only `Core::quiescent(journal_idle)` (crates/jig-core/src/lib.rs:614–645), a pure query that also demands fleet idleness, no due runs and no account waits — not "says once", and wrong for a drain (it requires `!fleet.is_ready()`, `fleet.turns()==0`, `!accounts.waiting()`). Callers: tests/core/tests/effects.rs:341,345 only (roots have their own `quiescent`: examples/ops/domain/src/domain.rs:161, testing/jig-test-domain/src/lib.rs:427).
- Runs: the turn path can already answer busy (`fleet::Event::TurnBusy` from `route_refused_payload`, crates/jig-core/src/routing/runs.rs:631–645; fleet `turn::busy`, crates/jig-core-fleet/src/turn.rs:142). Calls: the fleet's answer to a relay is a root payload token only (`fleet::Event::Relayed { to, answer }`, crates/jig-core-fleet/src/boundary.rs:178–181); no "busy" relay answer exists in the fleet vocabulary (the hub has `host::Reply::Busy`, crates/jig-host/src/boundary.rs:521). Answers: `fleet::Request::Answered` is always routed to the task child (crates/jig-core/src/routing/children.rs:295).
- Roots: ops's root stops only on a failed commit or refused restart (`Output::Stop`, examples/ops/domain/src/boundary.rs:163–164; `release`, examples/ops/domain/src/domain.rs:300–306); jig-test-domain likewise (testing/jig-test-domain/src/lib.rs:863–866). Admission refusal for connectors already exists as `Output::Busy` (examples/ops/domain/src/domain.rs:247). Neither root has a shutdown input (`Event` enums: examples/ops/domain/src/boundary.rs:50–109; testing/jig-test-domain/src/lib.rs:166–230).

**Gap.** A core input that sets a stopping state; gates: party/application requests and sign-ins → `NotReady` (simplest: a people `Phase::Stopping`, so the 8 existing `!ready()` sites answer it, plus `sign_in`, crates/jig-core/src/routing/people.rs:69); `fleet::Request::Turned` → `TurnBusy` + drop payload (children.rs:291); `Answered` → drop payload, no acknowledge (children.rs:295); `Relay` → busy answer (children.rs:269; needs a fleet or root busy answer); `Activate`/`resume_fleet` → no placement (run_preparation.rs:351–372; routing/mod.rs:384–392); task timers and procedure steps not fired (routing/mod.rs:523–527; `route_activate` procedure arm run_preparation.rs:356–362); connector events refused at the root's admission (`Output::Busy`). Every watch ended `Stopping` (a stopping arm in `end_invalid`, watch_marks.rs:382–386, covers core and application watches since both live in `view_subjects`). A once-only "drained" output (new `Now` variant) when nothing waits for a commit. Each root: a `Stop` input and the drain/abort sequencing of service.md 5.

**Types and functions that change, with their users.**
- `core::Event` (vocabulary.rs:331) + new variant: exhaustive matches in jig-core only: `room` routing/mod.rs:301, `step_one` routing/mod.rs:402 (2). Constructors outside: none needed beyond the two roots.
- `core::Now` + a drained variant (vocabulary.rs:188): exhaustive matches 7: crates/jig-core/src/tests/completion_retry.rs:52, examples/ops/domain/src/route.rs:1245–1400 (arm list ends 1359), testing/jig-test-domain/src/routes.rs:421–542, tests/core/src/client_peer.rs:159–162, tests/ops/src/joint/engine.rs:342–345, tests/ops/src/joint/front.rs:751–754, tests/client/src/joint/session.rs:796–799.
- `restart::Restart` (restart.rs:33): matched only in restart.rs (≈12 arms in the file).
- `people::Phase` (people/domain.rs, `ready()` 383): matched at people/domain.rs:445, 1540–1544, 1629–1630.
- `views::End::Stopping`: already handled by every `views::End` consumer (crates/jig-core/src/routing/views.rs:406–458; ops route.rs:795, 1000; jig-test-domain client_documents.rs:33; tests/ops/src/joint/front.rs:778–791) — no new users.
- `fleet::Event` (if a relay-busy or `Stopping` event is added; boundary.rs:132): matched by fleet `step` (crates/jig-core-fleet/src/domain.rs:~330–379) and `crates/jig-core/src/routing/children.rs:376` (tag list) — 2.
- Roots: `jig_ops_domain::Event` (boundary.rs:50) matched at examples/ops/domain/src/domain.rs:201–222 and route.rs dispatch; `jig_test_domain::Event` (lib.rs:166) matched in lib.rs:525 `step`.

**Worlds and fixtures.**
- tests/core (jig-core-world): `effects::World` (tests/core/src/effects.rs:27; `send` 442, `drain` 451, `restart_with` 1161, `turn` 1215, `host_call` 1239, `release_commits` 1130); scripted peers `faults::workers` / `workers_at` / `restart` (tests/core/src/faults.rs:12, 18, 74); `peers::Peers::restart` (tests/core/src/peers.rs:44) and fake worker `restart_engine`, which resends unacknowledged turns/answers (testing/jig-fake-workers/src/lib.rs:326). Watches: `watches::world`/`open` (tests/core/src/watches.rs:521, 560), ending recorded at watches.rs:349. Existing restart tests to extend: tests/core/tests/restart.rs, tests/core/tests/faults.rs. Gap in the fakes: jig-fake-workers has no busy-turn/busy-call handling (no `Busy` in testing/jig-fake-workers/src/lib.rs) and `jig-fake-parties` has `Again::NotReady` (testing/jig-fake-parties/src/facts.rs:199) with retry by key (`Reply::Again`, lib.rs:284).
- tests/conformance (jig-conformance-world): harness `crash` (testing/jig-conformance/src/harness.rs:247), `drain` 395, `cold_snapshot`/`from_snapshot` 474/495 — a "stop, drain, crash, restart" story beside the crash stories.
- tests/ops: maps `End::Stopping` (tests/ops/src/joint/front.rs:782); client half already proven by jig-scripted-engine (testing/jig-scripted-engine/src/faults.rs:203; its test tests/faults.rs:121).

**Estimate.** jig-core ≈200–260 (state, ~8 gates, end-all sweep, drained output); jig-core-people ≈15; jig-core-fleet ≈30–50 (busy relay answers while stopping); each root ≈40–60 (ops domain.rs/route.rs, jig-test-domain lib.rs/routes.rs). Total ≈320–420 production lines. Files over 800 that it touches: crates/jig-core-people/src/domain.rs (1873), examples/ops/domain/src/route.rs (1585), testing/jig-test-domain/src/lib.rs (1154) and routes.rs (1005), tests/core/src/effects.rs (1289); crates/jig-core/src/lib.rs is 707 (headroom 93).

**Dependencies.** None upstream among items 1–12. It shares the fleet's turn/answer/relay routing (children.rs:269–300) with item 6 (refusals with turns, unread fence) and item 8 (text by retry); land 1 before or apart from 6 to keep the gates in one place. Item 10's "signing out from the frame" and the client's watch endings consume the same `views::End` vocabulary. service.md 5's protocol half (listeners refusing, components closing) is the protocol layer's and needs this item.

## Item 2. Several charters, a chats' charter, a default budget, one account per charter

**Design.**
- `domain/engine.md` 7.1: the assignment's "charter, in the core's terms: the task's instructions, the brief, the tools its authority gives ... its budget for the run: turns, spend and time, with a reserve of turns and time ... its models ... its credential grants: those its models' accounts need, one account per charter, which its models' endpoints share, until a run needs several".
- `protocol/configuration.md` 3 (line 71), charters row: "each charter the deployment offers, numbered: its instructions ..., its models ..., its tool families, the hosts that may run it, its waiting time, whether its runs resume, its default budget (turns, spend and time, and a reserve of turns and time), which applies to a task whose batch gives none ...; its models' endpoints name one account ...; and which charter chats run under". Line 134–137: "each charter's turns and time are above its reserve's, and its default budget holds its reserve and one of its main model's most costly completions ... or the engine's start is refused, naming the charter."
- `domain/engine.md` 8: "A charter says which kinds may run it." `domain/tasks.md` 2 (lines 103–107): each agent task's charter brings "its default wake policy and waiting time, whether a run resumes ... and which hosts may run it". `domain/people.md` 5 table (line 130): a chat is "a task with a chat's charter ... its budget the role's default for chats"; `client/domain/contract.md` 6 (line 217): *not offered* "(no chat's charter in this project)".

**Today: partly.** The task child already holds a list of charters (`tasks::Domain::new(limits, seed, charters: Box<[u32]>)`, crates/jig-core-tasks/src/domain.rs:148–155, bounded by `tasks::Limits.charters`, limits.rs:77; `charters()` domain.rs:190), tasks carry `Executor::Agent { charter }` (tasks boundary.rs:226), and authority already names charters (`authority::Executor::Charter(u32)`). But the core has exactly one charter, one policy and one account:
- `core::Settings` (crates/jig-core/src/lib.rs:107–136): `charter: u32` (113), `run: RunPolicy` (115), `account: u32` (127), `account_generation` (129) and `account_valid` (131) — the last two are dead (never read; set only in crates/jig-core/src/tests.rs:295–296 and tests/core/src/world.rs:333–334), `chat_authority` (125) is deployment-wide, not per role, `tools: ToolFamilies` (135) deployment-wide.
- `Core::new` passes `Box::new([config.settings.charter])` to the task child (lib.rs:572) and prices one policy (`memory::policy_owned_bytes`, crates/jig-core/src/memory.rs:20–29, checked against `Limits.run_bytes`, lib.rs:478–484, vocabulary.rs:281).
- Every run gets the same policy: `Core::run_policy` clones `settings.run` (crates/jig-core/src/run.rs:36–44); resume decided from `settings.run.resume` (run_preparation.rs:89, 245); the claim names `charter: core.settings.charter` (run_preparation.rs:134) whatever the task's charter.
- A restored task with any other charter is refused (`supported_task`, crates/jig-core/src/stored.rs:386–395, called at 287).
- Default budget: the run's spend is `left(task numbers).min(rules.maximum_run_spend)` (run_preparation.rs:127–128; run.rs:65; connector.rs:591); turns/time/reserve come from the single `RunPolicy` (run.rs:262–282). Tasks carry only `Budget { spend, deadline }` (crates/jig-core-tasks/src/value.rs:124–129), never turns/time, so "a task whose batch gives none" has no per-task budget field to fall back from: the charter's turns/time/reserve always apply, and no per-charter spend default exists.
- One account: `accounts.usable(settings.account)` / `accounts.grant(settings.account, now)` (run.rs:68; run_preparation.rs:77; connector.rs:592); `Model.account` (run.rs:248) is never checked against it.
- Host kinds per charter: chosen by the root, not the charter (`fleet::Event::Start { kinds }` at ops `Ask::ClaimDone`, hard-coded `Kinds::Engine`, examples/ops/domain/src/route.rs:527–538; jig-test-domain `Config.hosting`, testing/jig-test-domain/src/lib.rs:294, 306, 379, used routes.rs:214; set in tests/core/src/world.rs:262 and peers.rs:546).
- Chats' charter: chats and "is this a chat" tests use `settings.charter` (below); the frame already offers one chat charter and a goal list per project (`OfferedCharter { chat, goals, cut }`, crates/jig-core/src/documents/small.rs:103–112) — goals from `tasks.charters()`, but goal admission accepts only `settings.charter` (acts/bounds.rs:62–64, 227).

**Gap.** A charter table in the core (number → instructions, models with their account, tool rights, kinds, waiting, resume, default turns/time/reserve, and its spend cap), plus which charter chats run under (per project per contract.md); runs prepared from the task's own charter; one account per charter checked against its models' `account`; kinds taken from the charter; restore admitting any configured charter; the dead `account_generation`/`account_valid` dropped (item 4 owns accounts' start-up).

**Types and functions that change, with their users.**
- `Settings.charter` — 13 production uses, all jig-core: inbox.rs:109; stored.rs:287; roles.rs:97; routing/watch_marks.rs:205; lib.rs:572; routing/run_preparation.rs:134; documents/inbox_deployment.rs:449; documents/small.rs:111; acts/bounds.rs:52, 62, 64, 227; routing/parties.rs:220. Tests: crates/jig-core/src/tests/frame_facts.rs:34. Constructions: crates/jig-core/src/tests.rs:259; tests/core/src/world.rs:297 (the shared fixture behind 65 `world::config(`/`world::limits()` calls in 18 files: tests/core, tests/ops/src/conformance/config.rs:83, tests/client/src/joint/* (10 files), examples/ops/domain/src/tests.rs).
- `Settings.run` (`RunPolicy`) — 6 production: run.rs:37; documents/task_tree.rs:786; documents/objects.rs:345; routing/run_preparation.rs:89, 245; lib.rs:480. Tests/fixtures: crates/jig-core/src/tests.rs:260 (construction), 1004; tests/core/src/world.rs:298 (construction); tests/ops/src/conformance/config.rs:113, 115, 116, 117; tests/ops/src/conformance/mod.rs:188. (tests/fleet/src/workers.rs:122 is the fleet world's own `Settings`, unrelated.)
- `Settings.account` — 4 core: run.rs:68; routing/run_preparation.rs:77; documents/inbox_deployment.rs:599; connector.rs:592. 4 roots: testing/jig-test-domain/src/lib.rs:430, 943; examples/ops/domain/src/domain.rs:165, 317. Constructions: tests.rs:294, world.rs:332.
- `Settings.chat_authority` (if it moves to the chat charter or the role) — 5 production: roles.rs:98; documents/small.rs:100; acts/bounds.rs:222; routing/parties.rs:223, 225. ~35 test/fixture writes: crates/jig-core/src/tests.rs:224–226, 292, 358, 399, 504–505, 529; tests/acts.rs:726–727; tests/acts/kind_order.rs:30; tests/frame_facts.rs:18; tests/conformance/src/lib.rs:62; tests/core/src/{stories/offers.rs:13–16, 39–42; faults.rs:102; effects.rs:137–138; peers.rs:548–550; world.rs:246–248, 330}; tests/core/tests/{projections.rs:14, 186, 200–201; stories.rs:352, 1073, 1844}; tests/ops/src/conformance/config.rs:104, 106; tests/client/src/joint/{engine_stories.rs:37; role_stories/family_stories.rs:18, 132; role_stories.rs:73}. Keeping it deployment-wide avoids this churn.
- `RunPolicy` / `Model` (run.rs:243–282): constructed at crates/jig-core/src/tests.rs:260 and tests/core/src/world.rs:298; translated by ops `assemble::assignment` (examples/ops/domain/src/assemble.rs:48–113, `model` 114) and jig-test-domain routes.rs:421 (passes `RunCharter` whole).
- `memory::policy_owned_bytes` (memory.rs:20) and `Limits.run_bytes` (vocabulary.rs:281; memory.rs:93, 242, 274; tests.rs:123; tests/core/src/world.rs:121; examples/ops/domain/src/limits.rs:85) — priced per charter.
- `supported_task` (stored.rs:386) — 1 caller (stored.rs:287).
- `fleet::Kinds` source: jig-test-domain `Config.hosting` (lib.rs:294/306/379, routes.rs:214) and ops route.rs:537; fixtures tests/core/src/world.rs:262, peers.rs:546.
- `Now::RunPrepared.charter` (vocabulary.rs:254–264) already exists; consumers: testing/jig-test-domain/src/routes.rs:421 (stores it in `Assignment.charter`, lib.rs:138), ops route.rs:1245 (ignores it).

**Worlds and fixtures.** tests/core: fixture `world::config` (tests/core/src/world.rs:215–345) and `world::limits` (16); variants `effects::fixture` (tests/core/src/effects.rs:112), `peers::fixture` (peers.rs:518), `faults::plan_fixture` (faults.rs:86). Offers stories tests/core/src/stories/offers.rs; frame test crates/jig-core/src/tests/frame_facts.rs:18–35. Unit fixture crates/jig-core/src/tests.rs:193–300 (`config`). ops conformance fixture tests/ops/src/conformance/config.rs:9–121 (`root_for`). A two-charter story (a chat on charter 1, a delegate on charter 2 with other turns/kinds/account) fits tests/core/tests/runs.rs.

**Estimate.** jig-core ≈200–280 (charter table and validation in `Core::new`, ≈25 call sites rewired to a lookup by the task's charter, per-charter pricing); roots ≈20–40 (kinds and account from the charter: ops route.rs/domain.rs, jig-test-domain lib.rs/routes.rs). Files over 800 touched: crates/jig-core/src/documents/inbox_deployment.rs (830), documents/task_tree.rs (1018), documents/objects.rs (933), examples/ops/domain/src/route.rs, testing/jig-test-domain/src/lib.rs, routes.rs, tests/core/src/world.rs (810), crates/jig-core/src/tests.rs (2936). crates/jig-core/src/lib.rs (707) cannot take the table inline; a `charters.rs` module is needed.

**Dependencies.** None upstream. Downstream: item 3 (the reserve is per charter's main model and reserve; buildable on the single `RunPolicy` first, but then redone), item 4 (every configured account wanted: the set comes from the charters' accounts; `settings.account` disappears), item 6's hello check (the fleet needs every charter's models, windows, outputs, turns/spend/time/waiting and reserve time to judge a hello, engine.md 8). Land 2 before 3, 4 and the hello half of 6.

## Item 3. The reserve before a run is placed

**Design.**
- authority.md 7 (l. 288): "A run's spend holds its reserve and one completion more. The reserve's spend is its turns, each at the main model's most costly completion: a window of input at its highest input rate, plus its output at the output rate. The core places a run only if its spend holds the reserve's and one more such completion".
- authority.md 8.3 (l. 355-381): checked "before a run is claimed and again as it starts ... holds its reserve and one more of its main model's most costly completions"; a run failing it "is not started: its task waits for its budget (held, if nothing will free one)".
- tasks.md 5.5 (l. 285): held when "too little left for its next run's reserve and one completion more, with nothing to free more". hosts.md 8 (l. 619-624): "The core places no run smith would refuse."
- smith's formula to match (smith 2a800ea, `crates/smith-domain-run/src/budget.rs:391-404`): `most_costly = ceil((window * max(input,cached,cache_write) + output * output_price) / unit)`, `reserve_spend = most_costly * reserve.turns`; smith's `admit` (budget.rs:365-383) refuses `reserved >= budget.spend` and `most > budget.spend`. The design's rule (`spend >= (reserve_turns + 1) * most_costly`) is strictly stronger, so smith never refuses.

**Today: partly.** Fields built in 0e952778: `jig-core` `Model.window`/`output`/`cache_write_price` (crates/jig-core/src/run.rs:242-257), `RunPolicy.reserve_turns`/`reserve_time` (run.rs:262-279). Only ops's root reads them, passing them to the charter (examples/ops/domain/src/assemble.rs:76, 127-128); the core never does (`grep reserve_turns|\.window|\.output` in crates/jig-core/src: declarations only). The run check is `authority::check_run` (crates/jig-core-authority/src/check.rs:485-540): it tests only `ask.budget <= minimum_run_spend || ask.budget > left` (check.rs:505) and the caps (508-513). **Gap:** no most-costly-completion computation anywhere in jig; `RunAsk` (crates/jig-core-authority/src/boundary.rs:257-273) carries no reserve; a run with spend below `(reserve_turns+1) * most_costly` is placed and smith refuses it at its entrance.
- Nuance (not in STATUS): `RunBudget` always maps to `Hold::Budget` (crates/jig-core/src/run.rs:82-89); 8.3's "waits ... (held, if nothing will free one)" has no waiting branch. hosts.md 8 says "is held", so holding is consistent with 3 of 4 texts.

**What changes, and every user.**
- New `jig-core` fn (in run.rs, ~25 lines): `most_costly(&Model) -> Option<u64>` and the floor `(reserve_turns+1) * most_costly`, u128 checked arithmetic, rounded up once as smith does. Alternatively depend on smith's `smith_domain_run::{most_costly, reserve_spend}` (pub, lib.rs:74), but only jig-charter depends on smith today (crates/jig-charter/Cargo.toml), and jig-core would take a smith domain crate for one formula; recommend jig's own, with a unit test pinning it to smith's on the same numbers.
- `authority::RunAsk` gains a field (e.g. `reserve: u64`, the spend a run must hold); check.rs:505 adds `ask.budget < ask.reserve` → `Answer::Wait, Finding::RunBudget` (reusing `RunBudget` avoids touching the 5 exhaustive `Finding` matches; a new `Finding::Reserve` would need run.rs:79-125, crates/jig-core/src/authority_facts.rs:40-90, and others). `RunAsk` constructors (4): crates/jig-core/src/run.rs:62, crates/jig-core/src/connector.rs:585, crates/jig-core-authority/src/tests_policy.rs:638, :725.
- `Core::run_admission` (run.rs:48-134) computes the floor from `self.settings.run.model` (per charter once item 2 lands); callers: crates/jig-core/src/routing/run_preparation.rs:66 (after workspace) and :363 (when due). `connector_run_write_admit` (connector.rs:575-599) builds a RunAsk too and must pass the same floor.
- `RunCharter.budget` (run.rs:288) computed twice the same way: run.rs:66 and run_preparation.rs:127-128; unchanged, but the floor must be compared against that same value.

**Worlds and fixtures.** The shared core fixture makes every run's floor 20: `tests/core/src/world.rs:298-320` (window 8192, output 1024, prices 1, `price_unit` 1000 → most_costly 10, reserve_turns 1); same in `crates/jig-core/src/tests.rs:260-280`. Fixtures that then hold every run for budget:
- `tests/core/src/faults.rs:110` (`maximum_run_spend = 10`);
- `tests/ops/src/conformance/config.rs:240` (`maximum_run_spend: 19`; ops reuses `jig_core_world::world::config` at config.rs:83, so its model is the same);
- `tests/core/src/stories/closing_chat.rs:59` (authority spend 10); borderline at exactly 20: `tests/core/src/peers.rs:553`, `tests/core/src/effects.rs:965`.
- `jig_core_world::world::config` is called 22 times across tests/ (tests/client joint/*, tests/conformance, tests/ops conformance and joint). Cheapest fix: raise `price_unit` in tests/core/src/world.rs:317 and crates/jig-core/src/tests.rs:279 (e.g. 1_000_000 → most_costly 1, floor 2) so no existing budget changes; the conformance `RunBudget` scenario (tests/conformance/src/lib.rs:277, scenarios.rs:147, spent == 20) is then untouched.
- New tests: a cell in `run_cells_hold_for_readiness_propose_grants_and_refuse_hard_caps` (crates/jig-core-authority/src/tests_policy.rs:636); a story in the effects world (`jig_core_world::effects::World`, tests/core/src/effects.rs; e.g. tests/core/tests/runs.rs) where a chat whose spend left is below the floor is held `Hold::Budget` and never assigned, and one at the floor is assigned.

**Size.** ~40 production lines (jig-core run.rs ~30, authority boundary.rs/check.rs ~6, two RunAsk sites ~2). Tests ~60.

**Dependencies.** None hard. With item 2 (several charters), the floor comes from the task's charter's policy, not `Settings.run` (run.rs:37); land item 3 first against `settings.run` or after item 2. Item 6's hello bounds check (engine.md 8) uses the same windows/outputs.

---

## Item 4. Accounts on skein's driver's terms; grants pushed and resent

**Design.**
- engine.md 12 (l. 762-771): "Every configured account is wanted, from the restart's first step (section 6) to the close"; "Every live attempt hears each new generation of its account, on whatever host, the engine's own included; and the fleet sends each attempt it keeps its current grant when its worker says hello, and when it adopts the attempt". Failure classes: later, sign in again, misconfigured, unloaded, cancelled (l. 776-790).
- engine.md 8 (l. 620-624): "each new generation of an LLM account or an application's credential goes to every live attempt that holds one ... and each attempt the fleet keeps at a worker's hello, or adopts, is sent its current grants".
- protocol/llm.md 5.1 (l. 161-167): the driver does "one exchange per account at a time; a held account refreshed its refresh lead before expiry ... every new record kept before its token is lent ... a rejected token refreshed once ... jig writes no OAuth exchange, and keeps no account's record". 5.2 (l. 171-199) gives the child's vocabulary: asks want / rejected / release / close; hears granted (generation, valid), failed later (with how long), sign in again, **expiring** (l. 188: "the account is usable until its token lapses, and the deployment's document says a person must hand in a newer one"), misconfigured, unloaded (where, why), cancelled, closed. 5.4 (l. 229-238): a handed-in account has no refresh; past its lapse or once rejected it "fails as sign in again".
- protocol/README.md 12.3: "no refresh schedule, keeping or generation numbering of jig's".

**skein's driver, concretely.** `skein-oauth-accounts` is built at jig's pin 30dc6f7 (skein `crates/skein-oauth-accounts/src/boundary.rs`, in the pin; skein STATUS: oauth.md 1, 6 built). Owner requests (boundary.rs:10-31): `SignIn`, `Redirected`, `Cancel`, `HandIn{record}`, `Grant{account}` (held until `Release`), `Rejected{account, generation}`, `Release`, `Kept`, `Close`, `Abort`. Events (36-53): `Visit`, `SignedIn`, `Granted{account, token, generation, valid}` (again for each new generation while held), `Expiring{account, generation}` (once, access-only records), `Keep`, `Failed{account, ends, failure: NotKept | Exchange(..) | Expired | Unloaded{at, why}}`, `Refused{asked, why: Busy|NotWaiting|Redirect|Closed|Account|Source|Full|NotAccessOnly|Held|Record}`, `Closed`. So "on its terms" means: the child sends one `want` (→ `Grant`) per account and then only hears; generations are the driver's; refresh timing is the driver's (its refresh lead); keeping is the driver's keeper. The child keeps only availability (usable, spent-until, failure class, a retry timer for later / sign-in-again / unloaded).

**Today: not on these terms.** `jig-core-accounts` (crates/jig-core-accounts/src/, 572 lines) is jig's own refresher:
- schedules refreshes: `Phase::Fresh{due}` with `refresh_due(expires - refresh_margin)` (domain.rs:260-264, 157, 407), fired by `fire` (domain.rs:228-251);
- numbers generations: `advance` = `generation + 1` (domain.rs:253-258), `Add` refreshes `generation + 1` (domain.rs:398-425);
- keeps: `Request::Keep` (boundary.rs:76), `Failure::Unsaved{valid}` (boundary.rs:58-62), `keeping` flags (domain.rs:11-12, 302-307, 311-320), `Fact.unsaved` (boundary.rs:84);
- throttles rejection itself (`rejected_interval`, domain.rs:180-184), and revokes on `Failure::Refused` (domain.rs:313-316).
- Vocabulary: `Event::{Add, Refreshed, Failed, Rejected, Exhausted, Close}` (boundary.rs:12-41), `Request::{Refresh, Keep, Cancel, Granted, Availability, Refused, Closed}` (boundary.rs:72-81), `State::{Fresh, Refreshing, Retrying, Revoked, Closing}` (boundary.rs:84-91), `Failure::{Unavailable, TimedOut, RateLimited, Refused, Unsaved, Cancelled}` (boundary.rs:45-63). None of llm.md 5.2's classes (sign in again, misconfigured, unloaded where/why) exist.
- **`expiring` has no place:** no `Event::Expiring`, no state, no fact field; the nearest is `Fact.attention` (boundary.rs:88, "Revocation, an unsaved rotation, or a long account cooldown") and the document's `Account.attention` (crates/jig-core/src/documents/inbox_deployment.rs:119). Usability already ends at expiry (`valid` test in `changed`, domain.rs:340-344), so `expiring` needs: an event, a flag/state carried to `Fact` and the document line, cleared by the next `granted`.
- "Wanted from the restart's first step": accounts exist only when the world sends `Event::Add` (no core code adds them; `Settings.account_generation`/`account_valid`, crates/jig-core/src/lib.rs:128-131, are read by nothing but fixtures). Close: nothing sends `accounts::Event::Close` outside tests/accounts (engine stopping, item 1).
- **Grants pushed: not built.** `accounts::Request::Granted` goes to the root as `Now::Account` (crates/jig-core/src/routing/children.rs:308-323) and every root drops it (examples/ops/domain/src/route.rs:1403 passes it out as `Output::Now`; testing/jig-test-domain/src/routes.rs:537; worlds ignore it). No code sends `fleet::Event::Grant` (only matched: crates/jig-core-fleet/src/domain.rs:321; crates/jig-core/src/routing/children.rs:371; tests/fleet/src/world.rs:1302). The fleet's own outputs `fleet::Request::{Grant, Rejected, Exhausted}` are `unreachable!` in the core (children.rs:287-290). The fleet forwards a grant only to the attempt's current channel and retains nothing (crates/jig-core-fleet/src/call.rs:199-208), so it cannot resend at a hello (`moved`, crates/jig-core-fleet/src/attempt.rs:919-925, emits nothing to the parent) or an adoption (`located`, attempt.rs:904-916, emits `Placed`).
- The hub side is built: jig-host keeps each hosted run's grants (crates/jig-host/src/hosted.rs:27), replaces by newer generation and forwards to the agent (hosted.rs:1198-1224), resends at agent start (hosted.rs:495-501); validated distinct by account (crates/jig-host/src/assignment.rs:12-20). Untested: no test sends `host::Event::Grant` (tests/host/src/world.rs:626 only lists it; tests/host/src/parent.rs:281 answers `Request::Grant` with nothing).
- Notices up are missing in the hub: smith's parent face has `Rejected{account, generation}` and `Exhausted{account, retry_after}` (smith `crates/smith-host-domain/src/parent.rs:93-108`); jig-host's `FromAgent`/`Event`/`Request` have neither (crates/jig-host/src/boundary.rs:14-350; `Exhausted` at :640 is a run ending). The fleet has them (crates/jig-core-fleet/src/boundary.rs:198-211, 336-347; `credential_notice` call.rs:211-224, tested crates/jig-core-fleet/src/tests.rs:738-759) but the core drops them as unreachable.
- Admission defects to fix with this item: an unusable account at the second check becomes `Hold::Effects` (crates/jig-core/src/routing/run_preparation.rs:68) instead of waiting (the first check waits, :365 `remember_due`); a missing grant fails the preparation (:77-81).
- One deployment-wide account: `Settings.account` (crates/jig-core/src/lib.rs:127) read at run.rs:68, connector.rs:592, run_preparation.rs:77, documents/inbox_deployment.rs:599, and by roots: examples/ops/domain/src/domain.rs:165, :317; testing/jig-test-domain/src/lib.rs:430, :943 (the due queue `core.due` is gated by that one account's usability).

**Types and functions that change, with users.**
- `jig_core_accounts::{Event, Request, Failure, State, Fact, Limits, Domain, step, fire}`: rewritten (boundary.rs 89, domain.rs 428, limits.rs 26). Users outside the crate (counted by symbol):
  - `Event::Add` 23: crates/jig-core/src/tests.rs:1682; examples/ops/domain/src/tests.rs:317; tests/core/src/world.rs:424; tests/core/src/effects.rs:202, :338; tests/core/src/faults.rs:77; tests/core/src/stories/instructions.rs:46; tests/core/tests/{runs.rs:94, restart.rs:18, stories.rs:1406, memory.rs:30}; tests/conformance/src/lib.rs:614; tests/ops/src/conformance/mod.rs:603; tests/accounts/tests/{simulation.rs ×9 (7,65,80,107,110,112,150,164), fuzzy_simulation.rs:33, memory.rs:18}.
  - `Event::Refreshed` 9 / `Request::Refresh` 12 (the worlds' stand-in component): tests/core/src/world.rs:497-498; tests/core/src/effects.rs:540-541; tests/conformance/src/lib.rs:454-455; tests/ops/src/conformance/mod.rs:512-513; crates/jig-core/src/routing/children.rs:312; rest in tests/accounts.
  - `Request::{Keep, Cancel, Granted, Availability, Refused, Closed}`: crates/jig-core/src/routing/children.rs:313-318 (one exhaustive match) + tests/accounts only.
  - `Event::{Failed, Rejected, Exhausted, Close}`, `Failure::*`: crates/jig-core/src/tests.rs:1689 + tests/accounts only.
  - `Limits{refresh_margin, backoff_*, rejected_interval, spent_attention, facts}` constructed at 3 sites: crates/jig-core/src/tests.rs:137-145, tests/core/src/world.rs:141-149, tests/accounts/src/lib.rs:6-14 (ops and client reuse tests/core's).
  - `State` → client mapping: crates/jig-core/src/documents/inbox_deployment.rs:116; tests/client/src/joint/wire/deployment.rs:160-167 (+ deployment/tests.rs:56, 357-361); crates/jig-core/src/tests.rs:1706. `Fact`: inbox_deployment.rs:594-650, :824.
  - Domain queries: `usable` (8 sites above), `grant` (run_preparation.rs:77), `waiting` (crates/jig-core/src/lib.rs:649), `next_deadline` (testing/jig-test-domain/src/lib.rs:338), `account_views` (inbox_deployment.rs:594), `pop_fact` (lib.rs:606); `fire` via `Timer::Account` (crates/jig-core/src/routing/mod.rs:549-551; tests/ops/src/conformance/mod.rs:227; crates/jig-core/src/tests/engine_timers.rs:141).
- `core::Now::RunPrepared.grant: accounts::Grant` (crates/jig-core/src/routing/vocabulary.rs:263) → a list naming whose table (LLM account vs application, item 5): producer run_preparation.rs:77, :139; consumers examples/ops/domain/src/route.rs:1245 → `assemble::assignment(.., grant)` (assemble.rs:56, :104-108); testing/jig-test-domain/src/routes.rs:421 → `Assignment.grant` (testing/jig-test-domain/src/lib.rs:145; read by nobody). Wildcard matches (unchanged): crates/jig-core/src/tests/completion_retry.rs:52, tests/core/src/client_peer.rs:159, tests/ops/src/joint/engine.rs:342, tests/ops/src/joint/front.rs:751, tests/client/src/joint/session.rs:796.
- Grant types, three copies of `{account, generation, valid}`: `jig_core_accounts::Grant` (boundary.rs:5-9), `fleet::Grant` (crates/jig-core-fleet/src/boundary.rs:94-99), `host::Grant` (crates/jig-host/src/boundary.rs:669-675). Each gains which table it names (item 5). `host::Grant` users: hosted.rs:27, 419-431, 497-498, 1198-1224; assignment.rs:12-20; examples/ops/domain/src/assemble.rs:104; tests/host/src/turn_world.rs:160, 182, 416-417, 491-495; tests/host/tests/memory.rs:135-136; tests/host/src/engine.rs:288; crates/jig-host/src/tests.rs:85.
- Fleet resend: either the fleet keeps each attempt's grant names (`fleet::Event::Start`/`Adopt` gain grants; `call::grant` updates the retained entry even while adrift; `moved`/`located` push `Request::Grant` per entry; `valid` must be kept as an expiry `Time` and re-derived), or the fleet tells the parent at `moved` (new `Request`, like `Placed`) and the core answers with `Event::Grant` from the accounts child. Exhaustive `fleet::Request` matches to update: crates/jig-core/src/routing/children.rs (tag_fleet), tests/fleet/src/world.rs:935-990, tests/fleet/src/turns.rs:242-260. `fleet::Event::Start` constructors: examples/ops/domain/src/route.rs:531, testing/jig-test-domain/src/routes.rs:208, tests/fleet worlds; `Adopt`: crates/jig-core/src/sibling_routes.rs:169.
- Core: for an adoption after restart the core must know an attempt's grants: `RunProof` (crates/jig-core/src/stored.rs:517-534; constructors stored.rs:576, run_preparation.rs:95, tests.rs:455, :1719, tests/activity.rs:68, :220) gains its grant names (the charter's account is derivable from the task's charter; credentials are not). A new core route: on `Granted` for account A, for each `core.proofs` entry with `terminal == None` whose run holds A, step the fleet with `fleet::Event::Grant` (children.rs `route_account`, :335-345). `fleet::Request::Grant` becomes a held/now request to the root, which passes it to the worker link or to the engine's hub (`host::Event::Grant`; ops routes `Held::Assign` at route.rs:250-256, the place to add it); `fleet::Request::{Rejected, Exhausted}` become `accounts::Event::{Rejected, Exhausted}`.
- Startup: the child is made with the configured accounts (from the charters, item 2) and asks `want` for each at the restart's first step (`RestartStep::LoadCore`, crates/jig-core/src/restart.rs:8-23) instead of the world's `Add`.
- Deployment document's account line (inbox_deployment.rs:110-124, 594-650): drop `unsaved`, `Refreshing/Retrying`; add why unusable (sign in again, misconfigured, unloaded where/why, later until, spent until) and expiring. Ripples to the client contract `OwnerAccount`/`OwnerAccountState` (crates/jig-client/src/contract/owner_deployment.rs:85-93, 185-196), the view (crates/jig-client-view/src/owner_deployment.rs, 7 uses; tests/owner_deployment.rs 6), testing/jig-scripted-engine/src/owner_deployment.rs (2) + tests (6), tests/client/src/joint/wire/deployment.rs (6) + tests (6); `unsaved` users: inbox_deployment.rs ×3, jig-client contract ×1, jig-client-view ×3, scripted-engine tests ×2, client wire ×3. client/domain/pages.md 7 (l. 323-324) asks "each usable or not, and why".

**Worlds and fixtures.**
- Accounts world `tests/accounts` (lib.rs `World`, `LIMITS` :6-14; simulation.rs 183 lines, 9 tests all about jig's own refresh/keep/numbering: `refresh_due_early_rejection_throttle_and_stale_generation` :16, `unsaved_rotation_retries_only_the_write...` :34, ...; fuzzy_simulation.rs `respond` :5 plays the component; memory.rs): rewritten, its `respond` playing skein's driver (granted, later, sign in again, expiring, misconfigured, unloaded, cancelled, closed).
- Fleet world `tests/fleet` (`World` src/world.rs:473; parent's `request` :935-990 ignores `Request::Grant` at :943; `adopt` :1252; referee src/referee.rs): the parent pushes new generations; the referee checks every live attempt's worker holds the newest after hellos, losses and adoption.
- Host world `tests/host` (src/world.rs, src/parent.rs:281, src/turn_world.rs:491-495 asserts grants empty, fixtures src/fixtures.rs `assign` :6): a newer grant reaches the agent once started; notices up.
- Core effects world `jig_core_world::effects::World` (tests/core/src/effects.rs: `send` :442, `restart` :1150, `say` :1193, `turn` :1215, `end` :1228, `Delivery::Assigned` :719, account answers :540-546); stories in tests/core/tests/restart.rs (helpers `hello` :10, `account` :17): a refreshed generation reaches a live attempt; a run kept at a hello and an adopted one are resent the current grant; an unusable account makes a due run wait (not `Hold::Effects`).
- ops world tests/ops/src/conformance/mod.rs:512-513, :603 (stand-in component), engine-hosted runs get `host::Event::Grant`.

**Size.** jig-core-accounts rewritten (~400 lines, net smaller than 572: no refresh schedule, keeping or numbering); jig-core ~150 (routing children.rs ~50, run_preparation ~15, vocabulary ~10, restart wants ~15, RunProof ~5, document ~40, Settings ~10); fleet ~50; jig-host ~40 (grant table, LLM grants to the agent only, notices up); ops root ~45; jig-test-domain ~40; client contract/view/scripted ~75. ≈ 450 new/changed production lines beyond the child's rewrite.

**Dependencies.** Item 2 (one account per charter, charters' accounts as the configured set; `Settings.account` goes). Item 1 (close: "Close releases every account, and ends once the component has closed"; the stop sends `Close`). Item 6 (the hub's link vocabulary: grants down, rejected/exhausted notices up, the hello carrying kept attempts). Item 11 (`RunProof` gains grant names: a store record change). Item 5 builds on this item's push/resend path; land 4 before 5.

---

## Item 5. An application's credentials

**Design.**
- engine.md 12.1 (l. 794-815): "Named, never seen: each by the application's number for it and a generation, with how long it is valid"; "Granted through the root: the application's root tells the child each generation granted ... One is usable while a granted generation is still valid"; "Lent to runs: a connector names those a run's workspace needs as it gathers its items; the assignment carries them as grants, each naming the application's table; and they follow their attempts as an account's do"; "Rejected and exhausted are LLM accounts' only".
- connectors.md 10 (l. 553-558): "the connector names the application's credentials the host needs ... by the application's number for each. The core grants the run the current generation of each, and the assignment carries them as grants". Interface row (connectors.md l. 84): "its workspace items, with their size, and the application's credentials they need".
- engine.md 4.1 (l. 129-130): the core hears "from the application, through its root: each generation of an application's credential granted". authority.md 8.3 (l. 360-362): "the application's credentials its workspace needs, are usable". protocol/channel.md 8: "A grant names whose table its value comes from: jig's accounts' or the application's".

**Today: not built.** No code names an application credential (`grep -i credential` in crates/testing/tests/examples: comments only). Pieces it would ride on:
- `core::Event::WorkspacePrepared{task, attempt, writes}` (crates/jig-core/src/routing/vocabulary.rs:397; routed crates/jig-core/src/routing/mod.rs:443 → run_preparation.rs:39-89) carries writes only.
- The reference connector gathers items (`jig-test-connector` `Event::Items` boundary.rs:535, `Request::Ready{token,size}`, `Request::Workspace{token, items}` boundary.rs:707, `Item{resource, writable, state}` boundary.rs:412-416, values.rs:80-117), but **no root asks for them**: jig-test-domain answers `Now::WorkspaceRequest` from holdings only (testing/jig-test-domain/src/routes.rs:385-420) and ignores `Request::Workspace` (routes.rs:766); ops has no workspace items (examples/ops/domain/src/route.rs:1242-1243 answers `writes: Some([])`; assemble.rs:101 `workspace: None`). Only tests/test-connector/tests/stories.rs:262-264 exercises items.
- Grants carry no table: `accounts::Grant`, `fleet::Grant`, `host::Grant` all `{account, generation, valid}` (item 4); jig-host matches by account number alone (hosted.rs:1205; assignment.rs:16-18), so an application number equal to an LLM account number would collide.
- `authority::RunAsk.accounts: Box<[bool]>` (crates/jig-core-authority/src/boundary.rs:267-269, bounded by `Limits.accounts`, crates/jig-core-authority/src/limits.rs:35) can carry the credentials' usability too; today it has one entry (run.rs:68, connector.rs:592).

**What changes, with users.**
- Accounts child: a second table beside LLM accounts (credentials: number → generation, expiry), `Event` for a credential granted by the root, `usable`/`grant` by (table, number); `Limits` gains a credentials bound (constructors: crates/jig-core/src/tests.rs:137, tests/core/src/world.rs:141, tests/accounts/src/lib.rs:6; worst case summed in crates/jig-core/src/memory.rs:112 and documents/inbox_deployment.rs:824).
- `core::Event::WorkspacePrepared` gains the credential numbers the connectors named (users: routing/mod.rs:334, :443; testing/jig-test-domain/src/routes.rs:391, :419; examples/ops/domain/src/route.rs:1243).
- `run_admission` (run.rs:48) and `connector_run_write_admit` (connector.rs:575) add each credential's usability to `RunAsk.accounts`; an unusable credential waits like an account (run_preparation.rs:66-70, :363-365 `remember_due`; the root's due gate examples/ops/domain/src/domain.rs:317, testing/jig-test-domain/src/lib.rs:943 must then ask per run, not per one account).
- `Now::RunPrepared` grants list (item 4) includes them; `RunProof` keeps their numbers for resends (stored.rs:517); a credential's new generation pushed to live attempts through item 4's route.
- jig-test-connector: `Item` or `Request::Ready`/`Workspace` names credentials (boundary.rs:412, :535, :707; values.rs:80-117; matches at testing/jig-test-connector/src/client_marks.rs:175, tests.rs:230, :431; testing/jig-test-domain/src/client_documents.rs:149, lib.rs:1129, routes.rs:766; tests/test-connector/src/world.rs:153, :236), with a config naming which resource needs which credential.
- jig-test-domain: `WorkspaceRequest` routing gathers items and credentials from its connectors (routes.rs:385-420, ~40 lines new), assembles grants into its `Assignment` (lib.rs:135-146).
- jig-host: grants named by table; application credentials are not forwarded to the agent (hosted.rs:497-498, :1215 send LLM grants only), and are kept for the workspace's operations (hosts.md 7: "No credentials on disk ... the protocol layer maps to credentials per operation"; `Request::Prepare{owner, workspace}` boundary.rs:231-234 carries no grants today).

**Worlds and fixtures.** Core effects world (tests/core/src/effects.rs; tests/core/tests/runs.rs or restart.rs): a run whose connector names credential N waits while N has no valid generation, starts once the root grants it, its assignment carries N's grant, and a new generation reaches the live attempt and a kept/adopted one (reusing item 4's stories). Test-connector world (tests/test-connector/tests/stories.rs:262-264): items name their credentials. Host world: an application grant is retained, never sent to the agent. Fixture config for credentials: jig-test-connector's world config (boundary.rs:426 "Configuration generated by a world from its seed") and tests/core/src/world.rs `config` (:215).

**Size.** ~200 production lines: accounts child ~70, jig-core ~50, jig-test-connector ~35, jig-test-domain ~50 (wiring items it ignores today), jig-host ~10 (with item 4's table field).

**Dependencies.** Item 4 first (the child on the driver's terms, grants as a list naming tables, the push/resend path, `RunProof` grant names). Item 6 (grants down the link). Item 11 (the `RunProof` record). Independent of item 3.

---

### Files over 800 lines these items touch
crates/jig-core-fleet/src/attempt.rs 1110 (resend at `moved`/`located`); crates/jig-host/src/hosted.rs 1340; crates/jig-core-authority/src/check.rs 1006, tests_policy.rs 1339; crates/jig-core/src/documents/inbox_deployment.rs 830; crates/jig-core/src/tests.rs 2936; crates/jig-core-fleet/src/tests.rs 1268; examples/ops/domain/src/route.rs 1585; testing/jig-test-domain/src/routes.rs 1005, lib.rs 1154; tests/core/src/effects.rs 1289, world.rs 810, tests/core/tests/stories.rs 1886; tests/fleet/src/world.rs 1320; tests/host/src/world.rs 1048; tests/ops/src/conformance/mod.rs 975. Near the limit: crates/jig-core/src/lib.rs 707, crates/jig-core/src/routing/vocabulary.rs 649.

## Item 6 (first half). The hub's link vocabulary: hello, notices, refusals, the fence, the start's messages

Audited at jig `main` 2fae240a. smith at its `main` 2a800ea2, which is the revision in jig's `Cargo.lock` (`/srv/data/git/runner/src/smith`). jig links only `smith-domain-run`, `smith-charter` and `smith-transcript` (Cargo.toml). It does not link `smith-host-domain`, `smith-inline-agent` or `smith-agent-service`.

### 6.0 Context: the hub's vocabulary today, and smith's

**jig-host has no enum of its own toward the link.** Its `Event` and `Request` mix four faces. `Request::to_agent` (crates/jig-host/src/boundary.rs:378-401) splits off the agent's requests, and the root routes the rest.

- **`Event`** (boundary.rs:12-148). Events by origin:
  - **From the link/engine:** `Inbound` :14, `Assign` :33, `AcknowledgeTurn` :48, `Cancel` :61, `Grant` :65, `Relayed` :72.
  - **From the agent, through `FromAgent`:** `Called` :22, `Withdrawn` :28, `Turn` :38, `Facts` :43, `Finished` :54, `Started` :113, `Bounced` :118, `Yielded` :124, `Faulted` :128, `Gone` :134.
  - **From the top level:** `CancelAll` :86, `Report` :91, `Unacknowledged` :96.
  - **From the workspace:** `Prepared` :101, `Unprepared` :107, `Delivered` :139, `Saved` :144, `RelayCancelled` :79.
- **`Request`** (boundary.rs:152-262). Requests by destination:
  - **Toward the link:** `Relay` :153, `Answer` :187, `Turn` :194, `Bounced` :219, `Hosting` :227.
  - **To the agent, through `ToAgent` at :267-303:** `Message` :165, `Reply` :172, `Start` :178, `AcknowledgeAgentTurn` :201, `Grant` :249, `Stop` :246.
  - **To the workspace or root:** `Deliver` :205, `CancelRelay` :214, `Prepare` :231, `Abort` :238, `Save` :254, `Release` :259.
- **`FromAgent`** (:308-354) has 10 variants. There is no notice (rejected or exhausted) and no unread or fence report.
- **smith's values cross as bytes throughout, never typed:**
  - the charter: `RunAssignment.charter: Box<[u8]>` :415;
  - the transcript: `Assignment.turns: Box<[Box<[u8]>]>` :424;
  - a turn's body: `Turn.body` :454;
  - call names: `Relay.call` :157 and `AnsweredCall.name` :433;
  - the result: `Finish::Ended { outcome }` :459;
  - facts: `Event::Facts { fact: Box<[u8]> }` :43. These are smith's opaque facts, not jig's typed activity (the other half).
  - **No run identity field exists anywhere.** Grants are names only: `Grant { account, generation, valid }` :671-675.
- **The engine side of that link** is ops's root:
  - hub to fleet: `host_out`, examples/ops/domain/src/route.rs:1411-1522;
  - fleet/core to hub: `Held::Assign`, `Held::Inbound`, `Held::Relay`, `Held::AcknowledgeTurn` and `Held::Cancel`, route.rs:249-307.
  - The fleet's vocabulary is crates/jig-core-fleet/src/boundary.rs:134-471.
  - jig-host and jig-core-fleet each depend on `skein-lib` only (their Cargo.toml). Every shared notion is a duplicated type, translated by the root: for example `host::Bounce` to `fleet::Bounce` at route.rs:1497-1499.
- **What smith's main offers for this half** (crates/smith-host-domain/src):
  - **The start:** `Start { identity, messages: Box<[Message]>, logical_run, activation, workspace, charter, transcript, answered, directories, grants }` (channel.rs:21-42), with `Message { name, label, text }` (channel.rs:13-17). `Down::Start { start, window: Window { turns, bytes } }` (channel.rs:590-596).
  - **A turn:** `Turn { opens_window, number, spent, read: Option<Token>, body }` (channel.rs:120-134).
  - **The answer:** `Answer { read, turns, spent, result }` (channel.rs:507-520). `read` is the unread fence: "Last message covered by an actual told turn, echoed at the terminal".
  - **The parent face** (parent.rs:56-144) has:
    - `MessageRefused { client, name, reason: MessageRefusal }` :129-135, with `MessageRefusal { TooLarge, Full, Ending, NameInUse }` (boundary.rs:55-64);
    - `Unread { client, names }` :121;
    - `Waiting { client, read }` :83-87;
    - the notices `Rejected { client, account, generation }` :93-99 and `Exhausted { client, account, retry_after }` :101-107;
    - `Long` and `LongDone` :89-91, which belong to the other half.
  - **Start refusals:** `RunInvalid::Messages` (channel.rs:452) and `RunInvalid::Window` (:460).
  - **smith has no single "agent bounds" value for a hello.** The bounds the hello must state are spread over:
    - `smith_agent_service::profile::Policy`: `max_turns`, `max_spend`, `max_time`, `max_waiting`, `inbox`, `unacknowledged` and `check_timeout` (crates/smith-agent-service/src/profile/quantities.rs:58-110);
    - `profile::Model { window, output }` (quantities.rs:115-120);
    - `smith_host_domain::Limits { message_bytes, messages, turns, unacknowledged_bytes }` (limits.rs:33-47);
    - `smith_inline_agent::Limits { smith, window: smith::Window }` (crates/smith-inline-agent/src/limits.rs:15-25).
  - smith's protocol/limits.md 2.1 and 2.3 name these quantities. jig's protocol/channel.md 5.1 lists the hello's fields. jig therefore defines its own `Bounds` in jig's terms. A root's protocol layer fills it from smith's profile, and only the root links smith.

### 6.1 The hello as one value, with its agent's bounds, checked by the fleet

**Design.**
- hosts.md 2: "a **hello:** the host's slots, how long it may take to stop a run it lost contact for, the workstreams it holds, the runs it hosts and those whose turns or answers wait for acknowledgement; and its agent's bounds, by which the fleet admits it."
- hosts.md 6.1: "the hello as one value, which a worker's root assembles from the hub, its workspace, its agent capability's bounds and its link's graces … In the engine … with one hello."
- engine.md 8: "The fleet refuses a hello, as not admitted and naming why, whose agent's bounds … do not hold every run the engine may place there."
- protocol/channel.md 5.1 lists the fields: "for each model it declares, its name, window and output; the most turns, spend, time and waiting a run may have; its checks' timeout; the largest message a run takes and how many its inbox holds; the turns it keeps unacknowledged".

**Today: partly built. The hub has no hello, and the fleet's hello has no bounds.**
- **jig-host.** There is no `Hello` type. `Event::Report` (boundary.rs:91) is answered by `Request::Hosting { runs: Box<[Hosting]> }` (:227-229, built by `hosted::report`, crates/jig-host/src/hosted.rs:395-403). It lists only run, attempt and phase (`Hosting`, :555-560, and `Phase`, :564-574, with no `Answered`).
  - The runs whose answers await acknowledgement are counted apart by the top level, through `Event::Unacknowledged { answers }` (:96-98).
  - Retained turns are read by query, through `Domain::retained_turns` and `turn`/`nth` (crates/jig-host/src/domain.rs:168-193).
- **jig-core-fleet.** `Hello { stop_bound, slots, workstreams, hosting }` (crates/jig-core-fleet/src/boundary.rs:475-484) has no bounds. `channel::hello` (crates/jig-core-fleet/src/channel.rs:99-181) checks only three things:
  - the reserved channel 0 (:101-115);
  - room for another worker;
  - `stop_bound >= grace` (:120-138).
  - It then drops the stop bound: `stop_bound: _` at :140.
  - `HelloRefusal { ReservedChannel, Workers{limit}, StopBound{grace} }` (crates/jig-core-fleet/src/history.rs:8-15) has no reason for agent bounds.
- **The engine's own host has no hello.** It is pre-registered from `Limits.engine_slots` in `Domain::new` (crates/jig-core-fleet/src/domain.rs:68-80), and channel 0 refuses any hello (channel.rs:101).
  - ops discards the hub's report: `host::Request::Hosting { .. } => {}`, examples/ops/domain/src/route.rs:1504.
  - ops only asserts `limits.host.slots == limits.core.fleet.engine_slots` (examples/ops/domain/src/domain.rs:92).
- **The fleet knows nothing the check needs:** no charters' models, windows or outputs, no budgets, no message bound, no start count.
  - The charter and policy are singular in the core: `Settings.charter: u32` and `Settings.run: RunPolicy` (crates/jig-core/src/lib.rs:107-140).
  - `RunPolicy { turns, time, waiting, reserve_turns, reserve_time, model, alternatives, modify, shell, call_timeout, .. }` is at crates/jig-core/src/run.rs:262-280. `Model { name, window, output, .. }` is at run.rs:242-257.
  - The deployment's most spend per run is `authority.rules().maximum_run_spend` (used at crates/jig-core/src/routing/run_preparation.rs:127).

**Gap.**
1. jig-host needs a link-facing hello value, made of the hub's part (slots, the runs hosted, and the runs whose turns or answers wait). The root adds its stop bound (link), workstreams (workspace) and `Bounds` (agent). It replaces `Report`/`Hosting`.
2. `fleet::Hello` gains `bounds: Bounds`, with `Bounds { models: Box<[ModelBound{name, window, output}]>, turns, spend, time, waiting, checks, message_bytes, inbox, unacknowledged }`.
3. The fleet needs the engine's requirements per host kind. Each charter needs its kinds, its models (main and alternatives, with window and output), its turns, time, waiting and reserve time, and whether it allows a change. The deployment needs its most spend per run, its message bound, how many messages a start carries, and the hub's unacknowledged-turn bound. These are handed to `fleet::Domain::new` (domain.rs) or carried as limits. `Limits` is `Copy`, so a list of models needs a constructor argument.
4. `channel::hello` must check `bounds` against those requirements for the charters whose `Kinds` allow `HostKind::Worker`. A failure is refused with a new `HelloRefusal::Agent{why}` naming the shortfall.
5. The engine host gets one hello on channel 0, carrying the bounds of its root's agent configuration (in ops, configured values, since `smith-inline-agent` in the engine is not planned). Either it is checked at construction and a refusal fails the start, or channel 0 is allowed one hello in place of `engine_slots` pre-registration.

**Types and functions that change, with their users.**
- **`jig_host::Event::Report` → a hello request, and `Request::Hosting`, `Hosting`, `Phase`:**
  - Definitions: boundary.rs:91, 227-229, 395 (the `to_agent` arm), 555-574. Producer: hosted.rs:395-403, 804. Handler: domain.rs:234.
  - **Users (12):** crates/jig-host/src/domain.rs:234; examples/ops/domain/src/route.rs:1504; tests/host/src/engine.rs:207, :223 (`reconnected(&[Hosting])`); tests/host/src/parent.rs:288; tests/host/src/turn_world.rs:307, :473; tests/host/src/world.rs:620, :730, :774, :908; tests/host/tests/memory.rs:50, :107, :305-306.
- **`fleet::Hello` struct literals (17)**, each gaining `bounds`:
  - crates/jig-core-fleet/src/tests.rs:125 (helper `hello()`, :120-132), :412, :1240;
  - crates/jig-core/src/tests.rs:1600, :1629; crates/jig-core/src/tests/deployment_connectors.rs:141;
  - tests/core/src/world.rs:431; tests/core/src/effects.rs:209; tests/core/src/peers.rs:61; tests/core/src/stories/instructions.rs:53; tests/core/tests/restart.rs:13; tests/core/tests/runs.rs:87;
  - tests/fleet/src/world.rs:878; tests/fleet/src/turns.rs:289; tests/fleet/tests/host_kinds.rs:130, :168; tests/fleet/tests/memory.rs:105.
  - Translators that build it from a kit's `Up::Hello`: tests/core/src/peers.rs:58-66 (from `jig_fake_workers::Up::Hello`, testing/jig-fake-workers/src/lib.rs:103 and :456) and tests/fleet/src/world.rs:870-879 (its own `Up::Hello`, :364, sent by tests/fleet/src/workers.rs:245).
- **`fleet::HelloRefusal`, a new variant.** The value carries on to the client.
  - **Fleet and core:** crates/jig-core-fleet/src/channel.rs (3 uses), crates/jig-core-fleet/src/tests.rs (2), crates/jig-core/src/documents/inbox_deployment.rs:169-174 and :577 (`HelloRefusalLine`), crates/jig-core/src/tests.rs:1657, crates/jig-core/src/tests/deployment_connectors.rs:169.
  - **Client contract and view:** crates/jig-client/src/contract/owner_deployment.rs:97-102 and :200 (`OwnerHelloRefusal`), crates/jig-client-view/src/owner_deployment.rs:135-156 (plus `P::HelloRefusal*` phrases in crates/jig-client-view/src/phrases/), crates/jig-client-view/src/tests/owner_deployment.rs (4 uses).
  - **Kits and worlds:** testing/jig-scripted-engine/src/owner_deployment.rs:114 and its tests (3 uses), tests/client/src/joint/wire/deployment.rs:170-173 and its tests (6 uses).
  - **About 30 sites in 11 files.** This is a client contract change.
- **`fleet::Domain::new` (domain.rs:60-80) and `Limits` (crates/jig-core-fleet/src/limits.rs:12-50).**
  - The constructor is called at crates/jig-core/src/lib.rs:575 and in fleet tests and worlds.
  - The requirements are computed in jig-core from `Settings` and the charters (lib.rs:107-140).
- **ops's root** (examples/ops/domain/src/domain.rs:80-130, route.rs:1504) sends the engine host's hello with its bounds at construction.

**Worlds and fixtures.**
- **The fleet world** (tests/fleet):
  - `world.rs`: `Up::Hello` :364 and its translation :870-879, the trace :1298;
  - `workers.rs:236-252`: the scripted hello;
  - `turns.rs:284-300`: `hello()`;
  - `tests/host_kinds.rs:120-175`: the engine and worker kinds, the natural home for "a worker whose agent lacks a charter's model, or holds a smaller window, is refused";
  - `referee.rs`.
- **The fleet's unit harness:** crates/jig-core-fleet/src/tests.rs `hello()` :120-132, with refusal tests near :389 and :410-425 (`StopBound`), which are the pattern for a new bounds refusal.
- **The core world:** the hello in tests/core/src/effects.rs:207-215 (inside `World::new`, :161), tests/core/src/world.rs:429-437 and tests/core/src/peers.rs:58-66. Here a refused hello is shown on the deployment document (crates/jig-core/src/tests/deployment_connectors.rs:139-170 is the existing pattern).
- **The host world:** tests/host/src/turn_world.rs:307 (`Event::Report`) and :473 (`Hosting`), and tests/host/src/engine.rs:223 `reconnected`.

**Production lines (estimate): about 530.**
- jig-host hello value and assembly: about 100;
- fleet `Bounds`, requirements, check and engine hello: about 250;
- jig-core requirements from charters: about 70;
- the client contract, view and phrase for the refusal: about 40;
- ops's root, for the engine hello and the translation: about 70.

### 6.2 Notices: a grant its provider rejected, an account exhausted until a time

**Design.**
- hosts.md 2: "**notices,** relayed from the run's agent: a grant its provider rejected, an account exhausted until a time".
- hosts.md 6.1: "notices relayed from the agent".
- channel.md 5.1, rows `rejected` and `exhausted`: "task, attempt; an LLM account, generation" and "task, attempt; an LLM account; how long until it may be used again".

**Today: not built at the hub. Half-built in the fleet, and a crash path in the core.**
- **jig-host.** There is no notice in `FromAgent` (boundary.rs:308-354), `Event` or `Request`. smith's `Request::Rejected` and `Exhausted` (parent.rs:93-107) have no place to land.
- **jig-core-fleet** has the notices:
  - `Event::Rejected { channel, run, attempt, account, generation }` and `Event::Exhausted {.., retry_after}` (boundary.rs:199-212);
  - the fence `call::credential_notice` (crates/jig-core-fleet/src/call.rs:208-223, routed at domain.rs:322-338);
  - out to `Request::Rejected` and `Request::Exhausted` (boundary.rs:334-345).
- **jig-core turns them into a panic:** `fleet::Request::Grant | Rejected | Exhausted | Told => unreachable!("account and diagnostic host routes are handled separately")` at crates/jig-core/src/routing/children.rs:287-290.
  - Nothing handles them "separately". No root ever sends `fleet::Event::Rejected` or `Exhausted` (no user outside the fleet's tests, crates/jig-core-fleet/src/tests.rs:738-760).
  - The accounts child has `Event::Rejected` and `Event::Exhausted` (crates/jig-core-accounts/src/domain.rs:176, :191). A root reaches them only directly, through `core::Event::Account` (crates/jig-core/src/tests.rs:1682-1689; examples/ops/domain/src/tests.rs:317), and never from a run.

**Gap.**
- jig-host needs:
  - `FromAgent::Rejected { owner, account, generation }` and `FromAgent::Exhausted { owner, account, retry_after }`, mapped in `Event::from_agent` (boundary.rs:359-372);
  - link requests `Request::Rejected { run, attempt, account, generation }` and `Request::Exhausted {..}`, fenced to live runs in hosted.rs.
- ops's `host_out` (route.rs:1411-1522) translates these to `fleet::Event::Rejected` and `Exhausted`.
- The core routes `fleet::Request::Rejected` and `Exhausted` to `accounts::Event::Rejected` and `Exhausted`, replacing the `unreachable!` at children.rs:287-290. `Grant` and `Told` keep their own handling. What the accounts child then does is item 4's: availability and lending only.

**Types and functions that change, with their users.**
- **`jig_host::FromAgent`:** producers are tests/ops/src/conformance/scripts.rs (agent scripts, :94-260) and tests/host/src/parent.rs (agent capability, :263-295).
- **`jig_host::Request`, which is matched exhaustively (6 sites):**
  - examples/ops/domain/src/route.rs:1424-1520;
  - tests/host/src/engine.rs:188-221;
  - tests/host/src/parent.rs:263-295;
  - tests/host/src/turn_world.rs:470-480;
  - tests/host/src/world.rs:725-780;
  - tests/host/tests/memory.rs:100-110.
- **jig-core:** children.rs:287-290 and :371-373 (the `route_fleet` touch list).

**Worlds and fixtures.**
- **The host world:** the agent capability in tests/host/src/parent.rs (`take` :263, `wake` :295), with a scripted rejection and exhaustion. The peer world tests/host/src/turn_world.rs checks that a notice of a fenced attempt is dropped.
- **The fleet:** crates/jig-core-fleet/src/tests.rs:730-762 already covers the fleet half.
- **The core world:** tests/core/src/effects.rs, which feeds `fleet::Event::Rejected` the way `host_call` does at :1238-1254, and checks the accounts child's state on the deployment document.

**Production lines (estimate): about 120** (hub 60, ops 25, core 35).

### 6.3 Refusals carried with the next turn or the answer; the unread fence

**Design.**
- hosts.md 2, turns: "the messages refused since the run's previous turn, each by name with smith's reason". Answer: "the last message it read (the unread fence, section 9), the messages refused since its last turn".
- hosts.md 6.4: "No refusal crosses on its own: the hub keeps each, its name and smith's reason, with the run's next turn, or its answer if none follows … the hub makes its own refusals before it relays a later message."
- engine.md 7.2: "The core commits the turn with its spend, with the messages up to that one taken but those it says the run refused".
- channel.md 5.1: "A turn or an answer carries at most as many as the run's inbox takes", and they are resent after a hello with their turn.

**Today: not built. Refusals cross on their own, the answer has no fence, and the core neither keeps refused messages nor reads a fence.**
- **The hub sends `Request::Bounced { run, attempt, name, bounce }` (boundary.rs:219-224) at once, at five places:**
  - too large, hosted.rs:260;
  - ending while stopping, hosted.rs:284;
  - relayed from the agent, hosted.rs:600-614 (`bounced`, via `FromAgent::Bounced`, boundary.rs:338-342);
  - the pre-start hold full, hosted.rs:1000 (`hold`);
  - held messages of a run that never starts, hosted.rs:1012-1017 (`bounce_held`).
- **The hub's types lack the fields:**
  - `Turn { turn, spent, read, body }` (boundary.rs:449-455) has no refusals and no `opens_window`.
  - Retention (`turns::Pending`, crates/jig-host/src/turns.rs:20-25, and `copy` :170-172) keeps none.
  - `Answer { turns, spent, ending }` (boundary.rs:464-469, built by `next_answer` hosted.rs:1226) has no `read` and no refusals.
  - `FromAgent::Finished { owner, turns, spent, finish }` (:328-333) has no `read`.
  - `Bounce { TooLarge, Full, Ending }` (:545-552) lacks smith's `NameInUse` (smith boundary.rs:63).
- **The fleet passes bounces up as notices:**
  - `Event::Bounced` (crates/jig-core-fleet/src/boundary.rs:259-265) goes through `call::bounced` (call.rs:163-179) and `domain.rs:351-353` to `Request::Bounced` (:446-451). `Bounce` is at :531-538.
  - Turn and answer bodies are parent tokens: `Event::Turn { body: Token }` :236-242 and `Event::Answer { payload }` :226-232.
- **The core drops the bounce:** `fleet::Request::Bounced { .. } => continue` (crates/jig-core/src/routing/children.rs:284-286). See 6.5.
  - The turn path carries only `read`: ops's `Payload::Turn { spent, read, .. }` (route.rs:1437-1460, 1261-1269) goes to `core::Event::TurnPayload { run, attempt, turn, body, read, cumulative }` (crates/jig-core/src/routing/vocabulary.rs:425).
  - That becomes `route_turn_payload` (crates/jig-core/src/routing/runs.rs:760-784), then `tasks::Event::Turn { reply_to, task, attempt, turn, read, offered, cumulative }` (crates/jig-core-tasks/src/boundary.rs:1372-1389).
  - It is admitted by `admission::turn` (crates/jig-core-tasks/src/admission.rs:42-116), whose `inbox::take(task, read)` (crates/jig-core-tasks/src/inbox.rs:456-466) removes **every** word numbered at most `read`, refused or not.
- **The answer path carries no fence:**
  - `core::Event::AnswerPayload { .., cumulative, end, saved, invalid_saved }` (vocabulary.rs:427-435) becomes `tasks::Event::Activation { reply_to, task, attempt, end, saved, cause }` (boundary.rs:1453-1470).
  - `TerminalRecord` (crates/jig-core/src/stored.rs:484) and `TurnProof` (:501) keep no refusals. `RunProof.offered` (stored.rs:527) is the only fence kept.

**Gap.**
1. **jig-host:**
   - Add a `Refused { name, reason }` type, with reason `TooLarge | Full | NameInUse | Ending`.
   - Keep a per-hosted-run list of refusals since the last turn, bounded by the run's inbox.
   - Move that list into the next staged turn (`Turn.refused`, retained and resent with it in turns.rs), or into the answer (`Answer.refused`).
   - Add `Answer.read` (the fence) and `FromAgent::Finished.read`.
   - Remove `Request::Bounced`. The five producers above push into the per-run list instead.
2. **jig-core-fleet:** remove `Event::Bounced`, `Request::Bounced`, `Bounce` and `call::bounced`. The refusals ride inside the parent's turn and answer payloads, which the fleet never reads.
3. **ops's root:**
   - `Payload::Turn` and `Payload::Answer` (examples/ops/domain/src/route.rs:1437-1475; the payload enum in examples/ops/domain/src/domain.rs) carry `refused` and the answer's `read`.
   - `Now::TurnPayload` and `Now::AnswerPayload` (route.rs:1261-1290) pass them to `core::Event::TurnPayload` and `AnswerPayload`.
4. **jig-core:**
   - `TurnPayload` gains `refused: Box<[u64]>`, and `AnswerPayload` gains `read` and `refused`.
   - `route_turn_payload` (runs.rs:760-784) and the answer route `route_answer_payload` (runs.rs:721-758) pass them on.
   - The answer's `read` is checked against `RunProof.turn.read`: it must equal the last committed turn's read (smith: "covered by an actual told turn").
5. **jig-core-tasks:**
   - `Event::Turn` and `Event::Activation` gain `refused: Box<[u64]>`, validated as offered and in the inbox.
   - `inbox::take` keeps refused words, and the refusals are durable for the attempt in a new `TaskRecord.refused: Box<[u64]>` (the record is at boundary.rs:~850-960, the inbox at :952), cleared at the run's end. Without it, a later turn whose fence passes a refused message takes it. This also holds across a restart, since the task record is what the store keeps.

**Types and functions that change, with their users.**
- **`jig_host::Request::Bounced` (removed):** definition at boundary.rs:219-224 and :394, 5 producers listed above.
  - **Consumers (9):** crates/jig-host/src/tests.rs:333, :364; tests/host/src/engine.rs:196; tests/host/src/parent.rs:287; tests/host/src/turn_world.rs:476; tests/host/src/world.rs:728, :773; tests/host/tests/memory.rs:106 (`Asked::Bounced` :49, :276); examples/ops/domain/src/route.rs:1491-1502.
- **`fleet::Event::Bounced`, `Request::Bounced` and `Bounce` (removed):**
  - **Fleet:** call.rs:163-179, domain.rs:351-353, lib.rs:68 (re-export), crates/jig-core-fleet/src/tests.rs:8, :626, :736, :751-752, :798-815.
  - **jig-core:** children.rs:284-286, :377.
  - **Worlds:** tests/fleet/src/world.rs:367 (`Up::Bounced`), :907, :1022, :1314; tests/fleet/src/workers.rs:44; tests/fleet/src/turns.rs:265; tests/fleet/tests/memory.rs:305; tests/core/tests/runs.rs:221.
  - **ops:** route.rs:1492.
  - **20 sites.**
- **`jig_host::Turn` struct literals (5):** crates/jig-host/src/turns.rs:171; tests/host/src/turn_world.rs:218, :548; tests/host/tests/memory.rs:394; tests/ops/src/conformance/scripts.rs:216.
  - The same struct gains `opens_window` in the other half. Both changes should land in one increment.
- **`Event::Finished` and `FromAgent::Finished` (gain `read`), 8 producers:** boundary.rs:365; crates/jig-host/src/tests.rs:397; tests/host/src/fixtures.rs:23 (`finished`); tests/host/src/turn_world.rs:269; tests/host/tests/memory.rs:398; tests/ops/src/conformance/scripts.rs:139, :176, :187, :242. Handler: crates/jig-host/src/domain.rs:226. Consumer: tests/host/src/world.rs:600.
- **`jig_host::Answer` (gains `read` and `refused`):** built at hosted.rs:1226. Matched in crates/jig-host/src/tests.rs:126, :188, :209, :312, :338 and tests/host/src/engine.rs (`ending` :397). Read by ops's route.rs:1462-1476.
- **`core::Event::TurnPayload` and `AnswerPayload`:** crates/jig-core/src/routing/mod.rs (4 arms), crates/jig-core/src/tests/activity.rs (5), examples/ops/domain/src/route.rs:1270 and :1281, testing/jig-test-domain/src/routes.rs:453-475 (2), tests/core/tests/effect_fences.rs (2).
- **`tasks::Event::Turn` literals (13):**
  - crates/jig-core/src/routing/runs.rs:772; crates/jig-core/src/tests/live_delegates.rs:35; crates/jig-core/src/tests/read_filters.rs:188;
  - crates/jig-core-tasks/src/tests/revisions.rs:325, :455;
  - tests/tasks/src/world.rs:1105; tests/tasks/tests/activity.rs:47; tests/tasks/tests/funding.rs:15; tests/tasks/tests/inbox.rs:36, :48; tests/tasks/tests/memory.rs:168, :179, :344.
  - Destructured at crates/jig-core-tasks/src/domain.rs:614. Matched at tests/tasks/src/world.rs:385, :513 and tests/tasks/src/coverage.rs:50.
- **`tasks::Event::Activation` (24 constructions in 10 files):**
  - crates/jig-core/src/routing/children.rs (2), crates/jig-core/src/routing/runs.rs (2);
  - crates/jig-core-tasks/src/tests/hold_origin.rs (1), crates/jig-core-tasks/src/tests/revisions.rs (3);
  - tests/tasks/src/world.rs (4), tests/tasks/src/coverage.rs (2), tests/tasks/tests/funding.rs (1), tests/tasks/tests/memory.rs (5), tests/tasks/tests/restart.rs (4).
  - The handler is crates/jig-core-tasks/src/domain.rs.
- **The test application's root event `root::Event::Turn` and `Answer`** (testing/jig-test-domain/src/lib.rs:244-260, handled at :594-599) gain `refused`, and the answer gains `read`.
  - **Constructions (7):** tests/core/src/effects.rs:1217 (`turn`), :1230 (`end`); tests/core/src/peers.rs:82; tests/core/src/world.rs:544, :645; tests/core/tests/projections.rs:280; tests/core/tests/store.rs:76; tests/core/tests/fuzzy_watches.rs:30.
- **`jig_core_tasks::inbox::take`** (inbox.rs:456) has 2 callers: admission.rs:87 and procedure.rs:60.
- **`TaskRecord` literals (5):** crates/jig-core-tasks/src/domain.rs, tests/core/src/stories/closing_chat.rs, tests/core/src/stories/instructions.rs, tests/core/tests/stories.rs, tests/tasks/src/world.rs. The stored task codec is in jig-fake-store and ops's store, which is item 11's.

**Worlds and fixtures.**
- **The host world:**
  - The agent capability tests/host/src/parent.rs never scripts `FromAgent::Bounced` today. It is to refuse (full or name in use) and leave messages unread.
  - The peer world tests/host/src/turn_world.rs (`assign` :150, `assign_conversation` :174, `received_message` :211, link loss and `Report` :307) checks the design's rule, hosts.md 11: "each refusal inside a turn or answer no later than the first whose fence passes it", resent after a hello.
  - Fixtures: tests/host/src/fixtures.rs `inbound` :10 and `finished` :22.
  - hosts.md 11 wants smith's `smith-host-domain` over smith's scripted agent processes. tests/host links no smith crate (tests/host/Cargo.toml).
- **The tasks world:** tests/tasks/tests/inbox.rs (`say` :8, pattern `a_read_fence_takes_only_offered_words` :28-60), with a refused word kept past a later fence. Helpers in tests/tasks/src/world.rs: `claim` :852, `terminal` :883, `stage` :517.
- **The core world:**
  - tests/core/src/effects.rs `World::new` :161, `say` :1193, `turn` :1215, `end` :1228.
  - `a_bounced_message_stays_unread_for_a_later_read` (tests/core/tests/runs.rs:214-229) asserts the opposite of the design: a bounced message is read by the same run's next turn. It is rewritten.
- **The ops world:** the scripted agents in tests/ops/src/conformance/scripts.rs (`answer` :90, `poll` :170-260). The design says "Messages that come once its run has finished, before the stop, are not read" (scripts.rs:27-28), which today produces neither a fence nor a re-wake.

**Production lines (estimate): about 300.**
- the hub's refusal list, fields and fence: about 120;
- the fleet: about −50 net;
- ops payloads: about 30;
- the core's payload events, routes and fence check: about 50;
- the tasks child's `refused`, `take` and validation: about 100.

### 6.4 The start carries the inbox's waiting messages; the brief has no inbox section

**Design.**
- engine.md 7.1(3), the claim commits "the messages its start carries (tasks.md, 8.2)"; 7.1(4): "its start's messages, named, oldest first; the rest are relayed as the run reads (7.4)".
- engine.md 9: "A brief has no inbox section. The messages waiting in the task's inbox go with the assignment as its start's messages".
- tasks.md 8.2: "A run's start carries the messages waiting in its task's inbox, each whole, oldest first, as many as the run's inbox takes … never more relayed and unread than the run's inbox takes".
- channel.md 5.2, assignment: "the start's messages: … as many as the run's inbox takes, each with its name, its sender's label and its words".
- smith: `Start.messages` (channel.rs:24-25), refused with `RunInvalid::Messages` (:452).

**Today: partly built. The core hands the whole inbox to the root, which replays it as ordinary inbound messages after the assignment. The hub's start carries none, the claim commits only a high-water mark, and nothing bounds relays by the run's inbox.**
- **The tasks child** puts the whole inbox in `RunContext.inbox`, with `last_message` (crates/jig-core-tasks/src/domain.rs:748-808; `RunContext` at boundary.rs:1712-1717).
- **The core:**
  - sets `RunProof.offered = context.last_message`, which is every waiting message (crates/jig-core/src/routing/run_preparation.rs:93-96; `RunProof::claimed` at stored.rs:576);
  - emits `Now::RunPrepared { .., inbox: context.inbox, .. }` (run_preparation.rs:131-141; vocabulary.rs:254-264).
- **`tasks::Event::Claim { reply_to, task, attempt, resumed, budget, writes }`** (boundary.rs:1422-1436; sent at run_preparation.rs:415) carries no start's messages.
- **ops's root** keeps `inbox` beside the assignment (examples/ops/domain/src/route.rs:1245-1258; `domain::Assignment.inbox` at examples/ops/domain/src/domain.rs:20). After `Held::Assign` it replays every word as `host::Event::Inbound` through `offered_inbox` (route.rs:249-259, 341-356): unbounded, outside the assignment, and bypassing the fleet.
- **The hub** holds those inbound messages until the run is live, within `Limits.held` (crates/jig-host/src/limits.rs:35; `hold`, hosted.rs:997-1002). Beyond that bound it refuses them `Full`.
  - `Assignment { assignment, turns, answered }` (boundary.rs:420-427), `Request::Start` and `ToAgent::Start` (:178-186, :282-290) have no `messages`.
  - `Invalid { Charter, Transcript, Grants }` (:597-602) has no `Messages`.
- **The test application's root** passes `inbox` in its `Assignment` (testing/jig-test-domain/src/lib.rs:133-145; routes.rs:421-443). The core world's peers drop it: `workers::Assignment { task, attempt, budget, transcript }` at tests/core/src/peers.rs:255-262, and testing/jig-fake-workers/src/lib.rs:17-22. In the core world the start's messages never reach a worker.
- **Later messages:**
  - Relayed on arrival, unbounded, while the run is claimed or running: `wake::after_message` sends `Request::Relay` (crates/jig-core-tasks/src/wake.rs:170-172).
  - Chained by the core on `proof.offered == relay.previous` (crates/jig-core/src/routing/children.rs:243-267; `relay_previous` at crates/jig-core/src/sibling_routes.rs:439-459).
  - There is no relay of the waiting remainder as the run reads.
- **The brief:**
  - `brief::Core::Inbox` exists: crates/jig-core-brief/src/planned.rs:19-20, the required-whole rule :111-112 and :149-154, `cut_core` crates/jig-core-brief/src/orchestrated.rs:532, the test planned.rs:257-269.
  - ops titles it at examples/ops/domain/src/assemble.rs:37.
  - The core never builds it (no `Core::Inbox` in crates/jig-core/src; the sections built are run_preparation.rs:221-312). It is dead vocabulary.

**Gap.**
1. **The start's count.** A deployment number for the run's inbox, "as many as the run's inbox takes", which the fleet also checks at a hello (6.1). The start carries the oldest N whole waiting words, and their bytes are bounded by the hub's limits.
2. **The tasks child.** `Event::Claim` gains the start's fence, the last message the start carries, committed with the claim. A relay window: relayed-and-unread is at most N. `after_message` relays only within the window, and admitting a turn (admission.rs:42-116) relays the next waiting words as its fence frees room.
3. **The core.**
   - `RunProof.offered` is set from the start's fence, not `last_message`.
   - `Now::RunPrepared` carries only the start's words.
   - The relay chain `relay_previous` and `relaying` (crates/jig-core/src/lib.rs:265) must allow relays that the turn admission triggers.
4. **jig-host.**
   - `Assignment.messages: Box<[Message{name, sender, words}]>`, carried by `Request::Start` and `ToAgent::Start` to the agent.
   - Admission refuses `Invalid::Messages` beyond the hub's bounds.
   - `held` remains only for messages relayed between the assignment and the run being live.
5. **ops's root.** Put the start's words in the assignment (examples/ops/domain/src/assemble.rs:48-100) and delete `offered_inbox` (route.rs:341-356).
6. **The brief.** Remove `Core::Inbox` and its whole-rule. Removing it touches 6 sites.

**Types and functions that change, with their users.**
- **`jig_host::Assignment` literals (9):** crates/jig-host/src/tests.rs:144, :387; tests/host/src/fixtures.rs:7 (`assign`); tests/host/src/turn_world.rs:186; tests/host/src/engine.rs:295; tests/host/tests/memory.rs:159, :295, :370; examples/ops/domain/src/assemble.rs:97. Destructured at hosted.rs:123.
- **`Request::Start` and `ToAgent::Start`:**
  - Constructed at hosted.rs:424, mapped at boundary.rs:382-383, fully destructured at tests/host/src/turn_world.rs:416.
  - Matched with `..` at crates/jig-host/src/tests.rs:68, :106, :146, :360; tests/host/src/parent.rs:272; tests/host/src/world.rs:829; tests/host/tests/memory.rs:103; tests/ops/src/conformance/scripts.rs:94; examples/ops/domain/src/tests.rs:372.
- **`Invalid`:** crates/jig-host/src/tests.rs:188, tests/host/src/engine.rs (the expected refusals).
- **`core::Now::RunPrepared`** (vocabulary.rs:254-264):
  - Producer: run_preparation.rs:131.
  - Consumers: examples/ops/domain/src/route.rs:1245; testing/jig-test-domain/src/routes.rs:421.
  - Matched with `..`: crates/jig-core/src/tests/completion_retry.rs:52, tests/core/src/client_peer.rs:159, tests/ops/src/joint/front.rs:751, tests/ops/src/joint/engine.rs:342, tests/client/src/joint/session.rs:796.
- **`tasks::Event::Claim` (19 sites in 10 files):**
  - crates/jig-core-tasks/src/domain.rs:605 (handler) and `run::claim` crates/jig-core-tasks/src/run.rs:64;
  - crates/jig-core/src/routing/run_preparation.rs:415;
  - crates/jig-core/src/tests/live_delegates.rs (1), crates/jig-core/src/tests/read_filters.rs (1);
  - crates/jig-core-tasks/src/tests/hold_origin.rs (1), crates/jig-core-tasks/src/tests/instructions.rs (1), crates/jig-core-tasks/src/tests/revisions.rs (6);
  - tests/tasks/src/world.rs (3: `claim` :852, `claim_budget` :858, `claim_budget_writing` :862);
  - tests/tasks/tests/memory.rs (2), tests/tasks/tests/restart.rs (3).
- **`jig_fake_workers::Assignment`** (testing/jig-fake-workers/src/lib.rs:17-22): tests/core/src/peers.rs:256, testing/jig-fake-workers/src/tests.rs:4.
- **`test-domain::Assignment.inbox`** (testing/jig-test-domain/src/lib.rs:141): its readers in tests/core (effects.rs:719, peers.rs:255) and tests/conformance.
- **`brief::Core::Inbox`:** planned.rs:20, :149, :154, :257-269; orchestrated.rs:532; examples/ops/domain/src/assemble.rs:37.

**Worlds and fixtures.**
- **The tasks world:** tests/tasks/tests/inbox.rs `a_parked_chat_wakes_on_its_persons_words` :63-71 (it reads `contexts[&1].inbox`), with a start bounded at N and the rest relayed as turns read. Helpers: tests/tasks/src/world.rs `claim` :852 and `stage` :517.
- **The core world:** tests/core/src/effects.rs (`say` :1193, `turn` :1215) and `Assigned` handling (:719). Here the assignment should carry the start's words, and tests/core/src/peers.rs:255-262 should forward them to `jig_fake_workers` (whose `Script::Turn { read }`, lib.rs:58, reads them).
- **The host world:** tests/host/src/turn_world.rs `assign` :150 and `received_message` :211, with start messages delivered in the start, before any `Message`. Fixture: tests/host/src/fixtures.rs `assign` :6-8.
- **The ops world:** tests/ops/src/conformance/scripts.rs `answer` :90-125, where the start's messages become `agent.unread`.
- **The brief world:** tests/brief needs no change (it uses `Core::Task` only: tests/brief/tests/simulation.rs:26, :50; fuzzy_simulation.rs:30).

**Production lines (estimate): about 240.**
- tasks start fence and relay window: about 120;
- core proof and `RunPrepared`: about 30;
- hub `messages` and admission: about 60;
- ops assemble, net: about 10;
- the brief: about −15.

### 6.5 Messages refused or unread stay in the inbox and wake the task again; `fleet::Request::Bounced` today

**Design.**
- hosts.md 9: "**Refused or unread, it stays** in the task's inbox, untaken, and wakes the task again when the run ends, as it would had it arrived then (tasks.md, 8.2 and 8.3). Nothing a host does drops a message."
- tasks.md 8.3: "A message a run refused or left unread (8.2) is judged again when the run ends, as if it arrived then: it wakes the task if its kind does, and otherwise waits for the next activation."

**Today: not built.**
- **`fleet::Request::Bounced` is a no-op.** crates/jig-core/src/routing/children.rs:284-286 reads:
  ```rust
  // A message its host did not pass on stays in the task's inbox,
  // unread, for the run's next activation (hosts.md, section 2).
  fleet::Request::Bounced { .. } => continue,
  ```
  The comment holds only until the run's next turn. `inbox::take` (crates/jig-core-tasks/src/inbox.rs:456-466) removes every word numbered at most the turn's `read`, so a bounced word older than a later-read one is taken unread. tests/core/tests/runs.rs:214-229 even has the run read the bounced word.
- **Nothing re-judges the inbox when an agent run ends.**
  - `run::activation` (crates/jig-core-tasks/src/run.rs:300-441) sends a parked run to `Active::Idle` (run.rs:409-415), or to `Due` only if narrowed.
  - It never re-applies wake policy to the words left in the inbox: those relayed but unread, and those refused.
  - The procedure path has this: `wake::after_step` (crates/jig-core-tasks/src/wake.rs:184-196, called at crates/jig-core-tasks/src/procedure.rs:91). Agents do not.
  - So a person's words relayed as the run answered sit until something else wakes the task. The ops world documents that case (tests/ops/src/conformance/scripts.rs:27-28, :115-118).
- **`fleet::Request::Undelivered`** (no live host, children.rs:279-282) also leaves the word in the inbox with no re-judgement.

**What `Bounced` would do.** Under the design it disappears: refusals arrive inside the turn or the answer (6.3). The core then:
1. at a turn, takes the words up to `read` but those refused, recording the refused ones durably for the attempt (`TaskRecord.refused`, 6.3 gap 5);
2. at the run's end (`tasks::Event::Activation`), clears the attempt's refused set. For every word still in the inbox that was relayed or refused, it calls the per-message wake judgement `wake::after_message(domain, env, task, None, word, out)` (wake.rs:114-183), as `after_step` does for procedures. A waking kind sends the task `Due` with `Why::Message`, and a quiet kind waits.

**Types and functions that change, with their users.**
- **crates/jig-core-tasks/src/run.rs** `activation` (the Parked, Failed and Refused arms, :400-418): a new `wake::after_run` or a generalised `after_step`.
- **crates/jig-core-tasks/src/wake.rs:184-196.**
- **crates/jig-core/src/routing/children.rs:284-286:** the arm is removed with the variant (6.3). The `Activation` users are as in 6.3.

**Worlds and fixtures.**
- Two stories the design names do not exist (no test matches; `grep` for unread or refused tests finds only tests/core/tests/runs.rs:215 and the procedure test tests/tasks/tests/inbox.rs:434):
  - tasks.md 12: "a person's words left unread as a run ended, waking its task again";
  - engine.md 15: "a message a run left unread, waking its task again as the run ends".
- **The tasks world:** tests/tasks/tests/inbox.rs (`say` :8; the patterns `a_parked_chat_wakes_on_its_persons_words` :63-71 and `a_refused_procedure_step_takes_nothing_and_restart_offers_the_unread_storm_again` :434). The referee tests/tasks/src/referee.rs, rule "no committed message is lost", gains "none taken unread or refused".
- **The core world:** tests/core/src/effects.rs `say` :1193, `turn` :1215, `end` :1228. Rewrite tests/core/tests/runs.rs:214-229 into "refused, kept past a later fence, and waking the task as the run parks", and add the unread story.

**Production lines (estimate): about 60** (tasks 50, core 10, the latter net negative with 6.3's removal).

### Dependencies

**Within this half.**
- 6.3 comes before 6.5, since refusals must reach the tasks child first.
- 6.4's start fence and relay window come before 6.5's "relayed and unread" set.
- 6.1 needs 6.4's start count: the hello checks "an inbox that holds fewer than a start carries".

**On the other items.**
- **6.1:**
  - Item 2 (several charters, a chats' charter, a default budget): the fleet's requirements are per charter and per host kind. Today there is one charter and one `RunPolicy` (lib.rs:107-140).
  - Item 3 (the reserve): the checks' timeout against `RunPolicy.reserve_time` of a charter that allows a change.
  - Item 4: grants are resent at a hello and an adoption, in the same `channel::hello` (channel.rs:99-181).
- **6.2:** item 4 (accounts on skein's driver: what the accounts child does with `Rejected` and `Exhausted`). Item 5 only if notices may name an application's credential; the design names LLM accounts only.
- **6.3:**
  - Shares `jig_host::Turn`, ops's `Payload::Turn`, `core::Event::TurnPayload` and `TurnRecord` with the other half's windows (`opens_window`). Land them in one increment, or sequence them deliberately.
  - Item 11 (turns saved with `opens_window`) touches `TurnRecord` too.
  - Item 1 (stopping) touches the same core routes for a turn answered busy.
- **6.4:** shares `jig_host::Assignment`, `Now::RunPrepared` and ops's `assemble::assignment` with:
  - item 7 (the conversation's first attempt for the run's identity);
  - items 4 and 5 (grants and application credentials granted with the assignment);
  - the other half (the transcript's current window).
  - One "assignment whole" increment avoids touching the 9 `host::Assignment` literals three times.

### Files over 800 lines this half touches

- crates/jig-host/src/hosted.rs (1340)
- examples/ops/domain/src/route.rs (1585)
- crates/jig-core-tasks/src/boundary.rs (1776)
- crates/jig-core-tasks/src/domain.rs (1251)
- crates/jig-core-fleet/src/tests.rs (1268)
- crates/jig-core-fleet/src/attempt.rs (1110)
- tests/fleet/src/world.rs (1320)
- tests/host/src/world.rs (1048)
- tests/tasks/src/world.rs (1165)
- tests/core/src/effects.rs (1289)
- crates/jig-core/src/tests.rs (2936)
- crates/jig-core-tasks/src/stored.rs (1049), where the task record codec lives

## Item 6 (second half). Windows, facts, smith's payloads

jig `main` = 2fae240a; smith pinned at 2a800ea2 (jig `Cargo.lock`), which is smith's checkout `/srv/data/git/runner/src/smith` HEAD. All smith citations below are at that revision.

### 6(f) Windows: `opens_window`, the current window's start, a resume from it

**Design.**
- hosts.md 6.4: "A turn ... numbered within its attempt, with what it spent, the last message it read, and whether it opens a window ... A resumed run is given its current window's turns only."
- engine.md 7.2: "The core commits the turn ... and, when it opens a window, with its task's current window starting there"; "A run that resumes is given its current window only, within the resume limit (the deployment's bound on the current window's bytes, across the attempts it spans)". "One past the limit starts fresh from a brief that carries the transcript's tail."
- engine.md 5.4 table: "each live agent task's conversation: its first attempt, and where its current window starts (7.2) | the core"; "transcripts: turns, with their spend and whether each opens a window | ... to resume, from the current window's start".
- channel.md 6.1: "The record carries beside them ... whether it opens a window ... Down, a resumed run's transcript is its current window's turns".

**smith offers (what jig would call).** smith's `record::Turn.kind` is `First{affinity} | Ordinary | Window{affinity}` (smith `crates/smith-protocol-channel/src/turn.rs:46-59`); `record::Kind::opens_window()` (`crates/smith-domain-session/src/record.rs:120`). The parent face hands it up typed: `smith_host_domain::Turn { opens_window, number, spent, read, body: TurnValue }` (`crates/smith-host-domain/src/channel.rs:118-133`), and `smith-inline-agent` the same (`crates/smith-inline-agent/src/translate.rs:333-346`). Encode/decode for the hand-out: `smith_protocol_channel::encode_turn(&record::Turn, &wire::v3::Limits, &Endpoints)` (`turn.rs:17`), `decode_transcript(&[Box<[u8]>], limits, endpoints)` (`transcript.rs:18`). **`decode_transcript` refuses a transcript whose first turn is not `First`/`Window` or that has a `Window` turn after its first (`transcript.rs:40-52`, `Refusal::Malformed`), and requires places without a gap (`transcript.rs:33-36`)**: so a run resumed today with a conversation that spans two windows, or whose turns came from a tail, would fail transient on a real smith agent. smith's place numbering runs across activations (smith `docs/design/protocol/transcript.md` §2 "its place in the conversation: from one, going on across the activations"), while jig's hub numbers per attempt; jig never reads the place, so nothing changes there.

**Code today: not built.** No `opens_window`/window field anywhere in jig (`grep -rn opens_window` in crates/testing/tests/examples: 0 hits).
- Hub: `jig_host::Turn { turn, spent, read, body }` (crates/jig-host/src/boundary.rs:448-455); copied by `turns::copy` (crates/jig-host/src/turns.rs:170-172). No flag.
- Fleet: carries only `body: Token` (crates/jig-core-fleet/src/boundary.rs:236-242, `Request::Turned` 364-369). Spend and read already ride in the root's payload, not in the fleet, so the flag needs no fleet change: it rides in the root's `Payload::Turn` like `spent`/`read` (examples/ops/domain/src/boundary.rs:170; testing/jig-test-domain/src/lib.rs:518).
- Core turn commit: `route_accepted_turn` (crates/jig-core/src/routing/runs.rs:681-719) saves `RunProof` and a `TurnRecord { task, attempt, turn, spent, read, at, transcript }` (crates/jig-core/src/stored.rs:461-478). No flag is stored and no window start is set.
- The conversation's start is `RunProof.transcript_from` ("Earliest attempt whose turns belong to this usable conversation", stored.rs:522-523), set at claim (crates/jig-core/src/routing/run_preparation.rs:92-95) and validated `transcript_from <= attempt` (crates/jig-core/src/lib.rs:672). This is the attempt granularity of the *conversation*, which item 7 reuses for the run's identity. **No window start is kept.**
- Resume: `begin_transcript` (crates/jig-core/src/run.rs:137-158) reads `proof.transcript_from`; `append_transcript` (run.rs:163-209) counts **every turn of the conversation** (`transcript.bytes`) and keeps a byte tail, slicing an oversize turn's bytes (run.rs:185-191, which is only fit for the brief's tail); `resumed_turns` (run.rs:220-229) sends the whole conversation if its bytes ≤ `resume_bytes`, else nothing; `transcript_oversized` (run.rs:213-215) and the fresh-start rule (run_preparation.rs:89-91, 245) compare the whole conversation, not the current window.
- The load: the core asks `Now::StartPreparation { task, transcript_waiter }` (crates/jig-core/src/routing/vocabulary.rs:238; produced run_preparation.rs:335-336) with no start key. The worlds read **every turn of the task** (tests/core/src/effects.rs:507-528; tests/ops/src/conformance/mod.rs:547-566) and ops hands them to `append_transcript` (examples/ops/domain/src/restart.rs:89-103).

**Gap.** (1) A `opens_window: bool` on the hub's `Turn`, the roots' `Payload::Turn`, the core's `Event::AcceptedTurn`, and `TurnRecord` (store.md 15.4 "saved with whether each opens a window"). (2) The window's start, `(attempt, turn)`, kept with the task. `RunProof` is replaced at every claim (`RunProof::claimed`, stored.rs:576-586), so the start must be carried over as `transcript_from` is: `Transcript` gains it from the old proof in `begin_transcript`, and `workspace_prepared` writes it into the new proof (run_preparation.rs:92-95); `route_accepted_turn` sets it when an accepted turn opens a window (`tasks::Accepted::New` arm, runs.rs:697-713). `valid_proof` (lib.rs:671-680) checks `window <= (attempt, turn)`. (3) `Now::StartPreparation` names the window's start so the root reads the range from `CoreKey::Turn { task, attempt, turn }` (stored.rs:193; key built at examples/ops/domain/src/store.rs:67 and testing/jig-test-domain/src/lib.rs:59), and `append_transcript` drops turns before it. `transcript.bytes` then counts the current window only. (4) Keep the brief's tail (`BriefPart::TranscriptTail`, crates/jig-core/src/brief_text.rs:224, 388) on the loaded rows. Store-side debts (never erased singly, dropped whole) are item 11's; this item gives them the flag and the start.

**Types and functions that change, with their users:**
| Changes | Users (file:line) | Count |
|---|---|---|
| `jig_host::Turn` + `opens_window` (jig-host boundary.rs:450) | construct: crates/jig-host/src/turns.rs:171 (prod), crates/jig-host/src/tests.rs:154, tests/host/src/turn_world.rs:218, :548, tests/host/tests/memory.rs:394, tests/ops/src/conformance/scripts.rs:216; read: examples/ops/domain/src/route.rs:1437-1450 | 7 |
| ops `Payload::Turn` (examples/ops/domain/src/boundary.rs:170) | examples/ops/domain/src/route.rs:274, :1269, :1297, :1314, :1438, :1454; assemble.rs:499 | 7 |
| test root `Payload::Turn` (testing/jig-test-domain/src/lib.rs:518) and `Event::Turn` (lib.rs:244) | Payload: lib.rs:800, routes.rs:455, :466, :483, :500. Event::Turn: lib.rs:594; tests/core/src/peers.rs:82, world.rs:544, :645, effects.rs:1217; tests/core/tests/projections.rs:280, fuzzy_watches.rs:30, store.rs:76; patterns at tests/core/src/observations.rs:362, reads.rs:78, watches.rs:190 | 5 + 11 |
| `jig_fake_workers::Script::Turn`/`Up::Turn` (testing/jig-fake-workers/src/lib.rs:58, :107) | lib.rs:248, :254, :395-403; tests/core/src/peers.rs:82; Script::Turn: testing/jig-fake-workers/src/tests.rs (4), tests/client/src/joint/fixture.rs (1), tests/conformance/src/lib.rs (3), tests/core/src/faults.rs (1), tests/core/tests/peers.rs (2) | 15 |
| `core::Event::AcceptedTurn` + `opens_window` (vocabulary.rs:437-445) | crates/jig-core/src/routing/mod.rs:347, :469; crates/jig-core/src/tests/activity.rs:30; testing/jig-test-domain/src/routes.rs:488; examples/ops/domain/src/route.rs:1302 | 5 |
| `TurnRecord` + `opens_window` (stored.rs:461) | construct: runs.rs:699 (prod); tests/core/tests/store_peer.rs:36, runs.rs:123, :136; tests/client/src/joint/archive.rs:218, wire/activity.rs:332 (+:342, :365 updates). Readers: reads/rows.rs:345, :1359; run.rs:163; examples/ops/domain/src/restart.rs:96, store.rs:67; testing/jig-test-domain/src/lib.rs:59, :553 | 6 + 7 |
| `RunProof` + window start (stored.rs:517-534; `claimed` 576) | crates/jig-core/src/routing/run_preparation.rs:92-95, :407; lib.rs:672; run.rs:139; tests: crates/jig-core/src/tests.rs:455, :1719 (literals), tests/activity.rs:68, :220 | 8 |
| `Transcript` (run.rs:25-32), `begin_transcript`/`append_transcript`/`transcript_oversized`/`resumed_turns` (run.rs:137-229) | run_preparation.rs:98, :245, :335; testing/jig-test-domain/src/lib.rs:553; examples/ops/domain/src/restart.rs:96; tests/core/tests/runs.rs:122-146 | 6 |
| `Now::StartPreparation` + window start (vocabulary.rs:238) | produced run_preparation.rs:336; matched examples/ops/domain/src/route.rs:1215, :1222; testing/jig-test-domain/src/routes.rs:368, :375; tests/core/src/effects.rs:507; tests/ops/src/conformance/mod.rs:547; tests/ops/src/joint/front.rs:743; tests/client/src/joint/session.rs:788; tests/core/src/client_peer.rs:151; crates/jig-core/src/tests/completion_retry.rs:44 | 11 |

**Worlds and fixtures.**
- Core world (`tests/core`): `World::turn(number, spent, read, body)` tests/core/src/effects.rs:1215 (gains the flag); the transcript loader effects.rs:507-528 (reads from the start key); `World::new` effects.rs:161. Tests to extend: `a_chat_answers_parks_and_resumes` tests/core/tests/runs.rs:7 (today asserts the whole two-attempt conversation is resent, :23-28), `a_chat_past_the_resume_limit_starts_fresh` runs.rs:109, `loaded_transcript_pages_reject_repeated_and_reversed_turns` runs.rs:118, `an_unusable_transcript_fails_transient...` runs.rs:149; `a_restart_after_a_turn_committed_resumes_from_it` tests/core/tests/restart.rs:134 (the window start across a restart). New: a window turn mid-conversation, then a resume that sends only from it, and a restart between.
- Hub world (`tests/host`): `turn_world::World` turn helpers tests/host/src/turn_world.rs:218, :547; `a_resumed_run_starts_with_its_turn_bodies_and_answered_calls` tests/host/tests/turns.rs:37 (the hub passes the flag through untouched).
- ops world: the scripted agent's `turn()` tests/ops/src/conformance/scripts.rs:401-468 hard-codes `TurnKind::Ordinary` with no affinity at place 1 (:441-443), which smith's `decode_transcript` would refuse; its loader tests/ops/src/conformance/mod.rs:547-566.
- Core unit: crates/jig-core/src/tests/activity.rs:16-120 (TurnPayload/AcceptedTurn).

**Estimate.** About 100–130 production lines: jig-host ≈5; core ≈70 (stored.rs ≈10, run.rs ≈35, run_preparation.rs ≈10, runs.rs ≈8, vocabulary ≈4, lib.rs ≈3); ops root ≈15; jig-test-domain ≈10 (kit).

**Dependencies.** It touches the same `jig_host::Turn` and test sites as 6.3's "refusals carried with turns" (hub `Turn` gains refused names), so land them in one increment or back to back. It shares `RunProof`/`run_preparation.rs:92-95` with item 7 (the conversation's first attempt for the identity): `transcript_from` keeps that meaning and the window start is a separate field. It is the domain half of item 11's "turns saved with `opens_window` ... resume reads the current window", so 6(f) lands first and item 11 keys the store on it.

### 6(g) Facts as jig's typed activity, long operations

**Design.**
- hosts.md 6.5: "Activity, from every host, carries no content: the run's phase; its long operations, each begun with what it is (its checks, an LLM completion, a command) and its span, and ended ...; and counts from smith's channel facts, each a kind, its time and a count ..., which the root decodes by hand-out before the hub sees them (6.8)." "Forwarding is best effort: a bounded queue ... sent on an open link only, after the hello."
- hosts.md 6.8: "each of smith's channel facts is decoded into jig's typed activity (6.5), so the hub and jig's channel carry jig's activity, never smith's facts."
- channel.md 6.4: "Activity is the fleet's typed report, which every host gives ... The fact record carries jig's activity, never smith's bytes."

**smith offers.** From agent processes, `parent::Request::Told { client, body: Box<[u8]> }` (smith `crates/smith-host-domain/src/parent.rs:108-113`), whose body is a `smith_channel::Fact { kind: FactKind, elapsed: Duration, count: u64 }` (`crates/smith-channel/src/generated/v2.rs:3446-3480`; kinds `Admitted, Ended, LlmStarted, LlmRetried, LlmFinished, ToolStarted, ToolFinished, CheckStarted, CheckFinished, TextArrived`, v2.rs:3367-3388), decoded with `smith_channel::Fact::decode(&limits, &mut Reader)` (as smith-host-protocol does, `component.rs:491`). smith-protocol-channel's projection `facts::project` is `pub(crate)` (`facts.rs:11`), so there is **no public `smith-protocol-channel` function for it**: jig's hand-out would call smith-channel's codec directly, or smith exports one. Long operations: `parent::Request::Long { client, operation: smith_domain::Operation {Checks, Completion, Command}, span: Duration }` and `LongDone { client }` (parent.rs:88-91; `crates/smith-domain/src/boundary.rs:414-421`). In one process: `smith_inline_agent::Domain::pop_fact(client) -> smith::Fact` (`crates/smith-inline-agent/src/domain.rs:136`), `pop_content(client) -> smith::Content` (:145).

**Code today: partly built (the fleet's typed report and the chip), with no producer in any root.**
- Hub: agent facts are **opaque bytes**: `Event::Facts { owner, fact: Box<[u8]> }` (crates/jig-host/src/boundary.rs:43-46), `FromAgent::Facts` (:324-327, mapped :364), kept by `hosted::told` (crates/jig-host/src/hosted.rs:1311-1317) as `Told { run, attempt, fact: Box<[u8]> }` (crates/jig-host/src/facts.rs:40-45) in `AgentFacts`, bounded by `told`/`fact_bytes` (crates/jig-host/src/limits.rs:45-47, worst case :90-91), drained by `pop_told` (crates/jig-host/src/domain.rs:156). The hub's own lifecycle `Fact` (`Admitted, Prepared, Started, Waiting, Resumed, Parked, Ended, Failed`; facts.rs:20-37, pushed at hosted.rs:219, :279, :414, :492, :558, :623, :975, :979, :983) is drained by `pop_fact` (domain.rs:144). **No long operations**: `FromAgent` has no `Long`/`LongDone` (boundary.rs:308-354).
- **No root drains either hub queue**: `pop_told`/`pop_fact` of the hub are called only by tests/host/src/turn_world.rs:529, :533 and tests/host/src/world.rs:537. The ops root (examples/ops/domain/src/route.rs `host_out` :1413-1525) never reads hub facts.
- Fleet: a typed `Activity { Preparing, Starting, Active, Writing, Tool{call, tool}, Waiting, Ending }` and `Report { Started, Activity }` (crates/jig-core-fleet/src/progress.rs:11-32), taken by `Event::Observe` (crates/jig-core-fleet/src/boundary.rs:267-272; `progress::received` progress.rs:48-122, fenced by `call::owned` :57), shown by the core's chip (crates/jig-core/src/documents/chip_phase.rs:178-190). Missing: long operations (kind + span, begun/ended) and counts (kind, time, count). Beside it is a second, opaque path: `Event::Told { fact: Token }` (boundary.rs:274-279) → `Request::Told` (:460-464) via `call::told` (crates/jig-core-fleet/src/call.rs:181-192), which the core treats as **`unreachable!`** (crates/jig-core/src/routing/children.rs:286-290; the event is listed at :378). Only the fleet world sends it (tests/fleet/src/world.rs:916). `Observe` is fed only by tests (crates/jig-core/src/tests/chip_phase.rs:103-140; tests/client/src/joint/stories.rs:25-29, :183; tests/fleet/tests/memory.rs:160). The core discards the fleet's own content-free facts (`drain_facts`, crates/jig-core/src/lib.rs:595-611).

**Gap.** (1) One typed jig activity vocabulary on the hub and the fleet: phase, long operation begun (`Checks | Completion | Command`, span) and ended, and count (kind, elapsed, count). It replaces `FromAgent::Facts`/`Event::Facts`/`Told` bytes and the fleet's `Told` token path. (2) A hub output (or a drained queue) of `(run, attempt, activity)` that the engine's root translates into `fleet::Event::Observe` as it does its other hub requests (route.rs:1425-1503), "after the hello" (6.1). (3) `progress::received` and `chip_phase` learn long operations and counts. (4) Ops runs no agent, so the hand-out decoding of smith's facts is the worker root's, not built in jig (no jig crate composes smith's agent capability). In jig-protocol it lives in the worker-link/routing (protocol/README.md §5) and the reference service.

**Types and functions that change, with their users:**
| Changes | Users | Count |
|---|---|---|
| `jig_host::Event::Facts`/`FromAgent::Facts` → typed (boundary.rs:43, :324, :364) | crates/jig-host/src/domain.rs:224; crates/jig-host/src/tests.rs:287-289; tests/host/src/turn_world.rs:223; tests/host/src/world.rs:625 | 4 |
| `jig_host::Told` (facts.rs:41), `AgentFacts` (:48-74), `pop_told` (domain.rs:156), `Limits::{told, fact_bytes}` (limits.rs:45-47) | hosted.rs:1316; crates/jig-host/src/tests.rs:292-299; tests/host/src/turn_world.rs:529; re-export lib.rs:34 | 4 |
| hub lifecycle `Fact` → activity phases (facts.rs:20) | 9 pushes in hosted.rs (above); tests/host/src/world.rs:537, turn_world.rs:533 | 11 |
| `fleet::Activity`/`Report` (progress.rs:11-32) + long ops/counts; `received` (:48) | crates/jig-core/src/documents/chip_phase.rs:180-190; crates/jig-core-fleet/src/tests/progress.rs (all 103 lines); crates/jig-core/src/tests/chip_phase.rs:103-140; tests/client/src/joint/stories.rs:25-29, :183; tests/client/src/joint/wire/chips.rs:205-209; tests/fleet/tests/memory.rs:160-164 | 7 files |
| fleet `Event::Told`/`Request::Told` removed (boundary.rs:274, :460) | crates/jig-core-fleet/src/domain.rs:357, call.rs:181-192; crates/jig-core/src/routing/children.rs:290, :378; tests/fleet/src/world.rs:916, :1029, :1315; tests/fleet/src/turns.rs:267; tests/fleet/tests/memory.rs:312 | 9 |
| ops root: hub activity → `fleet::Event::Observe` | examples/ops/domain/src/route.rs `host_out` :1413-1525 (new arm), and the drain beside `host_out` | 1 |

**Worlds and fixtures.** Fleet world tests/fleet/src/world.rs (Observe is in its exhaustive arm :1316; Told generation :916, :1029); fleet unit crates/jig-core-fleet/src/tests/progress.rs; hub `turn_world` tests/host/src/turn_world.rs:223 (`Facts` send) and :529 (drain), and the hub's best-effort-facts check named in tests/host/src/lib.rs:30-32; core chip tests crates/jig-core/src/tests/chip_phase.rs:103-140; client joint world `report(world, assignment, activity)` tests/client/src/joint/stories.rs:25-29 (the client's activity line). The ops world (tests/ops/src/conformance/scripts.rs `poll` :165-253) gains a long-op/count stream beside `stream()` :372.

**Estimate.** About 250–300 production lines: hub ≈90 (vocabulary ≈40, typed queue ≈30, lifecycle → phase ≈20); fleet ≈80 (Activity ≈30, received ≈40, drop Told ≈-15); core ≈25 (chip_phase, children.rs); ops root ≈40.

**Dependencies.** It needs the hub's link vocabulary of 6.1 (hello first, "facts go only after it", hosts.md 6.1). Item 8's text transport rides the same fact path, so 6(g) lands before item 8's transport half.

### 6(h) smith's payloads: bytes in the hub, handed out by the root

**Design.**
- hosts.md 6.8: "**Bytes in the hub.** The hub links no smith crate. It carries as bytes it never reads the charter, a resumed run's transcript, each turn's body, call names and a run's result, and the run's identity". "**Translated by hand-out.** The root ... hands each payload out to its protocol layer, whose routing decodes or encodes it with `smith-protocol-channel`'s functions ... and hands it back as an event of the root". "**Who links smith:** `jig-charter` ..., a client's decoding of turns ..., and an application's roots that compose smith's agent capability".
- root.md 5: "smith's payloads, by hand-out: ... it never encodes or decodes one itself."

**What the hub carries now: bytes, as designed.** Charter `Box<[u8]>` (crates/jig-host/src/boundary.rs:415, `Start.charter` :181, `ToAgent::Start` :285); turns `Box<[Box<[u8]>]>` (:183, :424); turn body (:454); call names `Box<[u8]>` (`Called.call` :24, `Relay.call` :157, `AnsweredCall.name` :433); tool, input (:159-161, :492-496); result `Finish::Ended { outcome: Box<[u8]> }` (:459) / `Ending::Ended { outcome }` (:474); facts bytes (:45). `jig-host` depends on skein-lib only (crates/jig-host/Cargo.toml), enforced by `host_hub_links_only_skein` (tests/boundary/src/tests.rs:260-275). Missing beside the bytes: `opens_window` (6(f)), refused names and the start's messages (6.3, 6.4), the run's identity (item 7; `Start` carries `activation: u64` only, :182).

**What the ops root does with smith's values today (examples/ops/domain), all in the root's step, none by hand-out:**
- **Encodes the charter itself**: `assemble::assignment` (examples/ops/domain/src/assemble.rs:48-112) builds `jig_charter::Charter` and calls `charter::charter` (:58-84) and `charter::encode(value, &domain.charter_endpoints, &env.limits.charter)` (:85), from `Now::RunPrepared` (route.rs:1245-1258). Design: "`jig-charter` ... which the engine's protocol layer calls for every run" (hosts.md 8).
- **Decodes a run's result itself, before the fleet fences it**: `host_out` on `host::Request::Answer` (route.rs:1462-1477) calls `assemble::ending` (assemble.rs:244-258) → `decode_result` (:260-284) → `smith_charter::RunResult::decode(&smith_charter::v2::CEILINGS, ...)` (:262). The typed `tasks::End` is then stored in `Payload::Answer`, and every hub answer is reported to the fleet as `fleet::Answer::Ended` (route.rs:1474), including refusals and failures. (Refusals are 6.3's; noted only.)
- **Reads call names itself**: `translate::call_name` (examples/ops/domain/src/translate.rs:249-269, 16 bytes LE: activation, completion, position) in `assemble::call` (assemble.rs:426) for the `CallKey`.
- **Decodes tools' inputs itself** (item 9).
- **Links smith**: `smith-charter` (examples/ops/domain/Cargo.toml:26), `jig-charter` (:23), `skein-json` (:12); `smith_charter::v2::Limits` in its limits (examples/ops/domain/src/limits.rs:26-27). Ops composes no smith agent capability ("Ops runs no agent in its engine", boundary.rs:81-82), so by hosts.md 6.8 its root should link none of them.
- Passes the hub's `ToAgent::Start` bytes out untouched to whoever hosts the agent (`Output::Agent`, boundary.rs:157-158; route.rs:1421). The ops world's scripted agent reads the charter bytes raw to pick its role (tests/ops/src/conformance/scripts.rs:94-96, `role` :255) and writes smith transcript v3 turn bytes directly (scripts.rs:401-468).
- Text and calls streamed by the agent bypass the hub: `Event::AgentContent` (boundary.rs:86, :110-116) → `agent_content` (route.rs:1527-1585) → `core::Event::RunText`/`RunCall` (see item 8).

**smith functions the hand-out would call** (worker root and an engine root with `smith-inline-agent`):
- Down: `smith_protocol_channel::decode_charter` (smith `crates/smith-protocol-channel/src/translate.rs:272`), `decode_transcript` (`transcript.rs:18`), `start_context` (`translate.rs:50`). The call names are decoded into `smith_host_domain::CallName { activation, completion, position }` (`crates/smith-host-domain/src/channel.rs:105-114`).
- Up: `encode_turn` (`turn.rs:17`), `encode_result` (`translate.rs:439`), `smith_channel::Fact::decode` (6(g)).
- Engine: `decode_declared(bytes, &smith_charter::v2::Limits) -> smith_domain::run::outcome::Declared` (`result.rs:9`) for jig-charter's reader (item 9), and `encode_charter` (`charter.rs:16`) if jig-charter does not keep its own `encode`.

**Gap.** For ops (an engine root with no agent), hand-out applies to the charter (encode), the result (decode, after the fence) and the call (decode, after the fence; item 9). That means a root `Output` per hand-out and an `Event` back, the ops world standing in for the protocol layer's routing. The worker root's and the inline root's hand-outs of turns, transcripts, facts and call names are protocol-layer code (the reference service and the worker link), and no domain change is needed beyond the hub fields above. Then the ops domain drops `smith-charter`, `jig-charter` and `skein-json` (Cargo.toml:12, :23, :26).

**Types and functions that change, with their users:**
| Changes | Users | Count |
|---|---|---|
| ops `assemble::assignment` (assemble.rs:48) → charter handed out | route.rs:1248 | 1 |
| ops `assemble::ending`/`decode_result` (assemble.rs:244, :260) → handed out at `Now::AnswerPayload` (route.rs:1276-1292) | route.rs:1463 | 1 |
| ops `Payload::Answer` (boundary.rs:171) keeps the hub's bytes until `AnswerPayload` | route.rs:274, :1278-1279, :1316, :1466; assemble.rs:500 | 5 |
| ops `Output`/`Event` gain hand-out variants (boundary.rs:51-107, :132-165) | exhaustive matches: examples/ops/domain/src/domain.rs (1), examples/ops/domain/src/tests.rs (10 `Output::Stop` arms), tests/ops/src/conformance/mod.rs (2), tests/ops/src/joint/engine.rs (2), tests/ops/src/joint/front.rs (1) | 5 files |
| ops `Domain.charter_endpoints` (domain.rs:51, :65, :94-99, :130), `Limits.charter` (limits.rs:27) | examples/ops/domain/src/tests.rs:50; tests/ops/src/conformance/config.rs:93 | 2 |

**Worlds and fixtures.** ops world: `Ops` loop tests/ops/src/conformance/mod.rs (Output arms; `CoreLoad` handling :547) and joint tests/ops/src/joint/engine.rs, front.rs; agent scripts tests/ops/src/conformance/scripts.rs (`result()` :494-528 builds smith result bytes with `smith_charter`). Root unit examples/ops/domain/src/tests.rs `a_committed_chat_assignment_starts_its_agent_through_the_engine_hub` :407 and `started_chat` :306. Stories: tests/ops/tests/stories.rs:59 `the_alert_storm_reaches_triage_and_the_person_accepts_the_production_restart`.

**Estimate.** ≈80–120 production lines in the ops root (hand-out variants and arms), with ≈60 lines (`decode_result`, `model`, `contract` assemble.rs:114-176, 260-284) moved to the protocol side.

**Dependencies.** Shares the call/result path with item 9 (same hand-out mechanism; design channel.md 6.1: "by the path of host tools' calls (6.3)"): land item 9's mechanism first, then the result and charter on it. The renames (item 12) decide whether the reader goes into `jig-charter` or `jig-protocol-charter`.

## Item 7. A run's identity inputs: the assignment carrying the conversation's first attempt

**Design.**
- `domain/engine.md` 7.1: the assignment carries "its run and attempt, the core's names for them, and its conversation's first attempt, from which the engine's protocol layer derives the run's identity ...: 16 bytes from the deployment's id, the task's number and that attempt, the same for every attempt of a conversation ... and never shared by two. No domain holds it".
- `protocol/channel.md` 1 (lines 411–418): "the first 16 bytes of a SHA-256 digest of the deployment's id, the task's number and its conversation's first attempt ... derived by the engine's protocol layer for each assignment, and for runs in the engine alike ... the store keeps nothing for it but the conversation's first attempt." channel.md 12 (646–648): "The domain gives the records their fields: the assignment's current window, start's messages and conversation's first attempt".
- `domain/hosts.md` 2 (line 75): the hub's assignment carries "the run's identity"; 6.8 (line 499): the hub carries it as bytes it never reads. smith `domain/host.md` 2: the start carries "the run's identity ... the host keeps it the same across the run's starts, resumes included"; smith `protocol/limits.md` 2.4: "16 opaque bytes" (`smith_domain::run::Identity`, smith-host-domain/src/channel.rs:23 `identity: Option<...>`).

**Today: partly.** The input exists as `RunProof.transcript_from` ("Earliest attempt whose turns belong to this usable conversation; zero keeps all", crates/jig-core/src/stored.rs:522–523), saved with the claim (crates/jig-core/src/routing/tasks.rs:661–663) and restored (lib.rs:655–668, validated `transcript_from > attempt` refused, lib.rs:672). Set in `workspace_prepared`: `let transcript_from = if resume { transcript.from } else { attempt };` (crates/jig-core/src/routing/run_preparation.rs:92–96), where `transcript.from` is the previous proof's value or 0 (`begin_transcript`, crates/jig-core/src/run.rs:136–158). Gaps:
1. **The first claim of a resuming charter records 0, not an attempt**: no proof exists yet, so `from = 0` (run.rs:138–141) and `resume` is true (run_preparation.rs:89–91), so `transcript_from = 0` for the conversation's whole life. The identity's input must be an attempt number.
2. **A refused claim forgets the conversation.** Preparation replaces the task's proof in memory before the claim commits (run_preparation.rs:93–96); every claim failure then removes it entirely — `route_claim_refused` (run_preparation.rs:386–390), `WriterWaiting` (routing/tasks.rs:227–230), refused payload (routing/tasks.rs:1119–1121) — although the store still holds the previous committed `RunProof`. The next `begin_transcript` finds no proof and starts from 0: the resumed run reads every turn of the task, earlier conversations included, and the conversation's first attempt is lost (memory and store disagree until a restart).
3. **Not carried.** `Now::RunPrepared` (vocabulary.rs:254–264) has no first attempt; `host::RunAssignment` (crates/jig-host/src/boundary.rs:406–417) and `host::Request::Start` / `ToAgent::Start` (boundary.rs:178–186, 282–290) have no identity; `fleet::Assignment` (crates/jig-core-fleet/src/boundary.rs:104–110) is two tokens. Nothing derives the 16 bytes (no SHA-256 over deployment/task/attempt anywhere; `sha2` is already a workspace dependency, Cargo.toml).

**Types and functions that change, with their users.**
- `RunProof::claimed` (stored.rs:576) — 1 caller (run_preparation.rs:95); `RunProof` literal constructions in tests: crates/jig-core/src/tests.rs:458, 1722 (both `transcript_from: 0`). `transcript_from` reads: run.rs:139, run_preparation.rs:92, 407, lib.rs:672 (4).
- Claim failure paths that drop the proof (restore the previous one instead): run_preparation.rs:389; routing/tasks.rs:229, 1120 (3).
- `Now::RunPrepared` + `conversation`/first-attempt field: constructor run_preparation.rs:131; field-naming destructurers examples/ops/domain/src/route.rs:1245 and testing/jig-test-domain/src/routes.rs:421 (both `..`, so they compile unchanged but must pass it on); `..` matches at crates/jig-core/src/tests/completion_retry.rs:52, tests/core/src/client_peer.rs:159, tests/ops/src/joint/engine.rs:342, tests/ops/src/joint/front.rs:751, tests/client/src/joint/session.rs:796 (no change).
- Root assignments: jig-test-domain `Assignment` (testing/jig-test-domain/src/lib.rs:133–146; built routes.rs:421–437; read by tests/core/src/peers.rs:255–262, world.rs:634, effects.rs:719, observations.rs:835–846); ops `domain::Assignment { attempt, value, inbox }` (examples/ops/domain/src/route.rs:1250–1252) and `assemble::assignment` (assemble.rs:48–113), which derives the identity as the engine's protocol layer would.
- `host::RunAssignment` + `identity` (boundary.rs:406): 7 constructions — crates/jig-host/src/tests.rs:79; tests/host/src/turn_world.rs:154, 176; tests/host/src/engine.rs:287; tests/host/tests/memory.rs:134; tests/host/src/world.rs:549; examples/ops/domain/src/assemble.rs:98; 1 full destructure crates/jig-host/src/hosted.rs:202 (kept in `Hosted`, hosted.rs:203–214, until `start_prepared` builds `Request::Start`, hosted.rs:424–432); admission check crates/jig-host/src/assignment.rs:8 (fixed size, nothing to check if `[u8; 16]`).
- `host::Request::Start` / `ToAgent::Start` + identity: full destructures crates/jig-host/src/boundary.rs:382–383, tests/host/src/turn_world.rs:416; `..` matches crates/jig-host/src/tests.rs:68, 106, 146, 360; tests/host/src/engine.rs:211; tests/host/tests/memory.rs:103; tests/host/src/parent.rs:272; tests/host/src/world.rs:732, 829; examples/ops/domain/src/route.rs:1516; tests/ops/src/conformance/scripts.rs:94; examples/ops/domain/src/tests.rs:372.
- jig-fake-workers `Assignment { task, attempt, budget, transcript }` (testing/jig-fake-workers/src/lib.rs:17–22), filled at tests/core/src/peers.rs:256–261 — to let the fake worker assert a stable identity across resumes.

**Worlds and fixtures.** tests/core: `effects::World` (tests/core/src/effects.rs:27, field `assigned` 35) with the resume tests tests/core/tests/runs.rs:7 (`a_chat_answers_parks_and_resumes`), 109 (`a_chat_past_the_resume_limit_starts_fresh`), 149 (`an_unusable_transcript_fails_transient_and_the_next_conversation_starts_fresh`), tests/core/tests/restart.rs:134 (`a_restart_after_a_turn_committed_resumes_from_it`); a claim-refused-then-resume story needs a connector that refuses a claim (`core::Event::ClaimRefused`, run_preparation.rs:386). Referee: the conformance promise "a conversation's attempts share one first attempt; a fresh start takes its own" fits `Observed::Assigned` (testing/jig-conformance/src/referee.rs:273, 874–895; observed in tests/core/src/observations.rs:835–846 and tests/ops/src/conformance/mod.rs:337). Hub: tests/host world (tests/host/src/world.rs:549, parent.rs:272) for identity passed through to the agent unchanged; ops scripted agent tests/ops/src/conformance/scripts.rs:94.

**Estimate.** jig-core ≈25–40 (first claim records its own attempt; a refused claim restores the previous proof rather than erasing it; the field on `RunPrepared`); jig-host ≈15–20 (identity on `RunAssignment`, `Hosted`, `Start`/`ToAgent::Start`); identity derivation ≈15 in ops's `assemble.rs` (or the protocol crate that later owns it); jig-test-domain ≈5. Total ≈60–80. Files over 800 touched: crates/jig-host/src/hosted.rs (1340), crates/jig-core/src/routing/tasks.rs (1414), examples/ops/domain/src/route.rs (1585), testing/jig-test-domain/src/routes.rs (1005) and lib.rs (1154).

**Dependencies.** None upstream. It must land before item 6's windows (engine.md 7.2: "the current window's start kept per task"), which adds a second per-task position beside `transcript_from` and changes `begin_transcript`/`append_transcript` (run.rs:136–208); fixing the 0 sentinel and the refused-claim loss first keeps "conversation's first attempt" and "current window's start" distinct. It touches the same `Now::RunPrepared` / `host::RunAssignment` / `Request::Start` constructors as items 4 and 5 (grants per account and the application's credentials) and item 6 (the start's messages): batching those field additions in one increment avoids editing the 7 `RunAssignment` constructions three times.

## Item 8. Text by retry, a gap marked cut

**Design.**
- engine.md 11: "A run's text as it is written is the one stream: kept per live run being watched, within a bound, named by the turn and the retry the host reports it for (hosts.md, 6.5), and delivered from the offset each watcher has; never stored, and marked provisional."
- hosts.md 6.5: "The engine's views keep a run's text by turn, retry and offset ...: a later retry replaces an earlier one's text, and since facts may be dropped, text that starts past what the engine holds of its retry marks that turn's text cut."
- channel.md 6.4: "text that starts past what they hold of a retry, because facts were dropped between, marks that turn's text cut, as text past its bound is; a new turn starts at zero under its own name."
- STATUS engine.md row: "8: ... a run's text from hosts through the fleet"; "11: text by retry, a gap marked cut".

**Code today: retry built, gap not, transport bypasses hub and fleet.**
- Naming by retry is built. `views::TurnName { attempt, turn, retry }` (crates/jig-core-views/src/boundary.rs:48-52). `text::written` (crates/jig-core-views/src/text.rs:50-157) treats a new `TurnName`, including a later retry, as a new turn that replaces the earlier text (:97-113), and tests cover it (crates/jig-core-views/src/tests/completion_retry.rs:24-37; tests/views/tests/completion_retry.rs:12-25; tests/core/tests/completion_retry.rs:19-58).
- **A gap is dropped, not marked.** Within one `TurnName`, `offset > run.to` → `return` (text.rs:114-116), and every later piece of that retry is dropped too, since `run.to` never advances. The test asserts this today: `report(&mut domain, 1, 21, b"gap").is_empty()` (crates/jig-core-views/src/tests/completion_retry.rs:102). The module doc says "stale turns and gaps are ignored" (text.rs:7). `cut` exists only as derived booleans: a stream's `cut: from > offset` (text.rs:294) and `writing()`'s `cut: from != 0` (text.rs:347). A first report past zero is shown cut (`a_late_first_report_after_restart_states_its_retained_start`, completion_retry.rs:85-95), but nothing marks a mid-retry gap. The views world's referee **forbids gaps**: "the scripted host writes consecutive offsets" (tests/views/src/referee.rs:155).
- The core fences text by the proof (current attempt, unterminated, `turn > kept turn`) in `run_text` (crates/jig-core/src/routing/views.rs:463-486), not by the fleet's channel.
- **Not through the hub or the fleet**: the hub has no text input (`FromAgent`, crates/jig-host/src/boundary.rs:308-354) and the fleet has no text event (boundary.rs:135-280). Ops takes `Event::AgentContent { owner, AgentContent::Text { turn, retry, offset, text } | Call(views::Call) }` (examples/ops/domain/src/boundary.rs:86, :110-116) straight into `core::step(... Event::RunText ...)` (route.rs:1527-1585). It maps the owner with `domain.host.hosting(owner)` (:1535) and slices by the views' bound itself (:1557-1575). The test root does the same, with retry fixed at 0: `Event::Text { task, attempt, turn, offset, text }` (testing/jig-test-domain/src/lib.rs:236-242) → `RunText { retry: 0 }` (:580-592); the fake workers' `Up::Text`/`Script::Text` carry no retry (testing/jig-fake-workers/src/lib.rs:56, :101).
- smith: from agent processes only `FactKind::TextArrived` counts (no content; smith `docs/design/protocol/channel.md` §8). In-process, `smith::Content::Text { owner, completion: CompletionName { sequence, retry }, text }` (smith `crates/smith-domain/src/facts.rs:23-25`; `crates/smith-domain-session/src/llm.rs:300-305`) gives a conversation-wide `sequence`, a retry and no offset. Offsets come from smith's LLM component (protocol/llm.md §3, "not planned" per STATUS §3, Agents in the engine). So in jig-protocol's scope the text's sources are the worlds; the domain change is the gap rule and the transport.

**Gap.** (1) Views: on a gap within a retry, keep the new text from its offset and mark the turn cut. That means a `cut: bool` on `text::Run` (text.rs:30-39), set on a gap and on trimming past the bound, reset on a new `TurnName`, ORed into `Request::Stream.cut` (text.rs:288-296) and `Writing.cut` (:342-349). Flip the test at completion_retry.rs:97-106 and relax the referee (tests/views/src/referee.rs:152-157). (2) Transport: a text fact on the hub (`FromAgent::Text`/`Event::Text` → a hub request, best effort, after the hello), a fenced fleet `Event::Text`/`Request::Text` (by channel and attempt, like `progress::received`'s `call::owned`), routed by the core to `views::Event::Text`. The ops root's `AgentContent` and the test root's `Event::Text` then go through the hub/fleet instead of `core::Event::RunText`. Whether `core::Event::RunText`/`RunCall` remain as root entries or become fleet-internal is the planner's choice: keeping them as the fleet's routed request keeps the views' users unchanged.

**Types and functions that change, with their users:**
| Changes | Users | Count |
|---|---|---|
| `views::text::written`/`offer`/`writing` + `Run.cut` (text.rs:30, :50, :265, :334) | crates/jig-core-views/src/domain.rs (Event::Text dispatch); tests: crates/jig-core-views/src/tests.rs (4 `Event::Text`), tests/completion_retry.rs (2), tests/views/src/referee.rs, world.rs, tests/views/tests/memory.rs, referee.rs | 9 |
| hub `FromAgent::Text`/`Event::Text` + hub request (new) | ops route.rs:200 (`Event::Agent`), :1413 `host_out`; tests/host/src/world.rs:625 (exhaustive), turn_world.rs | 4 |
| fleet `Event::Text`/`Request::Text` (new) | crates/jig-core-fleet/src/domain.rs:354-357 area; crates/jig-core/src/routing/children.rs:286-290, :378-379 (exhaustive); tests/fleet/src/world.rs:1305-1316 (exhaustive) | 3 |
| `core::Event::RunText` (vocabulary.rs:377) | crates/jig-core/src/routing/mod.rs:327, :433; examples/ops/domain/src/route.rs:1543, :1561; testing/jig-test-domain/src/lib.rs:583; crates/jig-core/src/tests.rs:2119; crates/jig-core/src/tests/completion_retry.rs (2); tests/core/tests/completion_retry.rs:8 | 9 |
| ops `Event::AgentContent`/`AgentContent` (boundary.rs:86, :111) | route.rs:200, :1527-1585; domain.rs, lib.rs re-export; tests/ops/src/conformance/scripts.rs:211, :372-399 | 9 |
| test root `Event::Text` (testing/jig-test-domain/src/lib.rs:236) + retry; fake `Up::Text`/`Script::Text` (testing/jig-fake-workers/src/lib.rs:56, :101) | testing/jig-fake-workers/src/lib.rs, tests.rs (2); tests/client/src/joint/fixture.rs; tests/core/src/peers.rs (2), observations.rs, reads.rs, watches.rs; tests/core/tests/fuzzy_watches.rs, run_positions.rs, stories/watches.rs | 13 |

**Worlds and fixtures.** Views world `jig_views_world::World::text(run, name, text, call)` (tests/views/src/world.rs) and `open` (tests/views/tests/completion_retry.rs:8-10); its referee tests/views/src/referee.rs:152-200. Views unit `report(domain, retry, offset, text)` crates/jig-core-views/src/tests/completion_retry.rs:6-8 and `expected` :12-22. Core world `report(world, task, attempt, retry, text)` tests/core/tests/completion_retry.rs:7-17, with the `watches::{open, world}` fixtures (tests/core/src/watches.rs). Fleet world for the fenced text (tests/fleet/src/world.rs). ops world `stream()` tests/ops/src/conformance/scripts.rs:372-399 (add a retry and a dropped piece).

**Estimate.** About 120–160 production lines: views ≈25; hub ≈30; fleet ≈40; core routing ≈15; ops root ≈20 net (`agent_content` shrinks).

**Dependencies.** The gap rule is independent and can land first. The transport depends on 6(g) (the typed fact path, the fleet's fencing of facts) and on 6.1's hello ("facts go only after it").

## Item 9. Host tools decoded outside the root

**Design.**
- hosts.md 8: "Each host tool's schema and description are written once, with the function that decodes its calls and the one that renders its answers ... The engine's protocol layer decodes every call the fleet has admitted, from workers and from the engine's own hub alike, so no domain parses a call's input."
- channel.md 6.3: "The core then hands the admitted call out for decoding, and the engine's protocol layer decodes its input with the decoder written beside the tool's schema: `jig-protocol-tools`' for the core's own tools, the application's for its connectors' reads and effects ... An answer crosses as the engine's text ... rendered in the engine's protocol layer".
- channel.md 7: call names "are decoded ... by `jig-charter` in the engine, which reads the call's key (its completion and position) as the protocol layer decodes the call (6.3)".
- channel.md 6.1: "`jig-charter`'s reader decodes [a run's result] in the engine's protocol layer ... by the path of host tools' calls (6.3): the core hands an answer's result out once the fleet has fenced it, and the typed result comes back. A result that does not decode fails its attempt as invalid."
- root.md 11: a root never "read[s] or rewrite[s] a value it carries between children"; STATUS root.md row: "11, 12: the reference root decodes host tools' calls, a call's key and a run's result itself".

**Code today: not built; the reference root does all three itself.**
- The fleet fences first, as designed: `fleet::Request::Relay` → core `Now::Call { to, run, attempt, call: fleet::Call }` (crates/jig-core/src/routing/children.rs:269-271; vocabulary.rs:192). Then the ops root decodes in its step: `core::Now::Call` → `assemble::call` (examples/ops/domain/src/route.rs:1362-1364; examples/ops/domain/src/assemble.rs:417-486):
  - the call's key from `translate::call_name` (translate.rs:249-269) at assemble.rs:426-437;
  - the input with `translate::decode(tool, input) -> Result<Decoded, Problem>` (translate.rs:877-1011). That uses its own bounded JSON parser `translate::json` (:281-504, on `skein_json::tokenizer`), `nested` (:506-825: delegates, amendment, spec, contract, authority, grants, wake policy) and `notes` (:1026-1142), into `Decoded { Read(observability::Read), Action(core::NamedAction), Delegate(Box<[core::Delegate]>), Effect{..} }` (:826-836);
  - the routing on what comes back, at assemble.rs:438-485 (reads → observability; actions → `core::Event::NamedAction`; delegates → `DelegateValidated`; effects → `Event::Effect`; errors → `CallPart::Unavailable`, not "an error naming what is wrong" as channel.md 6.3 asks).
- Tool declarations: `translate::tools` (translate.rs:837-876) declares every tool with the schema `{"type":"object","additionalProperties":true}` and its name as its description, so there are no real schemas or descriptions.
- Answers rendered in the root: `assemble::answer`/`render`/`delegated`/`notes` (assemble.rs:286-416) and `host_answer` (:230-242, which prepends an error mark byte inside the hub's bytes); used at route.rs:325, :1199, :1383; assemble.rs:497.
- The result is decoded in the root before the fence: `assemble::ending`/`decode_result` (assemble.rs:244-284) at route.rs:1463 (6(h)).
- Config templates are parsed with the same JSON code in the root: `translate::triage` (:1168-1190) and `lead` (:1230-1239), called at route.rs:1139, :1144, :1161. This is configuration parsing (configuration.md), also out of the root by the same rule; flagged for the planner.
- jig-charter has only `charter` (crates/jig-charter/src/charter.rs:86) and `encode` (encode.rs:172): **no result reader and no call-key reader** (lib.rs:15-20). `jig-protocol-tools` does not exist.
- The other roots already hand out: the reference test root passes `Now::Call` to its world (testing/jig-test-domain/src/routes.rs:532-542), and the worlds answer with typed `core::Event::NamedAction`/`DelegateValidated` (22 constructions across tests/core, tests/client, tests/conformance, tests/ops/src/joint; e.g. tests/core/src/peers.rs, tests/core/tests/stories.rs ×6). **The core's vocabulary needs no change; only the ops root and new protocol crates do.**

**Constraint.** `hosts_and_translation_use_no_core_crates_even_when_empty` (tests/boundary/src/tests.rs:243-257) forbids `jig-charter` any `jig-core*` dependency. So a result reader returning `tasks::TaskResult` breaks it. Either the reader returns its own type (e.g. smith's `smith_domain_run::outcome::Declared`, through `smith_protocol_channel::decode_declared` (smith `result.rs:9`), or `smith_charter::RunResult::decode` plus a trailing-bytes check, as ops does at assemble.rs:262) and the protocol routing maps it to `tasks::End`, or the rule is rewritten with the rename (README.md 15: `jig-charter` → `jig-protocol-charter`; protocol/README.md §5: a protocol crate "depends on ... the domain vocabulary it translates"). The new `jig-protocol-tools` needs a new boundary rule of its own: it links no smith (protocol/README.md §5, "`jig-protocol-tools` gives its schemas as bytes the charter declares").

**Gap.** (1) `jig-protocol-tools` (new, engine protocol): the core's tools (`delegate, message, cancel, release, introduce, decide, withdraw, subscribe, unsubscribe, amend, decide_escalation, propose, note, recall`, translate.rs:838-856) with real schemas and descriptions, decoders into `core::NamedAction`/`core::Delegate`/notes requests, and answer renderers. That is moved from ops translate.rs:271-1142 and assemble.rs:230-416. (2) The application's decoders for its connectors' tools (`read_logs, read_series, read_alerts, effect_restart, propose_restart`, translate.rs:879-912) in an ops protocol crate (`jig-ops-protocol-*`, README.md 15). (3) jig-charter: a call-name codec (activation, completion, position; replaces translate.rs:249-269 and the world's `name()` tests/ops/src/conformance/scripts.rs:364-369) and a result reader. (4) The ops root hands out `Now::Call` (route.rs:1362) and the answer's result at `Now::AnswerPayload` (route.rs:1276) as `Output`s, and takes back typed events. It keeps `domain.calls`/`host_calls` bindings (assemble.rs:466; route.rs:1426-1429). (5) Errors are answered "naming what is wrong", replacing `CallPart::Unavailable` (assemble.rs:484) and the `"unavailable"` read answers (:428-435, :461).

**Types and functions that change, with their users:**
| Changes | Users | Count |
|---|---|---|
| ops `translate::decode`/`Decoded`/`Problem`/`json`/`nested`/`notes` (translate.rs:271-1142) → protocol crates | assemble.rs:440-461, :468-484; translate.rs:1169-1170, :1231-1232 (triage/lead) | 4 sites |
| ops `translate::call_name`/`CallName` (translate.rs:249-269) → jig-charter | assemble.rs:426; test helper examples/ops/domain/src/tests.rs:397-405; world encoder tests/ops/src/conformance/scripts.rs:364-369 | 3 |
| ops `translate::tools` (translate.rs:837) → jig-protocol-tools + ops protocol | assemble.rs:66 | 1 |
| ops `assemble::call` (assemble.rs:417) → hand-out | route.rs:1363 | 1 |
| ops `assemble::answer`/`render`/`delegated`/`notes`/`host_answer`/`read_answer` (assemble.rs:230-416, :513) → rendering handed out | route.rs:325, :1047, :1075, :1199, :1367, :1383, :1393; assemble.rs:428, :454, :462, :497 | 11 |
| ops `assemble::ending`/`decode_result` (assemble.rs:244-284) → jig-charter reader via hand-out | route.rs:1463 | 1 |
| jig-charter public surface (crates/jig-charter/src/lib.rs:15-20) + reader + call-key codec | examples/ops/domain (assemble.rs, domain.rs:51-67, limits.rs:7, :27); tests/ops/src/conformance/config.rs:93; examples/ops/domain/src/tests.rs:50; crates/jig-charter/src/tests.rs:11 | 6 |
| ops `Output`/`Event` hand-out variants | exhaustive matches: tests/ops/src/conformance/mod.rs, tests/ops/src/joint/engine.rs, tests/ops/src/joint/front.rs, examples/ops/domain/src/domain.rs, examples/ops/domain/src/tests.rs | 5 files |
| tests/boundary rule (tests/boundary/src/tests.rs:243-257) + a rule for `jig-protocol-tools` | — | 1 |

**Worlds and fixtures.** The ops world as protocol stand-in: its output loop tests/ops/src/conformance/mod.rs (add the hand-out arms that call the decoders), with the scripted calls `call(tool, input)` tests/ops/src/conformance/scripts.rs:477-479, `logs()` :480, `watch()` :530, `remediate()` :542, `provision()` :554, `delegate()` :562, `triage()` :577, `lead()` :583 (JSON inputs the decoders must accept). Stories: tests/ops/tests/stories.rs:59 (alert storm → triage → restart proposal: decode of `read_*`, `delegate`, `propose_restart`, `decide`), tests/ops/tests/fuzzy_stories.rs, fuzzy_joint.rs. Root unit: `a_tool_mutation_and_its_rendered_reply_share_one_commit` examples/ops/domain/src/tests.rs:412 (asserts rendered bytes: moves to hand-out), `call_name` helper :397. New unit suites in `jig-protocol-tools` (each tool's decode, refusal naming, render) and `jig-charter` (crates/jig-charter/src/tests.rs: result round trip with smith's `encode_result`, call-name round trip). The core and client worlds already feed typed actions (no change).

**Estimate.** ≈900–1,100 production lines moved out of the ops root (translate.rs:249-1142 ≈ 890; assemble.rs:230-416 ≈ 185), and ≈300–400 new: `jig-protocol-tools` schemas and descriptions ≈150, error naming ≈50; jig-charter reader ≈60 and call-key codec ≈30; ops root hand-out plumbing ≈100; ops protocol decoders' crate scaffolding ≈40. The ops root falls from 5,522 to ≈4,500 lines and `translate.rs` from 1,310 to ≈420.

**Dependencies.** Item 12 first, or at the same time: `jig-protocol-tools` takes its name at once, and the reader lands in `jig-charter` or `jig-protocol-charter`. 6(h)'s result and charter hand-outs reuse this mechanism. It is independent of 6(f)/6(g)/8. 6.3 touches the same `host_out` Answer arm (route.rs:1462-1477, `fleet::Answer::Ended` for every ending).

### Files over 800 lines these three items touch

examples/ops/domain/src/route.rs 1,585; crates/jig-host/src/hosted.rs 1,340; examples/ops/domain/src/translate.rs 1,310 (shrinks with item 9); tests/core/src/effects.rs 1,289; tests/fleet/src/world.rs 1,320; testing/jig-test-domain/src/lib.rs 1,154; testing/jig-test-domain/src/routes.rs 1,005; tests/host/src/world.rs 1,048; tests/ops/src/joint/front.rs 1,035; tests/ops/src/conformance/mod.rs 975; crates/jig-core/src/tests.rs 2,936; tests/client/src/joint/session.rs 1,553. Under 800 but close: crates/jig-core/src/routing/runs.rs 784, crates/jig-host/src/boundary.rs 675, crates/jig-core/src/routing/vocabulary.rs 649, crates/jig-core-fleet/src/boundary.rs 615.

## Item 10. People's and tasks' refusals, signing out

Checked on `main` 2fae240a. Paths are as built (before item 12's renames). The
unversioned draft `~/src/rust/plans/jig-protocol-implementation/jig/03-people-and-tasks.md`
and `08-client-owed.md` (drafted on `5d31617`) cover the same ground; where the
code has moved since, this section says so.

| Sub-item | State | Production lines (est.) | Lands after |
|---|---|---|---|
| 10.1 a reply naming its own sign-in | not built | ~15 | none (item 12 if renames first) |
| 10.2 escalation decided when not waiting: refused as not waiting | partly | 15–30 | none |
| 10.3 chats forgotten once ended and read | partly (mechanism only, test-driven) | 40–80 (chat mark) + 40–80 (results mark) | results-mark half wants item 11's conversations index |
| 10.4 a task's proposal nobody may accept, refused | not built (party path built) | 30–50 | none; "saying so" to an agent rides item 9 |
| 10.5 signing out from the frame | engine built; client not | 50–70 | none |
| 10.6 reads answered signed out | not built (watches and requests built) | 15–25 | none |
| 10.7 a reopened run's watch from its turn's beginning | built (one nuance) | 0 (≤20 for the nuance) | item 8 if the nuance is taken |
| 10.8 the people child's timers | built in 187eb86d for jig's root and the core world; conformance worlds and `ops`'s root never fire them | 0 domain; ~5 test | none |

### 10.1 A reply naming its own sign-in

- **Design.** `domain/people.md` 3: "the domain names a sign-in only by its
  number"; STATUS.md row `domain/people.md`: "3: a reply naming its own
  sign-in". `protocol/store.md` 9: "A sign-in is committed first, which gives
  it its number; its digest is then kept ... only then does the client's
  component hand its secret out" (the protocol layer needs the number on the
  sign-in's reply, and only there).
- **Today: not built.** `Core::signing_in: Option<u64>`
  (`crates/jig-core/src/lib.rs:267`, init `:537`) is set to every candidate
  sign-in at `crates/jig-core/src/routing/people.rs:80` and **never cleared**
  (no other assignment in the workspace). Every later people reply carries it:
  `routing/people.rs:131` (busy replies) and `:169` (every other reply), so a
  second party's replies name the first party's sign-in until the next sign-in,
  and a refused sign-in after a successful one names the old number.
  `Core::quiescent` (`lib.rs:615`, term `:636`) is false forever after the first
  sign-in (only test callers: `tests/core/tests/effects.rs:341,345`).
  `people::Reply::SignedIn { person, expires }`
  (`crates/jig-core-people/src/boundary.rs:504`) carries no number; made at
  `crates/jig-core-people/src/domain.rs:815` (replay of a live sign-in) and
  `:879` (new).
- **What changes.**
  - `people::Reply::SignedIn` gains `sign_in: u64` (boundary.rs:504; constructors
    domain.rs:815, :879). Sites with no `..` that must change: 7 (all test):
    `crates/jig-core-people/src/tests.rs:210`; `tests/core/tests/joint_clock.rs:32,65`;
    `tests/client/src/joint/wire/answers.rs:28,95`; `tests/client/src/joint/engine.rs:189`;
    `tests/client/src/joint/packets.rs:869`. ~30 other `Reply::SignedIn { .. }`
    patterns are untouched.
  - `jig_core::Held::PeopleReply { to, sign_in, reply }`
    (`crates/jig-core/src/routing/vocabulary.rs:140`) loses `sign_in`.
    Production constructors: 3 (`routing/people.rs:70-73`, `:131`, `:169`).
    Test sites binding or building `sign_in`: 20 in 17 files —
    `crates/jig-core/src/tests/restore_admission.rs:22-24`;
    `examples/ops/domain/src/tests.rs:338`;
    `tests/core/src/client_peer.rs:57,60`; `tests/core/tests/client_peer_routes.rs:56-59`;
    `tests/core/src/observations.rs:864-867`; `tests/core/src/world.rs:593-594`;
    `tests/core/src/effects.rs:672-673,778-779`; `tests/core/tests/client_peer.rs:31-32`;
    `tests/core/src/referee/offers.rs:112-113`; `tests/core/tests/restore_pages.rs:38`;
    `tests/conformance/src/lib.rs:511-513`; `tests/ops/src/conformance/mod.rs:519-523`;
    `tests/ops/src/joint/front.rs:533`; `tests/client/src/joint/requests.rs:245`;
    `tests/client/src/joint/front.rs:428-429`; `tests/client/src/joint/engine.rs:185,273`;
    `tests/client/src/joint/session.rs:627-632`. 21 further `Held::PeopleReply { .. }`
    matches (e.g. `examples/ops/domain/src/route.rs:331`,
    `testing/jig-test-domain/src/routes.rs:83`, `testing/jig-test-domain/src/client_documents.rs:69`,
    `tests/ops/src/joint/engine.rs:278`) are untouched.
  - Helpers threading the `Option<u64>`: `tests/core/src/peers.rs:307`
    (`Peers::reply`, callers `tests/core/src/effects.rs:673`,
    `tests/conformance/src/lib.rs:513`) and `:367` (`party_reply`, reads it only
    for `SignedIn` at `:369-370`); `tests/client/src/joint/packets.rs:292`
    (`people_reply`, callers `packets.rs:428,444,477,494,497,871,876,900`,
    `session.rs:632`).
  - `Core::signing_in` removed: `lib.rs:267,537,636`, `routing/people.rs:80`.
- **Worlds and fixtures.** Core step: `crates/jig-core/src/tests/restore_admission.rs`
  (`signin` :17, `env` :5, `identity` :9). Core world: two parties signing in —
  `tests/core/tests/stories.rs:18` (`ask_as`), the second-person story at
  `stories.rs:1149`, `tests/core/tests/client_peer_routes.rs` (two sign-ins,
  :116-164), `tests/core/src/peers.rs` (scripted parties). Conformance:
  `tests/conformance/src/lib.rs:511`. People world: `tests/people/src/world.rs:279`
  (`signin`), `:303` (`signout`).
- **Dependencies.** None. Shares `routing/vocabulary.rs` (`Held`) with items 1,
  4, 5, 6: a merge hotspot, not an order. `crates/jig-core-people/src/boundary.rs`
  is 781 lines: +3 keeps it under 800.

### 10.2 Escalations decided when no longer waiting: refused as not waiting

- **Design.** `domain/people.md` 5.1: "deciding the escalation needs it to wait
  for the party, and is refused as not waiting for them otherwise"; "Two
  parties deciding one thing: the first commit decides; the second is told it
  was decided, by whom, and how."
- **Today: partly.** The request path
  (`crates/jig-core/src/routing/party_decisions.rs:438`, `escalation_request`,
  dispatched from `routing/people.rs:237`): waiting at the named revision →
  `person_escalation_decide` (:313), whose standing check
  (`crates/jig-core/src/acts/objects/decisions.rs:150-152`) already answers
  `NotWaiting` for one waiting for someone else; another revision → moved
  (:470-475, built); otherwise `Now::HistoricalEscalation` (:477), answered by
  `crates/jig-core/src/routing/historical.rs:11` `route_historical_escalation`.
  Gaps:
  1. `historical.rs:67-68`: no decision record → `Refusal::Unknown`. A live task
     whose escalation is `Unheld`/`Rejected` at the named revision with no
     escalation decision recorded (the hold released by a `Release` act or an
     amendment, not by deciding the escalation) is answered unknown, not not
     waiting. Also `:53` and `:62-65` (record found but not the party's /
     unbounded) → `Unknown`, which is right for "may not see".
  2. `decisions.rs:144-149`: `Unheld | Routing | Rejected` → `Unknown`; this
     judge also gives the escalation document's offers
     (`crates/jig-core/src/acts/objects.rs:43`), so a document of an escalation
     no longer waiting offers "unknown" for Release/LeaveHeld/PassUp.
  3. `party_decisions.rs:339-348`: `pending == false` → `Unknown`; unreachable
     behind `current_person_escalation` (:396, checked at :451), so it becomes
     `NotWaiting` or an assertion.
  The tasks child's `InvalidResult::Form` arm the draft names (its debt 1) was
  not re-checked here.
- **What changes.** `acts::escalation` (decisions.rs:132; users:
  `party_decisions.rs:354`, `acts/objects.rs:43`); `route_historical_escalation`
  (historical.rs:11; one caller `routing/mod.rs:494`), which needs the live
  `core.tasks.escalation(task)` to tell "live, not waiting" from "unknown";
  `person_escalation_decide` (one caller). No vocabulary change: `NotWaiting`
  exists (`crates/jig-core-people/src/refusals.rs:29`) and is mapped by every
  world (`tests/core/src/peers.rs:595`, `tests/client/src/joint/wire/refusals.rs:26`)
  and the client (`crates/jig-client/src/contract/refusals.rs:159`,
  `crates/jig-client-view/src/words/refusals.rs:25`).
- **Worlds and fixtures.** Core step: `crates/jig-core/src/tests/acts.rs:611`
  (escalation offers), helpers `sign_in` :520, `ask_now` :538. Core world:
  `tests/core/tests/stories.rs:941` (`a_release_after_a_pass_up_...`: `World::new(151,false)`
  from `tests/core/src/effects.rs:161`, `ask` :14, `live_row` :900) is the
  template: stop, park, release by `Ask::Release`, then `DecideEscalation` at the
  old revision → `NotWaiting`; `stories.rs:450` for a delegate's escalation;
  the moved case is at `stories.rs:920`.
- **Dependencies.** None.

### 10.3 Chats forgotten once ended and read

- **Design.** `domain/people.md` 6: "A chat is forgotten once it has ended and
  its person has read its replies and its result; its record and its read
  position are erased in one decision". `protocol/store.md` 11.1: "ended chats
  are forgotten once read" (item 11's retention row names the same).
- **Today: partly.** The decision exists: `core::Event::ForgetChat { row }`
  (`crates/jig-core/src/routing/vocabulary.rs:453`, routed `routing/mod.rs:479`)
  → `crate::inbox::forget_chat` (`crates/jig-core/src/inbox.rs:490-517`) checks
  ended, `chat_seen ≥ last_reply`, `results_seen ≥ result_position`, then erases
  `tasks::Key::Ended` and sends `people::Event::ForgetChat`
  (`crates/jig-core-people/src/boundary.rs:626`; `inbox.rs:721` erases each
  `ReadChat` at `inbox.rs:554-557`). **Nothing in production sends it:** its one
  sender is `tests/core/tests/activity.rs:211`. And
  `crates/jig-core/src/routing/tasks.rs:686-697` half-forgets: a chat ending with
  `last_reply == 0` tells people `ForgetChat` at its end, while its ended row (and
  its unread result) stays, never erased.
- **What changes.** Triggers when the last condition becomes true:
  - the chat mark: `inbox::seen_chat_loaded` (`inbox.rs:356-387`) has the ended
    row; push `Event::ForgetChat { row }` after `ChatSeen` (:378) — and the live
    path at `routing/people.rs:260-270` for a chat not yet ended needs nothing;
  - the results mark: `routing/people.rs:256-259` (`Seen::Results`) must find the
    party's ended chats with `result_position ≤ through`; the core's
    `archived_chats` (`lib.rs:222`, filled `inbox.rs:120-160`) holds only those
    the cached updates still name, so the rest needs a load: a new
    `Now` (or a scope of `Now::SeenChat`, vocabulary.rs:240) answered by each
    root. Exhaustive `Now` matches that would change: `testing/jig-test-domain/src/routes.rs:513`,
    `examples/ops/domain/src/route.rs:1325`, `tests/core/src/client_peer.rs:152`,
    `tests/ops/src/joint/engine.rs:335`, `tests/ops/src/joint/front.rs:744`,
    `tests/client/src/joint/session.rs:789`, `crates/jig-core/src/tests/completion_retry.rs:45`;
    the loader in the core world is `tests/core/src/effects.rs:547`.
  - `routing/tasks.rs:686-697`: the half-forget goes (the chat is forgotten
    when its result is read).
  - Users of `people::Event::ForgetChat`: `crates/jig-core-people/src/domain.rs:422`,
    `crates/jig-core/src/routing/people.rs:28,44`, `routing/tasks.rs:691`,
    `routing/watch_marks.rs:313`, `tests/people/src/world.rs:346-347`,
    `tests/people/tests/memory.rs:257`, `tests/people/tests/simulation.rs:727`.
    Of `core::Event::ForgetChat`: `routing/mod.rs:351,479`,
    `tests/core/tests/activity.rs:211`.
- **Worlds and fixtures.** Core world: `tests/core/tests/activity.rs:178-219`
  (`all_replies_read_then_ended_survive_restart_and_forgetting`) becomes the
  story with its own `ForgetChat` replaced by the mark; fixtures
  `jig_core_world::reads::{ask, latest}` (`tests/core/src/reads.rs:375,402`),
  `stories::reads::request` (`tests/core/src/stories/reads.rs:43`),
  `stories::reads::conversation` (:53), `faults::plan_fixture`
  (`tests/core/src/faults.rs:86`), `World::restart_with`
  (`tests/core/src/effects.rs:1161`). People world: `tests/people/src/world.rs:346`
  (`forget_chat`), `:323` (`remember_result`), `:337`/`:342` (positions).
- **Dependencies.** The results-mark half needs either a load of the party's
  ended chats or item 11's "each party's conversations" index (store.md 7.3,
  15 item 3); item 11's retention row "chats forgotten" is this same work: plan
  one increment, not two. `crates/jig-core/src/routing/tasks.rs` is 1,414 lines
  (over 800).

### 10.4 A task's proposal nobody may accept, refused to its proposer

- **Design.** `domain/README.md` 5: "A proposal nobody may accept is refused,
  whoever makes it: a party's at admission, a run's `propose` in its answer, a
  procedure's step in its decision". `domain/tasks.md` 9 (:603-613): "it is
  refused to its proposer as beyond authority, saying that no proposal could be
  accepted, and nothing of it is committed ... A proposal already pending waits".
  `domain/engine.md` 7.3 (:512): `propose` answers "refused, when nobody may
  accept it".
- **Today: not built for tasks; built for parties.** The party path checks
  `recipient` (`crates/jig-core/src/acts/bounds.rs:11-20`, private) and refuses
  `unproposable()` (`bounds.rs:384-386`:
  `Authority { lacking: Approver, proposable: Some(false) }`) at `bounds.rs:278`
  and `:372`. The task path's `holder` (`crates/jig-core/src/proposals.rs:124-170`,
  via `Core::proposal_holder` :311) returns `ProposalHolder::Policy` (:150,
  :162, :165) with no check that any role decides the kind, and the proposal is
  committed waiting for nobody at: `crates/jig-core/src/routing/runs.rs:270`
  (`named_propose` :246, reached from the `propose` call `runs.rs:558`, a batch
  proposal `routing/run_delegation.rs:233`, an effect's proposal
  `crates/jig-core/src/effects.rs:961`), and `effects.rs:1200`
  (`procedure_propose` :1167). Re-routing of a pending proposal
  (`routing/tasks.rs:318,337`, `proposals.rs:243`, `runs.rs:215`) must keep
  waiting.
- **What changes.** `recipient` made `pub(crate)` (callers `bounds.rs:91,176,278,372`);
  a check after `holder` at `runs.rs:270` and `effects.rs:1200`; a refusal
  value for a run: `tasks::Refusal` (`crates/jig-core-tasks/src/boundary.rs:1061`)
  has none (`AuthorityShape`, `Reference` exist); a new variant needs its one
  exhaustive map, `crates/jig-core/src/routing/refusals.rs:25-70`, to the party's
  `Authority { lacking: Approver, proposable: Some(false) }`. The run hears it as
  `CallPart::ProposalRefused(tasks::Problem)` (`runs.rs:48-60`); `ops`'s root
  renders every refusal as `"refused"` with no reason
  (`examples/ops/domain/src/assemble.rs:296-318`). The procedure's refusal in
  its decision: today `drop_effect(.., Answer::Refuse)` + `procedure_wait`
  (`effects.rs:1201-1203`) on a missing holder; the same for "nobody".
- **Worlds and fixtures.** Core step: `crates/jig-core/src/tests/acts.rs:24`
  (`policy(core, accept, period)`: `accept == false` sets `decides =
  Proposals(16)`), the party-path twin test at `acts.rs:87`. Core world:
  `tests/core/src/effects.rs:112` (`fixture`, edits `policy.roles[0]` at
  :120-135), `World::configured` :175, `World::propose` :910 (effect proposal),
  `stories.rs:207` (a policy narrowed while a proposal waits: must stay
  pending), `stories.rs:81` (`plan_world`) for a batch proposal.
- **Dependencies.** None for the refusal; the agent reading why ("no proposal
  could be accepted") rides item 9 (answers encoded outside the root).
  `routing/runs.rs` is 784 lines (+5 fits); `effects.rs` 1,441 (over 800).

### 10.5 Signing out from the frame

- **Design.** `client/domain/client.md` 12: "Signing out, from the frame, is
  the shell's exchange too: it ends the sign-in on the engine (people.md,
  section 3), and the client then shows the sign-in page, as when signed out."
  "A service signs in at start ... and never sees a sign-in page."
  `protocol/client.md` 16: signing out as the shell's exchange is the client
  domain's.
- **Today: engine built, client not.** Engine: `people::Event::SignOut`
  (`crates/jig-core-people/src/boundary.rs:661`, decided `domain.rs:437`,
  `end_signin` :893-899 erases `Key::SignIn` and drops the alarm); the joint
  world already drives it (`tests/client/src/joint/session.rs:214-219`,
  `Session::sign_out`). Client: `jig_client::action::Act`
  (`crates/jig-client/src/action.rs:12`) has `SignIn { provider }` (:78) only;
  `jig_client::Request` (`crates/jig-client/src/boundary.rs:61`) has
  `SignIn { provider, back }` (:79) only; the frame
  (`crates/jig-client-view/src/frame.rs:22-48`) offers no sign-out control. The
  client's signed-out handling exists (`crates/jig-client/src/signin.rs:47-60`,
  `signed_out`; `Event::SignedOut` `boundary.rs:54`, `domain.rs:625`).
- **What changes (production).** `Act::SignOut` (action.rs; exhaustive users:
  `crates/jig-client/src/domain.rs:644-695` dispatch,
  `crates/jig-client-view/src/binding.rs:167-206`); `Request::SignOut`
  (boundary.rs; exhaustive users: the two roots,
  `testing/jig-test-client-domain/src/route.rs:107-140` (match from :107),
  `examples/ops/client-domain/src/route.rs:153-185`); offered only when
  `signin.service == false` (`signin.rs:19`); a frame control
  (`frame.rs`) with a new `JigPhrase` (`crates/jig-client-view/src/phrases/jig.rs`:
  enum ~:132, number ~:1189, class ~:1920, list ~:2470 — 4 places in a 2,994-line
  file; `phrases/english.rs` one arm near :752, 1,495 lines); each root's own
  `Request::SignOut` (`testing/jig-test-client-domain/src/boundary.rs:61`,
  `examples/ops/client-domain/src/boundary.rs:72`).
- **Users of the roots' `Request` that must gain an arm (test):** 12 sites in 9
  files — `tests/client/src/test.rs:170,203`; `tests/client/src/scripted.rs:252,354`;
  `tests/client/src/joint/requests.rs:123`; `tests/client/src/joint/session.rs:425`;
  `tests/client/src/joint/front.rs:182`; `tests/ops/src/client/app.rs:196,249`;
  `tests/ops/src/client/transport.rs:350`; `tests/ops/src/joint/front.rs:268`;
  `examples/ops/client-view/src/tests.rs:347`. The harness's `Duty`
  (`tests/client/src/harness/mod.rs:58-63`) and `Front` trait (:66-76, `sign_in`
  :75) gain a sign-out; `Front` impls: 6 (`tests/client/src/scripted.rs:260`,
  `tests/client/src/joint/front.rs:158`, `tests/client/src/harness/navigation_tests.rs:18`,
  `tests/client/tests/referee.rs:170`, `tests/ops/src/client/transport.rs:184`,
  `tests/ops/src/joint/front.rs:829`); duty handling `tests/client/src/harness/world.rs:521-580`.
- **Worlds and fixtures.** Client world: `harness/world.rs:358` (`sign_as`),
  `tests/client/tests/faults.rs:126` (signed-out parks and resumes) as the
  template; the scripted engine's `Change::SignedOut`
  (`testing/jig-scripted-engine/src/truth.rs:38,172`) and `Fault::SignOut`
  (`faults.rs:26,107,148`) end the sign-in; the fake person finds the control by
  `Target::named(role, name)` (`testing/jig-fake-person/src/lib.rs:40`). Joint
  world: `Session::sign_out` (`session.rs:214`) then a request answered signed
  out. View step tests: `crates/jig-client-view/src/tests/`.
- **Dependencies.** None in the domain. The real exchange (cookie cleared,
  digest erased) is the client's protocol (`protocol/client.md` 8), outside
  items 1–12.

### 10.6 Reads answered signed out

- **Design.** `client/domain/contract.md` 6: "Refused for what the party may not
  see, as unknown; when the engine's loads are full, as busy; when the sign-in
  has ended, as signed out." `protocol/client.md` 16 lists it as owed.
- **Today: not built for reads.** `reads::Why`
  (`crates/jig-core/src/reads/vocabulary.rs:304-311`) has `Unknown` ("Missing,
  signed out, or invisible; these are indistinguishable"), `Busy`, `Gone`.
  `reads::start` (`crates/jig-core/src/reads.rs:182-185`) answers a sign-in the
  people child does not know, ended or expired with `refused_application`
  (:150-179) → `Why::Unknown`; `reads::loaded` (:386-410) answers `Unknown` when
  the sign-in ended while loading (:395-405). `people.person`
  (`crates/jig-core-people/src/domain.rs:369-378`) returns `None` both when not
  ready and when signed out, so the split needs `people.ready()`. Built
  elsewhere: watches refuse `WatchRefusal::SignedOut`
  (`crates/jig-core/src/routing/views.rs:196`, `vocabulary.rs:171-181`), live
  watches end `views::End::SignedOut` (`routing/watch_marks.rs:386`), requests
  answer `people::Reply::SignedOut`. The client already handles
  `ReadNone::SignedOut` (`crates/jig-client/src/contract/reads.rs:222`;
  `application.rs:149`, `objects.rs:782`, `pages/routing.rs:482`).
- **What changes.** `Why::SignedOut`; `refused_application` takes the `Why`;
  `start` and `loaded`. Exhaustive users of `Why`: 1
  (`tests/client/src/joint/read_packets.rs:38-42`); ~30 equality asserts in
  `crates/jig-core/src/tests*.rs` and `tests/core/tests/*.rs` stay unless a
  case is now signed out (`crates/jig-core/src/tests.rs:2445-2632`,
  `tests/core/tests/reads.rs:144` to re-check).
- **Worlds and fixtures.** Core step: `crates/jig-core/src/tests/read_filters.rs`,
  `crates/jig-core/src/tests.rs` (`core()` :301, `config` :193). Core world:
  `tests/core/tests/reads.rs`, `reads::ask`/`latest`
  (`tests/core/src/reads.rs:375,402`), sign-in expiry via
  `tests/core/tests/joint_clock.rs:73` (`sign_in` helper :7). Joint world:
  `read_packets.rs:38` maps it to `client::ReadNone::SignedOut`.
- **Dependencies.** None.

### 10.7 A reopened run's watch from its turn's beginning

- **Design.** `client/domain/client.md` 10: "A watch opened again starts from
  the beginning of the turn being written: the client keeps the text it holds
  of that turn, and takes what goes past it." `protocol/client.md` 16: "is in
  its design".
- **Today: built, with one nuance.** Engine: `core` opens a run watch as
  `views::Follow { run, turn: None, offset: 0 }`
  (`crates/jig-core/src/routing/views.rs:144`), and the views child delivers from
  the retained start, marked cut past its bound
  (`crates/jig-core-views/src/text.rs:283-294`). Client: a reopen re-sends the
  watch's original subject (`crates/jig-client/src/domain.rs:1186-1196`; the
  subject is `Run { task, turn: None, offset: 0 }`, `pages/task.rs:520`);
  `Request::Continue` (`boundary.rs:69`, sent `conversation/routing.rs:199-204`)
  is the in-watch stream acknowledgement, honoured by the scripted engine
  (`testing/jig-scripted-engine/src/watches.rs:83-95,309-318`) and mapped to
  `StreamDelivered` by the joint world (`tests/client/src/joint/transport.rs:312-331`).
  Nuance: after `interrupted()` (`crates/jig-client/src/conversation.rs:417-420`)
  the first delivery **replaces** the turn's held text (`reopened`, :386-402,
  called :320-322; pinned by `crates/jig-client/src/tests/conversations.rs:195-215,1060-1102`),
  so a reopened delivery cut by the engine's bound drops text the client held
  before the cut, where the design keeps it.
- **What changes, if taken.** `Conversation::stream`/`reopened` only; tests
  `conversations.rs:205,1091,1108,1116`.
- **Dependencies.** Item 8 (text by retry, a gap marked cut) changes the views
  child's text (`text.rs:114`) and what "cut" means on a delivery; take the
  nuance with or after it.

### 10.8 The people child's timers

- **Design.** `domain/people.md` 3: sign-ins "expire after a configured time
  ... the core says each that ends or expires, so the protocol layer erases its
  digest"; `protocol/store.md` 11.1: "Answered keys expire past the people
  child's retention".
- **Today: built for jig's root and the core world** (commit 187eb86d, after
  the draft's survey): `Timer::People` (`crates/jig-core/src/routing/vocabulary.rs:637-638`),
  fired at `routing/mod.rs:518-525` into `people::fire`
  (`crates/jig-core-people/src/domain.rs:467-478`: one answered key or one
  sign-in per fire); `jig-test-domain` reports it
  (`testing/jig-test-domain/src/lib.rs:332-341`, `core_deadlines`); the core
  world fires due timers (`tests/core/src/effects.rs:366-409`, `next_due`,
  `fire_due`); step tests `crates/jig-core/src/tests/engine_timers.rs:72,114`;
  world tests `tests/core/tests/joint_clock.rs:73`; the people world fires
  `people::fire` (`tests/people/src/world.rs:941`). "The core says" a sign-in
  ended: as `Write::Erase(Key::People(people::Key::SignIn(n)))` in the decision
  (`routing/people.rs:115-126`) plus a droppable `Fact::SignedOut`
  (`crates/jig-core-people/src/facts.rs:17`); no held output names it, so the
  secrets' component must read the erase from the commit (a protocol-layer
  choice, no domain change).
- **Gaps.** The conformance worlds never fire it: `tests/conformance/src/lib.rs:390-397`
  (`timers`: Fleet, Tasks, connectors) and `tests/ops/src/conformance/mod.rs:225-228`
  (Tasks, Fleet, Account, Brief). `ops`'s root reports no core deadlines
  (`examples/ops/domain/src/route.rs:176-180` fires any `core::Timer` it is
  given; nothing like `core_deadlines`). Neither the core nor jig's root exposes
  one earliest deadline (`core_deadlines` is per child).
- **Dependencies.** None; item 1 (stopping) and item 11 (retention on the
  domain's timers) share the timer plumbing.

## Item 11. The store's domain debts

Design: `protocol/store.md` 15 ("The domain owes" 1–7), with 4.3, 4.4, 7.2–7.4, 8.3, 8.5, 11.1, 11.2; `domain/engine.md` 5.4–5.6. All line numbers at `main` 2fae240a. Counts are `grep -rn --include=*.rs` over `crates testing tests examples`.

### 11.0 The store boundary as built (what jig-fake-store stands in for)

- **Core vocabulary** (`crates/jig-core/src/routing/vocabulary.rs`): writes are `Request::Write(Write::Save(Record) | Write::Erase(Key))` (:20, :102–108); no "drop a task's transcript" write. Loads: `Held::ReadLoad { load, range: reads::Range }` (:117) for parties' reads, `Held::NotesLoad` (:162), and five single loads the root serves by key: `Now::StartPreparation { task, transcript_waiter }` (:238, transcript), `Now::SeenChat` (:240), `Now::HistoricalProposal` (:242), `Now::HistoricalPersonTask` (:244), `Now::HistoricalEscalation` (:266). Answers come back as `Event::ReadLoaded { load, result: reads::Loaded }` (:335), `SeenChatLoaded`, `HistoricalProposal{row}`, `ForgetChat { row }` (:453), and the root's own transcript page, fed via `Core::append_transcript` (`crates/jig-core/src/run.rs:163`).
- **The core never hears a commit** (engine.md 4.1): the root's journal does. Ops: `Store::Committed { number }` (`examples/ops/domain/src/boundary.rs:18`) → `domain.journal.committed(number)` (`examples/ops/domain/src/domain.rs:203`); reference root: `Event::Committed { number }` (`testing/jig-test-domain/src/lib.rs:268`, handled :562). skein's `Journal::committed(number)` takes the number only.
- **Keys and records:** `core::Key { Core(CoreKey), Tasks(tasks::Key), People(people::Key), Notes(notes::Key) }` (`crates/jig-core/src/stored.rs:218–227`), `core::Record` (:205–214), `CoreKey` (:183–200), `CoreRecord` (:162–181). Roots wrap them once more: ops `Key/Record/Write/Range` (`examples/ops/domain/src/store.rs:8–60`, `core_key` :61–85), reference root `Key/Record` (`testing/jig-test-domain/src/lib.rs:25–70`). Routing of core writes: `testing/jig-test-domain/src/routes.rs:26–29`, `examples/ops/domain/src/route.rs:214–217`.
- **Restart:** `RestartStep::{LoadCore, LoadInbox, RestoreConnector, AdoptRuns, ReadAfresh, SettleOutbox, Open}` (`crates/jig-core/src/restart.rs:8–23`). The core defines **no live range** except the inbox census predicate `inbox::restore_key` (`crates/jig-core/src/inbox.rs:397–414`, re-exported `lib.rs:30`). Rows are routed by the root to `Core::restore_core` (`stored.rs:329`, archives answered `Restored::Archive` :378–381), `Core::restore_task_row` (`stored.rs:274`, refuses Ended/History/Activity/Milestone :318–321), `tasks::Event::Restore` (`crates/jig-core-tasks/src/stored.rs:703`, refuses history rows :766–769), people, notes (no-op, `crates/jig-core-notes/src/domain.rs:136–139`), `Core::restore_inbox_row` (`inbox.rs:418`). Root sites: `examples/ops/domain/src/restart.rs:24–25` (symbolic `Range::Core`/`Range::Inbox`), :104–170; `testing/jig-test-domain/src/lib.rs:1005–1050`.
- **Who decides what is "live" today: the worlds, by variant.** Core world `tests/core/src/effects.rs:1028–1048` loads *every* `root::Key::Core` key (`LoadCore`) as one `first..=last` range and re-sorts rows into deployment / children / proofs / calls, skipping archives (`:1066–1120`); conformance world the same (`tests/conformance/src/lib.rs:146–215`); ops world filters Ended/Activity/History/Milestone by variant and sorts Deployment, Person, people, tasks, core (`tests/ops/src/conformance/mod.rs:340–393`); tasks world restores by variant (`tests/tasks/src/world.rs:976–1006`). With intervals over keys, none of this survives: the key order must carry it.
- **jig-fake-store** (`testing/jig-fake-store/src/lib.rs`, 235 lines): generic `Store<K: Ord, R>` on a `BTreeMap`; `submit`/`tick` (ordered whole commits, :110–160), `restore(first,last)` (:175), `page(first,last,after,most)` (:180), `load`/`tick_pages` (:207–230). No fill, no bytes, no transcripts. Users: 35 files (`testing/jig-conformance/src/harness.rs`, `tests/core/**` 25 files, `tests/conformance/**` 4, `tests/ops/**` 6).
- **Parties' reads transport** is a world fake that scans every row per scope: `tests/core/src/reads.rs:228–329` (`candidate`), `:331–373` (`page`); ops wraps it and adds a History scan for task proposals (`tests/ops/src/joint/engine.rs:74–130`). The core states each scope's admissible keys in `reads::rows::position_allowed` (`crates/jig-core/src/reads/rows.rs:1427–1497`) and `key_kind` (:1512–1546); a real store must map each to intervals (store.md 4.3).
- **Cursors carry keys to the client:** `tests/client/src/joint/wire/cursor.rs` (1,036 lines) encodes `CoreKey` by tag (:597–645) and `tasks::Key` by tag (:793–850); every new key variant or reorder changes this codec.

### 11.1 Live before history in each child's keys; decided proposals and past periods apart

Design: store.md 15.1 "live before history in each child's keys: the tasks child's history and ended tasks after its live families (stubs among these), decided proposals apart from pending ones, and past funding periods apart from live ones (4.3)"; 4.3 "within each child, the families a restart loads come first, and history after them … so each child's live part is one interval"; engine.md 5.5 "what is live … is kept in key ranges of its own, so a restart reads it without reading history".

**Each child's families today, in derived `Ord` order:**

| Child | Families (order) | Live (at start) | Verdict |
|---|---|---|---|
| core `CoreKey` (`stored.rs:183–200`) | Projection, Deployment, RunProof, Call, Turn, Terminal, ProposalDecision, EscalationDecision | first four | **built** (live first). Missing: the per-task conversation family (first attempt, current window start; store.md 4.4 "conversations \| task"): `RunProof.transcript_from` (`stored.rs:523`) holds the first attempt only (items 6, 7) |
| tasks `Key` (`crates/jig-core-tasks/src/boundary.rs:978–999`) | Activity :980, PersonProposal :982, History :984, Milestone :986, Live :988, Ended :990, Stub :992, Ledger :994, Writer :996, Pool :998 | Live, Stub, Ledger (open), Writer, Pool, pending PersonProposal | **not built**: live families are split by History/Milestone/Ended; Activity sorts first |
| people `Key` (`crates/jig-core-people/src/boundary.rs:532–547`) | Person, ReadResults, ReadChat, SignIn, Roles, Policy, Answer | all (expired answers erased at restore, `crates/jig-core-people/src/domain.rs:1608–1610`) | **built** per store.md 4.4. (engine.md 5.4 says parties without a live sign-in or role load "as they sign in"; the people child restores every `Person`, `domain.rs:1584–1589`: a doc disagreement, not owed by store.md 15) |
| notes `Key` (`crates/jig-core-notes/src/boundary.rs:80–85`) | Entry{name}, Line{scope,name} | none (Restore is a no-op) | **built**; but the worlds' `LoadCore` reads them (all Core keys) |
| fleet, accounts, authority, brief, views | no `Key`/`Stored` (checked: none of their boundaries saves) | — | as designed |
| core `Key` (`stored.rs:218–227`) | Core, Tasks, People, Notes | — | matches 4.2 (core's own, then children) |

**Decided proposals apart:** task proposals already are (pending in `TaskRecord.proposal`, `boundary.rs:875`; decided into the proposer's `History` row, `crates/jig-core-tasks/src/control.rs:93–102`, and `CoreRecord::ProposalDecision`). **Person proposals are not:** one family `Key::PersonProposal(u64)` holds both; the decision overwrites the pending row at the same key (`crates/jig-core-tasks/src/proposals.rs:256` pending, :317–329 decided), and restore skips decided rows after reading them (`crates/jig-core-tasks/src/stored.rs:710–725`); `inbox::restore_key` reads every one (`inbox.rs:400`).

**Past funding periods apart: not built, and a latent capacity bug.** Closed ledgers stay in the live table forever: `retire` only sets `closed = true` and re-saves (`crates/jig-core-tasks/src/funders.rs:631–686`), `restore_funding` admits closed rows back into `domain.funding` (:594–619), and the only `funding.remove` is a rollback (`recurring.rs:131`). The table is bounded by `Limits::funders` (`crates/jig-core-tasks/src/limits.rs:16`); `open` refuses a new period `Busy`/`CapacityCause::Funders` once full (`funders.rs:418–432`). So every period's Period/Pool/Recurring rows accumulate until periods stop opening. Moving them out needs a replacement for the monotonic check that scans old periods (`funders.rs:407–416`, `newer_period` :621–629): only the current period need stay live.

**Types and functions that change, with users:**
- `tasks::Key` reorder (live: Live, Stub, Ledger, Writer, Pool, PendingProposal; history: Ended, PastLedger, DecidedProposal, History, Milestone, Activity, then indexes 11.3). Renaming by variant breaks no user; order-dependent users: exhaustive matches `crates/jig-core-tasks/src/boundary.rs:1040–1055` (`Stored::key`), `crates/jig-core/src/inbox.rs:397–414`, `crates/jig-core/src/reads/rows.rs:1512–1528`, `tests/tasks/src/world.rs:834–846`, `tests/client/src/joint/wire/cursor.rs:793–850` (tags). `tasks::Key::` is named 118 times in 27 files (rows.rs 15, inbox.rs 9, tests/core/tests/stories.rs 29, cursor.rs 20, …).
- New `Stored` variants (past ledger, decided person proposal): every exhaustive `Stored` match, by `Stored::Pool` as the marker: 20 files — tasks `boundary.rs`, `holds.rs`, `lib.rs`, `owned.rs`, `stored.rs`; core `inbox.rs`, `proposals.rs`, `reads/rows.rs` (7), `reads/vocabulary.rs` (2), `routing/tasks.rs` (5), `stored.rs`; tests `tests/client/src/joint/truth.rs`, `tests/core/src/observations.rs`, `tests/core/tests/effects.rs`, `tests/tasks/src/accounting_referee.rs` (3), `tests/tasks/src/world.rs` (6), `tests/tasks/tests/{activity,funding,person,restart}.rs`.
- `Stored::Ledger`: 72 uses in 32 files (most: `tests/tasks/src/accounting_referee.rs` 11, `crates/jig-core/src/reads/rows.rs` 7, `tests/tasks/tests/recurring.rs` 6, `tests/tasks/src/world.rs` 6, `tests/tasks/tests/restart.rs` 5, `crates/jig-core/src/routing/tasks.rs` 5). `Key::Ledger`: 23 in 11 files. `FundingRecord.closed` readers outside funders: `crates/jig-core/src/documents/objects.rs:432`, `documents/project_note.rs:284,345`, `crates/jig-core-tasks/src/{moving.rs:203,226, standing.rs:56,62, periods.rs:31}`. `Scope::Spend` reads `Ledger(Period{project,..})` (`rows.rs:1491`, `key_kind` :1521): must read the past family.
- `Stored::PersonProposal`: 44 in 22 files; `PersonProposalState::`: 75 in 18 files; `Key::PersonProposal`: 9 in 7 files (`boundary.rs:1044`, `inbox.rs:400`, `rows.rs` 2, `tests/core/tests/stories/procedure_goals.rs`, `tests/core/tests/stories.rs`, `tests/tasks/src/world.rs`, `cursor.rs` 2).
- New: a live-range definition the core owns (a predicate or fixed interval list per child, in restore order: deployment, people, tasks' live, core's live; store.md 7.2), replacing the worlds' variant filters at `tests/core/src/effects.rs:1028–1048,1066–1120`, `tests/conformance/src/lib.rs:146–215`, `tests/ops/src/conformance/mod.rs:340–393`, `tests/tasks/src/world.rs:976–1006`, and the ops special case `examples/ops/domain/src/restart.rs:121`.

**Worlds and fixtures:** tasks world `tests/tasks/src/world.rs` (`restart` :976, `records` map), `tests/tasks/tests/{restart,funding,recurring,proposals}.rs`; core world `tests/core/src/effects.rs` (`World::new` :161, `load_restart` :1028, `restored` :1066, `restart` :1150, `restart_with` :1161), `tests/core/src/faults.rs:74` (`restart`), `tests/core/tests/{restart,store,restore_pages}.rs` (`restore_pages.rs` helpers `world` :45, `pause` :61); conformance `tests/conformance/src/lib.rs` (`Neighbours::restored` :146); ops `tests/ops/src/conformance/mod.rs:340–393`. A unit property test per child ("every live key sorts before every history key") belongs beside each `Key` (`crates/jig-core-tasks/src/tests/`). No fixtures module exists in these worlds; helpers live in `world.rs`/`effects.rs`.

**Estimate:** 180–260 production lines (key reorder ~15; pending/decided person proposals ~40 in tasks `proposals.rs`/`stored.rs` + ~20 in core reads/inbox; past periods ~70 in `funders.rs` (784 lines today, will pass 800: split `retire`/`restore_funding` out) + ~20 in `project_note.rs`/`rows.rs`; live-range definition ~40). Plus the wire cursor codec (~20, test kit).

### 11.2 The inbox's restore reads every ended task

Design: store.md 15.2 "the inbox's restore reads what the inbox shows from an index, not every ended task (7.2)"; 7.2 "the inbox's census … the index of what parties' inboxes show (7.4), never every ended task".

**Today: not built.** `inbox::restore_key` admits `Live | Ended | PersonProposal` (`crates/jig-core/src/inbox.rs:397–414`); `Core::restore_inbox_row` (`inbox.rs:418–478`) rebuilds from each Ended row: unread results via `people.remember_result` (:438–450, `crates/jig-core-people/src/inbox.rs:235`), chat totals via `people::Event::ChatState` (:452–466), and `cache_archived_chat` (:468, def :120–158). The worlds load it as one `first..=last` key range (`tests/core/src/effects.rs:1035`, `tests/conformance/src/lib.rs:202`), which by today's key order also sweeps every History and Milestone row (they sort between PersonProposal and Live; `restore_inbox_row` ignores them, :472–477).

**Gap:** an index family of what parties' inboxes show (per party, by time: unread results, ended chats not forgotten with unread replies), saved in the decision that ends a task (`crates/jig-core-tasks/src/closing.rs:334–335` erase Live/save Ended; the core assigns `result_position` at `crates/jig-core/src/routing/tasks.rs:705–708`), erased when the chat is forgotten (`inbox.rs:493–516`) and when read positions pass it; `restore_key` and `restore_inbox_row` read index rows instead of Ended rows.

**Changes and users:** `inbox::restore_key` (4 users: `lib.rs:30`, `tests/conformance/src/lib.rs:202`, `tests/core/src/effects.rs:1035`, `tests/ops/src/conformance/mod.rs:358`); `Core::restore_inbox_row` (2: `testing/jig-test-domain/src/lib.rs:1009`, `examples/ops/domain/src/restart.rs:108`); `RestartStep::LoadInbox` 23 uses in 11 files (core `resources.rs`, `inbox.rs`, `restart.rs` 2, `tests/restore_admission.rs` 2; `testing/jig-test-domain/src/lib.rs` 3; ops `restart.rs` 2, `tests.rs` 1; `tests/core/src/effects.rs` 3, `tests/core/tests/restart.rs`, `tests/ops/src/conformance/mod.rs` 2, `tests/conformance/src/lib.rs` 5); ops `Range::Inbox` (`examples/ops/domain/src/store.rs:12`, `restart.rs:25`, `tests/ops/src/conformance/mod.rs:357`).

**Worlds:** core world restart and reads: `tests/core/tests/restart.rs`, `tests/core/tests/reads.rs` (`an_inbox_longer_than_the_engine_holds_pages_from_the_store` :82, `conversation_owner_replay_keeps_every_retained_page_after_durability_and_restart` :161), `tests/core/src/stories/reads.rs` (`a_backward_wall_correction_keeps_reply_paging_whole_after_restart` :73, `result_and_reply_filters_page_their_own_retained_updates` :192); unit `crates/jig-core/src/tests/restore_admission.rs`; people world `tests/people/src/world.rs` (`remember_result` :323, `restart` :916).

**Estimate:** 120–200 lines (index rows and their saves/erases ~80, census rewrite ~60). **Depends on 11.1** (key order) and **is a case of 11.3** (the party's inbox index); land with or after it.

### 11.3 Indexes for parties' reads

Design: store.md 15.3 "each party's inbox by time, each party's conversations, each project's goals, each task's ended delegates, a subtree's delegates for its history (7.3, 7.4)"; 7.4 "a family of small rows its owner saves and erases in the same decision as the records they point at, with keys reversed in time where a page reads newest first".

**Today: no index family exists** (none in `tasks::Key`, `CoreKey`, `people::Key`). Every `reads::Scope` (`crates/jig-core/src/reads/vocabulary.rs:61–98`), by what `position_allowed` admits (`rows.rs:1427–1497`) and how the world transport serves it (`tests/core/src/reads.rs:228–329`):

| Scope | Keys admitted today | Index or key | Verdict |
|---|---|---|---|
| Task / PersonTask / Result {task} | `Ended(task)` | by key | fine |
| Proposal {Party} | `PersonProposal(n)` | by key | fine (moves with 11.1) |
| Proposal {Task(t), n} | any `History{t,*}` | none: scanned (ops `tests/ops/src/joint/engine.rs:74–130`; core world returns `None`, `reads.rs:310`) | **11.7** |
| Escalation {task} | any `EscalationDecision{task,*}` | revision not named; hold milestone unkeyed | **11.7**; core world serves none (`reads.rs:311`) |
| Question {task, message} | any `Activity{task,*}` | scan for the line and its answer | **11.7**; core world serves none (`reads.rs:312`) |
| Reply {task, activity} | `Activity{task,activity}` | by key (inbox entry's `activity`, `documents/inbox_deployment.rs:31–32`) | fine |
| Turn | `Turn{..}` | by key (moves to transcripts, 11.4) | fine |
| EndedDelegates {task} | every `Ended(*)` | **no index** (ended delegates by requester) | not built; core world serves none (`reads.rs:313`) |
| History {task, subtree:false} | `History{task,*}`, `Milestone{task,*}` | two family intervals | fine |
| History {subtree:true} | **every** `History`/`Milestone` (`rows.rs:1459–1471`) | **no index** (a subtree's delegates) | not built; not served in core world (`reads.rs:314`) |
| Activity {task} | `Activity{task,*}`, backwards from `TaskRecord.activity` | family | fine |
| Inbox Updates/Results/Replies | every `Ended(*)` and `Activity(*)` (`rows.rs:1404–1425`) | **no index** (party's inbox by time) | not built; world scans all (`reads.rs:249–296`) |
| Inbox Needs/Proposals/Escalations/Questions/PersonTasks | `Live(task)`, `PersonProposal(n)` | **no index** | not built (live doc answers the bounded prefix); not served in core world (`reads.rs:315–322`) |
| Conversations | every `Ended(*)` | **no index** (party's conversations) | not built; world scans all ended (`reads.rs:223–226,307`) |
| Goals {project} | every `Ended(*)` | **no index** (project's goals) | not built; not served in core world (`reads.rs:323`) |
| Notes {project} / ScopedNotes | every notes `Entry` | the notes child's `Line{scope,name}` family exists (`crates/jig-core-notes/src/boundary.rs:84`) but `key_kind` maps lines to `Unsupported` (`rows.rs:1540–1543`) | **partly**: index exists, reads ignore it (store.md 7.3: "the notes' lines … each with its entry, by name") |
| Spend {project} | `Ledger(Period{project,*})` | family interval | fine (past periods: 11.1) |

**Sites that must save/erase index rows in the same decision:** task ending `crates/jig-core-tasks/src/closing.rs:334–335` (ended delegates by requester, project goals, party conversations, party inbox result); core ending `crates/jig-core/src/routing/tasks.rs:686–720` (result position, `ForgetChat` for reply-less chats :691); reply turns `crates/jig-core-tasks/src/admission.rs:85–110` (inbox replies); forget `crates/jig-core/src/inbox.rs:493–516`; requester moves `crates/jig-core-tasks/src/moving.rs` (a goal moved to a person changes its conversations/goals owner); person proposals `proposals.rs:256,329`.

**Types and functions that change:** `tasks::Key`/`Stored` (or people's) gain index variants (users as 11.1); `rows::position_allowed`/`key_kind`/`inbox_key_allowed` (`rows.rs:1404–1546`), `rows::row_key` (:340–367), `reads::Row` join contract (`vocabulary.rs:185–200`: an index row must arrive with the record it points at as `value`/`owner`); `reads::Position` (`vocabulary.rs:101–107`) if index keys become positions; cursor codec `tests/client/src/joint/wire/cursor.rs`. Scope users: `Scope::Inbox` 24/10 files, `Scope::Conversations` 50/10 (`crates/jig-core/src/tests/conversation_cursor.rs` 13, `reads/rows.rs` 9, `reads/cursors.rs` 7, `tests/client/src/joint/wire/cursor.rs` 7, `pages.rs` 5), `Scope::Goals` 20/7, `Scope::EndedDelegates` 16/5, `Scope::History` 17/7, `Scope::Notes` 14/5, `Scope::ScopedNotes` 21/8.

**Worlds and fixtures:** both read transports must serve from indexes: `tests/core/src/reads.rs` (`candidate` :228, `page` :331, `ask` :375, `complete` :406, `ReadParties::load` :126), `tests/ops/src/joint/engine.rs:74`; stories `tests/core/tests/reads.rs`, `tests/core/tests/fuzzy_reads.rs`, `tests/core/src/stories/reads.rs` (`world` :13, `read_world` :17, `request` :43, `conversation` :53), `tests/core/tests/read_filters.rs`, `crates/jig-core/src/tests/{conversation_cursor,read_filters,inbox_selection}.rs`. Each index's own test belongs in its owner's world (tasks: `tests/tasks/src/world.rs`).

**Estimate:** 400–700 production lines, the largest debt: 5 index families × (key, row, save/erase at 2–4 sites) ~60–100 each, plus `rows.rs` mapping ~80 (rows.rs is 1,744 lines already: the index mapping must go in a new file). Splits naturally per index. **Depends on 11.1** (key order first).

### 11.4 Turns as the store keeps them

Design: store.md 15.4 "saved with whether each opens a window, never erased one by one, a task's transcript dropped whole; a resume reads the current window, from the turn the core keeps as its start (8.3; engine.md 5.4 and 7.2)"; store.md 3 "The core never erases a single turn: a task's transcript goes whole, by retention (11.1)".

**Today: partly.**
- **Never erased singly: built** — no `Erase` of `CoreKey::Turn` anywhere (core erases only `Projection`, `Call`, `RunProof`: `projections.rs:282,344,350,355`, `routing/tasks.rs:957,960`, `routing/runs.rs:709`).
- **`opens_window`: not built.** `TurnRecord { task, attempt, turn, spent, read, at, transcript }` (`crates/jig-core/src/stored.rs:461–478`); saved once at `crates/jig-core/src/routing/runs.rs:699–707` (`route_accepted_turn`, :681). The fact must come from the hub through the fleet (item 6).
- **Dropped whole: not built.** `Write` has `Save`/`Erase` only (`vocabulary.rs:102–108`); nothing drops a transcript (retention, 11.6).
- **Resume from the current window: not built.** `Now::StartPreparation { task, transcript_waiter }` names no window start (`vocabulary.rs:238`, emitted `crates/jig-core/src/routing/run_preparation.rs:336`); the worlds load every turn of the task (`tests/ops/src/conformance/mod.rs:547–566`, `tests/core/src/effects.rs:507–520`, `tests/conformance/src/lib.rs:461`); `append_transcript` filters by attempt only (`run.rs:163–200`, `transcript.from` = `RunProof.transcript_from`, `stored.rs:523`, set `run_preparation.rs:92`). No core family keeps the window's start (store.md 4.4 core "conversations | task").

**Changes and users:** `TurnRecord` literal constructions: 1 production (`runs.rs:699`) + tests `tests/core/tests/runs.rs` 2, `tests/core/tests/store_peer.rs:36`, `tests/client/src/joint/wire/activity.rs` 3, `tests/client/src/joint/archive.rs:217–218` 2; `CoreRecord::Turn` 33 uses in 22 files. `Now::StartPreparation`: 13 uses in 11 files (`run_preparation.rs`, `crates/jig-core/src/tests/completion_retry.rs`, `testing/jig-test-domain/src/routes.rs:368,375`, `examples/ops/domain/src/route.rs:1215,1222`, `tests/core/src/effects.rs:507`, `tests/core/src/client_peer.rs:151`, `tests/conformance/src/lib.rs:461`, `tests/ops/src/conformance/mod.rs:547`, `tests/ops/src/joint/engine.rs:334`, `tests/ops/src/joint/front.rs:743`, `tests/client/src/joint/session.rs`). A new `Write` variant: root translations `testing/jig-test-domain/src/routes.rs:26–29`, `examples/ops/domain/src/route.rs:214–217`, `testing/jig-test-domain/src/client_documents.rs:86`, `examples/ops/domain/src/client_documents.rs:145`, and the worlds' commit appliers. Ops `Store::Transcript` (`boundary.rs:27`, `restart.rs:89–103`), reference root `Event::TranscriptLoaded` (`testing/jig-test-domain/src/lib.rs:262`).

**Worlds:** `tests/core/tests/store.rs` (`a_crash_before_a_turn_is_durable_loses_only_its_uncommitted_turn` :61, `a_crash_after_durability_before_acknowledgement_restores_the_turn` :116), `tests/core/tests/restart.rs:134` (`a_restart_after_a_turn_committed_resumes_from_it`), `tests/core/tests/runs.rs`, `tests/core/tests/completion_retry.rs`; host world for the window fact (item 6: `tests/host/src/turn_world.rs`).

**Estimate:** 60–120 lines here (field and its plumbing ~10, window-start record ~30, `StartPreparation` naming it and the append check ~30, drop-whole write ~10 + roots ~20). `runs.rs` is 784 lines: the field addition must not grow it past 800. **Depends on item 6** (windows through hub, fleet and kept turn) and **item 7** (the conversation's first attempt); the drop-whole write is used by 11.6.

### 11.5 Fill, and a high-water mark at the entrances

Design: store.md 15.5 "with each commit's answer, and new work refused above the high-water mark (11.2)"; 11.2 "the domain refuses new work at its entrances: new tasks and chats, new sign-ins. Work under way goes on, and a refusal is never a failed commit. The deployment's document shows the fill"; store.md 3 "committed n, with the store's fill".

**Today: not built.** Nothing in the domain names fill (grep: no `fill`/`high_water` outside unrelated uses). The root's commit answer carries the number only (`examples/ops/domain/src/boundary.rs:18`; `testing/jig-test-domain/src/lib.rs:268`), and the core has no event to hear a fill (engine.md 4.1: it does not hear commits, so the root must forward fill as a core event of its own). No refusal word exists: `people::Refusal` (`crates/jig-core-people/src/refusals.rs:8–38`), `LimitKind` (:87–), `Again` (:188–193); the client contract and people.md name none (design gap: the refusal's word and the limit's name are not in `domain/` or `configuration.md` 6). `documents::Deployment` (`crates/jig-core/src/documents/inbox_deployment.rs:142–166`) has no fill.

**Entrances to gate:** sign-in `crates/jig-core/src/routing/people.rs:62–90` (`sign_in`, already refuses on counter exhaustion with `Now::SignInRefused` :78–79); goals and person goal proposals `crates/jig-core/src/routing/parties.rs:134` (`begin_goal_creation`, from `routing/people.rs:240`); chats `routing/parties.rs:191` (`begin_chat_creation`, from `routing/people.rs:282`); possibly `Event::StartRecurring` and `Ask::MakeService` (design silent: "new tasks").

**Changes and users:** root commit answers: ops `Store::Committed` 9 uses in 4 files (`boundary.rs:18`, `domain.rs:203`, `restart.rs:78`, `examples/ops/domain/src/tests.rs` 6: :94,133,228,275,472,587, `tests/ops/src/conformance/mod.rs:236`); reference root `root::Event::Committed` 19 uses in 15 files (`testing/jig-test-domain/src/lib.rs:562`; `tests/core/src/effects.rs:610,1132`, `tests/core/src/world.rs:494`, `tests/core/src/{observations.rs:403, reads.rs:83, watches.rs:195}`, `tests/core/tests/{client_peer_routes.rs:190,199, fuzzy_reads.rs:32, application_documents.rs:119, client_peer.rs:71, reads.rs:29, stories.rs:1465,1642,1736}`, `tests/core/src/stories/reads.rs:292`, `tests/conformance/src/lib.rs:404`, `tests/conformance/src/negative.rs:155`); `core::Limits` (`vocabulary.rs:275`) gains the mark; deployment document field users: core `inbox_deployment.rs:742` (constructor), client `crates/jig-client/src/contract/owner_deployment.rs`, view `crates/jig-client-view/src/owner_deployment.rs`, wire `tests/client/src/joint/wire/deployment.rs`, scripted `testing/jig-scripted-engine/src/tests/owner_deployment.rs` (by `person_answer_retention`'s 12 uses in 10 files). jig-fake-store needs a fill model (it counts nothing: `testing/jig-fake-store/src/lib.rs:122–160`).

**Worlds:** core world `tests/core/tests/store.rs` (commit faults), `tests/core/tests/restore_pages.rs` (`sign_in` :20, `ask` :24 helpers for refused entrances), `tests/core/tests/joint_clock.rs`; deployment document `crates/jig-core/src/tests/deployment_connectors.rs`, `tests/client/src/joint/wire/deployment/tests.rs`.

**Estimate:** 90–150 lines in the domain (event and state ~20, limit ~5, 3–5 entrance checks ~40, refusal word ~10, document field ~15) plus ~40–80 in client contract and view words. Independent of the other sub-items; needs the store component (protocol) to report fill for real.

### 11.6 Retention

Design: store.md 15.6 "summaries of ended tasks, transcripts dropped, answered keys expired, chats forgotten, each on the domain's timer (11.1)"; 11.1 "one commit erases a task's full records (its record, history, activity, terminals) and saves a summary … Transcripts go past a longer horizon, whole … ended chats are forgotten once read"; people.md 6 "A chat is forgotten once it has ended and its person has read its replies and its result; its record and its read position are erased in one decision".

- **Summaries of ended tasks: not built, and not designed in `domain/`.** No summary record, no erase of `Ended`/`History`/`Milestone`/`Activity`/`Terminal` anywhere (tasks erases only `Live` `closing.rs:334`, `Stub` :45, `Writer` `writers.rs:40`). What a summary holds and which reads answer from it appears only in store.md 11.1 (and "retention's horizons" in `configuration.md:66`): a design question for the planner. Finding a task past its horizon needs an index of ended tasks by time (another index, 11.3).
- **Transcripts dropped: not built** (needs the drop-whole write, 11.4).
- **Answered keys expired: built in the child, driven only in the core world.** `people::fire` erases one expired answer per call (`crates/jig-core-people/src/domain.rs:467–477`; also lazily on a repeated ask :1374–1379, and at restore :1608–1610); the core routes `Timer::People` (`crates/jig-core/src/routing/mod.rs:519–525`); the reference root projects its deadline (`testing/jig-test-domain/src/lib.rs:332–340`), and only the core world's `World::fire_due` fires `Timer::People` (`tests/core/src/effects.rs:390–407`, used by `tests/core/tests/joint_clock.rs`); `World::advance` (`effects.rs:410–417`) and the conformance and ops worlds fire Tasks/Fleet/Account/Brief only (`tests/ops/src/conformance/mod.rs:225–228`, `tests/conformance/src/lib.rs:392–393`); ops has `fire` (`examples/ops/domain/src/domain.rs:289–296`) but no deadline projection. Item 10.
- **Chats forgotten: partly.** `core::Event::ForgetChat { row }` (`vocabulary.rs:453`) → `inbox::forget_chat` (`crates/jig-core/src/inbox.rs:493–516`) erases `Ended(task)` and people's `ReadChat` in one decision, but only when a root supplies the ended row: no domain trigger, no timer; marking an ended chat seen does not forget it (`inbox.rs:356–386`). Only `tests/core/tests/activity.rs:211` sends it; reply-less chats are forgotten at ending (`routing/tasks.rs:686–699`). It leaves the chat's activity, history, milestones, terminals and turns behind (`inbox.rs:487–489`: "collected by the store's retention pass", which does not exist).

**Changes and users:** new timer or a tasks/core deadline (`Timer` `vocabulary.rs:636–649`: 6 variants, users `crates/jig-core/src/tests/engine_timers.rs`, `testing/jig-test-domain/src/lib.rs:332–340`, `tests/core/tests/joint_clock.rs:80,101`); horizons in limits (tasks `limits.rs`, people `limits.rs:51` has `request_retention`); summary `Stored` variant (users as 11.1); `ForgetChat`: 13 uses in 11 files (`crates/jig-core-people/src/{boundary.rs:626, domain.rs:422}`, core `routing/people.rs:28,44`, `inbox.rs:507`, `routing/watch_marks.rs:313`, `routing/mod.rs:351,479`, `vocabulary.rs:453`, `routing/tasks.rs:691`; tests `tests/core/tests/activity.rs:211`, `tests/people/src/world.rs:347`, `tests/people/tests/memory.rs:257`).

**Worlds:** tasks world for summaries (`tests/tasks/src/world.rs`, `tests/tasks/tests/lifecycle.rs`); people world for expiry (`tests/people/src/world.rs:937` `advance` already fires people timers, `restart` :916); core world for chats forgotten and transcripts dropped (`tests/core/tests/activity.rs`, `tests/core/src/stories/closing_chat.rs`, `tests/core/tests/store.rs`).

**Estimate:** 300–500 lines (summaries with paged erasure ~200–300, transcript drop on a horizon ~60, chat forgetting triggered by the domain ~60–100, horizons and timers ~40). **Depends on** 11.3 (an ended-by-time index), 11.4 (drop write), item 10 (people timers fired; chats forgotten), and a design decision on the summary's shape.

### 11.7 Keys for single reads

Design: store.md 15.7 "a question's line and its answer's, an escalation's hold, and a task's proposal's row in its proposer's history, each named by its position in its task's record, as a reply's is, so a read finds it by key, never by a scan (7.3)"; 7.3 "an escalation | its decision, by the revision its task's record names, with the hold's milestone".

**The pattern as built:** `ReplyHint { number, activity, attempt, turn, at }` (`crates/jig-core-tasks/src/boundary.rs:832–838`), kept newest-first in `TaskRecord.replies` bounded by `inbox_messages` (:879), made at `crates/jig-core-tasks/src/admission.rs:96–110`; inbox entries carry the exact `activity` (`crates/jig-core/src/documents/inbox_deployment.rs:31–32`), and `Scope::Reply { task, activity }` is a point read. 5 uses (`boundary.rs:832,879`, `admission.rs:97`, `lib.rs:108`, `limits.rs:155`).

- **A question's line and its answer's: not built.** `QuestionCredit { number, answerer }` (`boundary.rs:198–201`; made `crates/jig-core-tasks/src/inbox.rs:353`, dropped on answer :330–336; 6 uses in 4 files, all in jig-core-tasks) has no positions; person questions come from inbox words (`crates/jig-core/src/inbox.rs:172–185`) with `EntryKind::Question { message }` (`crates/jig-core-people/src/boundary.rs:23`; 20 uses, 10 files) and `InboxObject::Question { message }` (`inbox_deployment.rs:17`; 37 uses, 13 files). `Scope::Question { task, message }` admits any `Activity{task,*}` (`rows.rs:1453`) and the object builder matches `Line::Words{kind: Question}` and an `Answer{question}` in `related` (`rows.rs:704–760`): two scans. The core world does not serve it (`tests/core/src/reads.rs:312`).
- **An escalation's hold: partly.** The decision is keyed `EscalationDecision { task, revision }` (`stored.rs:199`) and the record names the revision (`Escalation::revision`, `crates/jig-core-tasks/src/escalation.rs:184–195`), but `Scope::Escalation { task }` admits every revision (`rows.rs:1452`), and the hold's milestone arrives as an unkeyed `related` (`rows.rs:666–679`): `Escalation::Waiting { revision, holder, entry, since }` (`escalation.rs:164–173`) and `Phase::Held { was, why, source }` (`boundary.rs:667–674`) name no milestone position. `HistoricalEscalation` (people's act) is by key already (`routing/party_decisions.rs:477`). `Escalation::Waiting {` is written 69 times in 30 files.
- **A task's proposal's row in its proposer's history: not built.** `Scope::Proposal { Task(t), n }` admits any `History{t,*}` (`rows.rs:1448–1451`); ops serves it by scanning History rows for `proposal.number` (`tests/ops/src/joint/engine.rs:74–130`); the core world does not serve it (`tests/core/src/reads.rs:310`). Nothing records the revision that keeps proposal n (`Proposal { number, proposer, … }`, `crates/jig-core-tasks/src/proposals.rs:131–139`; `History.proposal`, `control.rs:99`). (`CoreKey::ProposalDecision`, `stored.rs:197`, keys person-facing races, read by `tests/core/src/historical.rs:18`.)

**Read vocabulary users if a read names a position:** `Read::Question` 38 uses/16 files, `Read::Escalation` 45/21, `Read::Proposal` 55/26 (mostly jig-client pages routing and tests/client wire); `Scope::Question` 14/5, `Scope::Escalation` 14/5, `Scope::Proposal` 16/6. Keeping the read's shape and resolving the position from the owner record inside the core (live: `core.tasks.task`; ended: the `owner` row) avoids touching the client.

**Worlds:** the core world's transport must serve these by key (`tests/core/src/reads.rs:228–329`, `:310–312`), stories in `tests/core/tests/reads.rs` and `tests/core/src/stories/reads.rs`; the ops scan (`tests/ops/src/joint/engine.rs:74`) becomes a point read; client wire tests `tests/client/src/joint/read_escalations.rs`, `tests/client/src/joint/wire/historical/tests.rs` are unaffected if the read's shape stays.

**Estimate:** 100–200 lines (positions kept in the record for questions/answers and the hold ~60, the proposer's revision for a task proposal ~30, `rows.rs` point mapping ~40, object builders ~30). `rows.rs` (1,744 lines) and `crates/jig-core-tasks/src/boundary.rs` (1,776) are both past 800: changes there need extraction first. **Independent of 11.3**, but both edit `position_allowed`/`key_kind`: sequence them.

### 11.8 Dependencies and order

- Inside item 11: **11.1 first** (key order, past periods, decided proposals; one cursor-codec change) → **11.3** (indexes; 11.2 is the inbox index's restart use) → **11.6** (needs an ended-by-time index, 11.4's drop write). **11.4** follows item 6 (opens_window through hub, fleet, kept turn) and item 7 (first attempt). **11.5** and **11.7** are independent (11.7 after 11.1 if both touch `rows.rs`).
- On other items: 6 (windows) and 7 (first attempt) for 11.4; 10 (people's timers fired; chats forgotten) for 11.6; 12 (crate renames) moves every path above: land the renames before or after this item, not between its increments. The store component (`jig-protocol-store`, protocol) consumes 11.1's live ranges, 11.3's indexes and 11.5's fill; the domain side can land and be proven against jig-fake-store first, but jig-fake-store has no fill, no bytes and no transcript files.
- Files over 800 lines this item touches: `crates/jig-core-tasks/src/boundary.rs` 1,776; `crates/jig-core/src/reads/rows.rs` 1,744; `crates/jig-core-people/src/domain.rs` 1,873; `crates/jig-core/src/routing/tasks.rs` 1,414; `tests/core/src/effects.rs` 1,289; `tests/tasks/src/world.rs` 1,165; `testing/jig-test-domain/src/lib.rs` 1,154; `crates/jig-core-tasks/src/stored.rs` 1,049; `tests/client/src/joint/wire/cursor.rs` 1,036; `testing/jig-test-domain/src/routes.rs` 1,005; `tests/ops/src/conformance/mod.rs` 975; `crates/jig-core-tasks/src/proposals.rs` 874; `crates/jig-core/src/documents/inbox_deployment.rs` 830; `tests/core/src/world.rs` 810. Near the limit: `crates/jig-core-tasks/src/funders.rs` 784, `crates/jig-core/src/routing/runs.rs` 784, `crates/jig-core-people/src/boundary.rs` 781.
- **Total estimate for item 11:** ~1,250–2,130 production lines across 7 sub-items, 11.3 and 11.6 the bulk.

## Item 12. The crates' renames

**Design.** `docs/design/README.md` 15: "`jig-domain-<name>`: a domain's step code: the core and its children, the hub, the client's domain; `jig-protocol-<name>`: the protocol layer's ... `jig-view` ... `jig-shell-<platform>`". "The design's documents use the names as built until the rename lands, and new crates take their names at once." "Every other crate keeps its name: the test kits, the worlds, `ops`'s domains, `jig-test-domain` ... and `jig-boundary`."

**Today.** Not built: every crate carries its old name. The workspace has 47 members (`cargo metadata`); `Cargo.toml:5-50` lists 46 paths, `crates/jig-core` is a member only as a path dependency (`Cargo.toml:69`), so the rename should add it to `members` explicitly.

### The 19 renames and who names them

Columns: manifests naming the crate (dependency or `name`), `.rs` lines naming its `snake_case` path (files), of which `use` lines; Markdown lines (docs). Counted with `grep -P` on word boundaries over `git ls-files`.

| As built (dir) | Renamed | Cargo.toml lines (files) | .rs lines (files) | `use` lines | Manifests that name it (besides the workspace) |
|---|---|---|---|---|---|
| `jig-core` (`crates/jig-core`) | `jig-domain-core` | 8 (8) | 321 (124) | 104 | crates/jig-core, examples/ops/domain, testing/jig-test-domain, tests/{client,conformance,core,ops} |
| `jig-core-accounts` | `jig-domain-accounts` | 11 (10) | 24 (24) | 18 | +crates/jig-core, tests/accounts, tests/{client,conformance,core,ops}, ops/domain, jig-test-domain |
| `jig-core-authority` | `jig-domain-authority` | 10 (9) | 103 (60) | 47 | crates/jig-core, ops/domain, jig-test-domain, tests/{client,conformance,core,ops} |
| `jig-core-brief` | `jig-domain-brief` | 7 (6) | 17 (17) | 14 | crates/jig-core, ops/domain, tests/{brief,core} |
| `jig-core-fleet` | `jig-domain-fleet` | 11 (10) | 63 (35) | 29 | crates/jig-core, ops/domain, jig-test-domain, tests/{client,conformance,core,fleet,ops} |
| `jig-core-notes` | `jig-domain-notes` | 11 (10) | 70 (41) | 26 | crates/jig-core, ops/domain, jig-test-domain, tests/{client,conformance,core,notes,ops} |
| `jig-core-people` | `jig-domain-people` | 11 (10) | 234 (126) | 114 | crates/jig-core, ops/domain, jig-test-domain, tests/{client,conformance,core,ops,people} |
| `jig-core-tasks` | `jig-domain-tasks` | 11 (10) | 367 (153) | 150 | crates/jig-core, ops/domain, jig-test-domain, tests/{client,conformance,core,ops,tasks} |
| `jig-core-views` | `jig-domain-views` | 10 (9) | 72 (36) | 25 | crates/jig-core, ops/domain, jig-test-domain, tests/{client,core,ops,views} |
| `jig-host` | `jig-domain-host` | 6 (5) | 22 (18) | 18 | ops/domain, tests/{host,ops} |
| `jig-client` | `jig-domain-client` | 12 (11) | 1422 (296) | 292 | crates/{jig-client-transcript,jig-client-view}, ops/{client-domain,client-view}, testing/{jig-scripted-engine,jig-test-client-domain,jig-test-client-view}, tests/{client,ops} |
| `jig-client-view` | `jig-view` | 8 (7) | 216 (54) | 49 | ops/client-view, testing/{jig-fake-person,jig-test-client-view}, tests/{client,ops} |
| `jig-charter` | `jig-protocol-charter` | 5 (4) | 11 (6) | 2 | ops/domain, tests/ops |
| `jig-client-transcript` | `jig-protocol-transcript` | 5 (4) | 21 (11) | 1 | tests/{client,ops} |
| `jig-test-client-domain` | `jig-test-domain-client` | 7 (6) | 231 (53) | 33 | testing/{jig-test-client-kit,jig-test-client-view}, tests/{client,ops} |
| `jig-test-client-view` | `jig-test-view` | 4 (3) | 14 (9) | 1 | tests/client |
| `jig-ops-client-domain` (`examples/ops/client-domain`) | `jig-ops-domain-client` | 4 (4) | 39 (19) | 21 | ops/client-view, tests/ops |
| `jig-ops-client-view` (`examples/ops/client-view`) | `jig-ops-view` | 3 (3) | 2 (2) | 2 | tests/ops |
| `jig-ops-connectors` (`examples/ops/connectors`) | `jig-ops-domain-connectors` | 6 (6) | 93 (15) | 9 | ops/{client-domain,client-view,domain}, tests/ops |

Where the `.rs` lines sit (lines/files), for the heavy ones: `jig_client`: crates/jig-client-view 663/120, tests/client 277/69, testing/jig-scripted-engine 143/56, ops/client-domain 125/12, testing/jig-test-client-domain 114/14, tests/ops 43/11, testing/jig-test-client-view 34/8, ops/client-view 22/5. `jig_core_tasks`: tests/tasks 121/18, crates/jig-core 84/55, tests/client 72/30, tests/core 31/28, ops/domain 31/8, tests/ops 23/9. `jig_core_people`: crates/jig-core 64/45, tests/client 50/27, tests/core 47/32, tests/people 42/5, tests/ops 17/7. `jig_core`: tests/client 230/49, tests/core 50/46, tests/ops 18/9, ops/domain 11/10, tests/conformance 9/7. `jig_test_client_domain`: tests/client 151/33, testing/jig-test-client-view 68/16. `jig_ops_connectors`: ops/domain 62/6.

**Paths that name a directory** (only if the directories move with the names; the design names crates, not directories, so the plan decides): `Cargo.toml:5-50` (members) and `:57-92` (workspace.dependencies paths); three `include_bytes!` of `crates/jig-client-transcript/tests/data/record_turn_full.bin` at `tests/client/src/joint/archive.rs:225`, `tests/client/src/joint/wire/activity.rs:314`, `tests/client/src/joint/wire/turns.rs:18`; `tests/boundary/src/tests.rs:859, 889, 945, 958, 973` (`kit_root().join("testing/jig-test-client-domain/src/lib.rs")` and the like).

**Docs.** Non-design Markdown naming them: `docs/development/tech-debt.md` (11 lines), `AGENTS.md` (1), `tests/client/README.md` (2). The design names them in about 105 lines over 20 files (`docs/design/README.md` 13, `client/domain/root.md` 19, `domain/README.md` 13, `domain/engine.md` 10, `domain/hosts.md` 9, `client/domain/README.md` 8, `protocol/channel.md` 7, ...): the designer's to change when the rename lands (README 15). `scripts/check.sh`, `.config/nextest.toml`, `clippy.toml`, `rustfmt.toml` name no crate.

**Formatting.** `rustfmt.toml` has `max_width = 120`; 189 lines naming these crates are over 113 columns, so `jig_client` → `jig_domain_client` (+7) and `jig_core_x` → `jig_domain_x` (+2) reflow some of them: `cargo fmt` after the sed.

### jig-boundary's crate graph check (`tests/boundary/src/tests.rs`, 1,009 lines)

Every rule names crates by string, several by prefix; the new names break the prefixes:

- `:216-240` `core_and_children_use_only_the_kit_and_foundation_even_when_empty`: `name == "jig-core" || name.starts_with("jig-core-")` under `crates/`, and jig-core may link `starts_with("jig-core-")`. `jig-domain-` would also take `jig-domain-host` and `jig-domain-client` (the hub links skein only, the client is apart), so the rule needs an explicit list of the core's children.
- `:243-257` `hosts_and_translation_use_no_core_crates_even_when_empty`: `Some("jig-host" | "jig-inline-agent" | "jig-charter")` and `name.starts_with("jig-core")`. `jig-inline-agent` is stale (the design's agent is smith's `smith-inline-agent`); `starts_with("jig-domain")` would forbid the hub its own name space.
- `:260-273` `host_hub_links_only_skein`: `"jig-host"`. Item 6's design keeps the hub skein-only (smith's payloads as bytes, hosts.md 6.8), so this rule stays, renamed.
- `:708-724` `client_view_links_only_its_domain_and_foundation`: `"jig-client-view"`, `["skein-lib", "jig-client"]`.
- `:727-740` `client_domain_depends_on_foundation_alone`: `"jig-client"`.
- `:743-754` `client_step`: `name == "jig-client" || (name.starts_with("jig-client-") && name != "jig-client-world") || matches!(name, "jig-test-client-domain" | "jig-test-client-view" | "jig-ops-connectors" | "jig-ops-client-domain" | "jig-ops-client-view")`. After the rename the client's crates share prefixes with the engine's: `jig-domain-client` with `jig-domain-*`, `jig-test-domain-client` with `jig-test-domain` (the engine's test application), `jig-ops-domain-client` and `jig-ops-domain-connectors` with `jig-ops-domain`, `-observability`, `-infrastructure` (engine side). `client_step` must become an exact list: `jig-domain-client`, `jig-view`, `jig-protocol-transcript`, `jig-test-domain-client`, `jig-test-view`, `jig-ops-domain-client`, `jig-ops-view`, `jig-ops-domain-connectors` (and later `jig-shell-*`, the client's protocol crates).
- `:757-774` `client_and_engine_step_crates_share_no_dependencies`: `dependency.starts_with("jig-core")` / `name.starts_with("jig-core")` (this also catches `jig-core-world`, which keeps its name): needs an engine-step predicate by list.
- `:777-787` `client_roles_cover_step_crates_and_exclude_the_world`: 9 literal asserts.
- `:791-816` `client_transcript_is_the_client_only_smith_link`: `"jig-client-transcript"`, `["jig-client", "skein-lib", "smith-transcript"]`.
- `:819-831`, `:834-845`: fake person (`jig-client-view`), scripted engine (`jig-client`).
- `:848-864`, `:867-879`, `:882-893`: `jig-test-client-domain` (deps `["jig-client", "jig-test-connector", "skein-lib"]`, path `testing/jig-test-client-domain/src/lib.rs`), `jig-test-client-kit` (deps `jig-test-client-domain`), `jig-test-client-view` (deps `["jig-client", "jig-client-view", "jig-test-client-domain", "skein-lib"]`).
- `:896-938` `client_world_joins_only_the_client_and_test_roles`: a 25-name allow list, 13 of them renamed.
- `:941-948`, `:951-961`, `:964-977`: `jig-ops-connectors`, `jig-ops-client-domain` (deps `["jig-client", "jig-ops-connectors", "skein-lib"]`), `jig-ops-client-view` (deps of five names), each with a source path.

About 55 string literals across 17 tests change; the prefix predicates at `:222`, `:230`, `:251`, `:745`, `:766-767` become lists. No rule yet covers `jig-protocol-*` (no `jig-protocol-*` may be named by a `jig-domain-*` crate, protocol/README.md 5) or `jig-shell-*`: the rename's increment can add them, since jig-protocol's new crates take their names at once.

**Estimate.** Production lines: 0 new; about 3,300 mechanical line edits in 540 `.rs` files (3,284 lines name a renamed crate's path), ~150 manifest lines in 37 manifests (the workspace's included), ~55 boundary literals, and the boundary predicates rewritten (~60 lines). One increment, best landed first, not in the middle of a run of domain changes, since it touches most of the tree (every concurrent increment's `Files:` would conflict). Directory moves (`git mv`) double the diff's noise, not its risk.

**Depends on.** Nothing. Every other item's new crates (`jig-protocol-*`) take their new names at once; landing the rename first saves those increments naming `jig-core-*` in their manifests and then renaming them.

## The suites' times

The last code commit `0e952778` (Gate-Result: "scripts/check.sh passed: 2173 tests / 9.007 s; 86 tests / 12.010 s; 136 s in all"; `91f8bfec`: 9.361 s / 12.467 s; `9af61c4c`: 2185 tests / 10.616 s / 14.049 s). Budgets (`.config/nextest.toml:7-9, 20-22`): focused 15 s, fuzzy 60 s, slow flag at 0.5 s / 5 s. Headroom today: focused ~6 s, fuzzy ~48 s.

Per-binary sums of test time (CPU-seconds over 8 threads, not wall), from the gate's log of `0e952778` (`~/src/rust/tmp/logs/20261012T114702-0.log`, run in `tmp/jig/repin-smith-skein-github`):

| World / crate | focused: tests, sum s, slowest s | fuzzy: tests, sum s, slowest s |
|---|---|---|
| jig-client-world | 324, 15.90, 0.434 (`joint::stories::role_stories::two_parties_decide_one_proposal_and_the_second_is_told_who`) | 35, 35.04, 2.813 (`fuzzy_client random_people_in_chat_starts`) |
| jig-ops-world | 46, 2.81, 0.524 (`client_stories p_accepts_a_restart_from_the_incidents_page`, the run's one SLOW) | 22, 27.36, 4.623 (`fuzzy_client random_people_in_ops_incident_starts`) |
| jig-conformance-world | 51, 4.78, 0.288 | 10, 15.74, 4.494 (`fuzzy_scenarios every_shared_story_survives_every_commit_and_seeded_random_cuts`) |
| jig-core-world | 134, 5.64, 0.336 (`faults every_startup_sign_in_chat_and_assignment_commit_restarts...`) | 8, 5.78, 1.246 |
| jig-fleet-world | 35, 1.36, 0.397 (`simulation an_engine_restart_adopts_what_the_workers_still_host`) | 2, 2.86, 2.633 |
| jig-host-world | 23, 0.43, 0.066 | 2, 1.55, 1.491 |
| jig-tasks-world | 109, 2.11, 0.171 | 1, 0.71 |
| jig-people-world | 36, 0.60, 0.063 | 1, 0.30 |
| jig-views-world | 19, 0.24 | 1, 0.70 |
| jig-accounts-world | 9, 0.09 | 1, 0.03 |
| jig-notes-world | 10, 0.14 | 1, 0.12 |
| jig-brief-world | 9, 0.08 | 1, 0.01 |
| jig-test-connector-world | 22, 0.25 | 1, 0.06 |
| jig-boundary | 88, 6.62, 0.159 (64 vocabulary parts) | — |
| unit tests: jig-client-view 293/16.03 s, jig-client 205/2.69, jig-core 171/2.24, jig-scripted-engine 143/1.68, jig-test-client-view 38/1.89, jig-core-fleet 62/0.66, jig-core-people 49/0.51, jig-core-authority 29/0.48, jig-core-tasks 36/0.42, jig-ops-domain 13/0.31 | | |

The fuzzy suite's wall time is bounded by its longest single tests (4.6 s), the focused suite's by the client world and jig-client-view. A new protocol world, the store's world, and the real store under the core's world will land on these budgets; the host and fleet worlds (items 4, 6) have the most room.

## Files over 800 lines in the crates the plan touches

The plan's limit is 800 lines per file. Every file the item sections above cite, sized with `wc -l` at 2fae240a.

**Step code, roots and kits over 800 that the items edit:**

| Lines | File | Items |
|---|---|---|
| 2994 | crates/jig-client-view/src/phrases/jig.rs | 10.5 (sign-out phrase, 4 places), 6.1 (hello refusal), 4 (account line) |
| 1873 | crates/jig-core-people/src/domain.rs | 1 (a stopping phase), 10.1, 10.3, 11.6 |
| 1776 | crates/jig-core-tasks/src/boundary.rs | 6.3, 6.4 (`Claim`, `Turn`, `TaskRecord.refused`), 10.4 (`Refusal`), 11.1, 11.3, 11.7 |
| 1744 | crates/jig-core/src/reads/rows.rs | 11.1, 11.3, 11.7 (index mapping must go in a new file) |
| 1585 | examples/ops/domain/src/route.rs | 1, 2, 4, 5, 6, 7, 8, 9, 10.3 |
| 1441 | crates/jig-core/src/effects.rs | 10.4 (`procedure_propose`) |
| 1414 | crates/jig-core/src/routing/tasks.rs | 7 (claim-refused paths), 10.3, 11.3 |
| 1340 | crates/jig-host/src/hosted.rs | 4, 5, 6.1–6.4, 6(g), 7, 8 |
| 1310 | examples/ops/domain/src/translate.rs | 9 (shrinks to ~420) |
| 1251 | crates/jig-core-tasks/src/domain.rs | 1, 6.4 |
| 1238 | crates/jig-client/src/domain.rs | 10.5 |
| 1154 | testing/jig-test-domain/src/lib.rs | 1, 2, 4, 5, 6, 7, 8, 11 |
| 1110 | crates/jig-core-fleet/src/attempt.rs | 4 (resend at `moved`/`located`), 6 |
| 1088 | testing/jig-conformance/src/referee.rs | 7 (one first attempt per conversation) |
| 1049 | crates/jig-core-tasks/src/stored.rs | 6.3, 11.1 |
| 1018 | crates/jig-core/src/documents/task_tree.rs | 2 |
| 1006 | crates/jig-core-authority/src/check.rs | 3 |
| 1005 | testing/jig-test-domain/src/routes.rs | 1, 2, 4, 5, 6, 7, 9, 11 |
| 933 | crates/jig-core/src/documents/objects.rs | 2, 11.1 |
| 874 | crates/jig-core-tasks/src/proposals.rs | 11.1 |
| 848 | crates/jig-core-brief/src/orchestrated.rs | 6.4 (`Core::Inbox` removed) |
| 830 | crates/jig-core/src/documents/inbox_deployment.rs | 2, 4, 6.1, 11.5 |

**Under 800 but close, and grown by these items:** `crates/jig-client/src/conversation.rs` 786 (10.7), `crates/jig-core/src/routing/runs.rs` 784 (6.3, 6(f), 7, 10.4), `crates/jig-core-tasks/src/funders.rs` 784 (11.1: will pass 800), `crates/jig-core-people/src/boundary.rs` 781 (10.1), `crates/jig-core-people/src/inbox.rs` 776, `crates/jig-client-view/src/owner_deployment.rs` 758 (4, 6.1), `crates/jig-core/src/lib.rs` 707 (2: the charter table needs its own module), `crates/jig-host/src/boundary.rs` 675, `crates/jig-core/src/routing/vocabulary.rs` 649, `crates/jig-core/src/connector.rs` 649, `crates/jig-core-fleet/src/boundary.rs` 615.

**Test files over 800 the items' tests would grow:** `crates/jig-core/src/tests.rs` 2936, `tests/core/tests/stories.rs` 1886, `crates/jig-core-people/src/tests.rs` 1592, `tests/client/src/joint/session.rs` 1553, `crates/jig-core-authority/src/tests_policy.rs` 1339, `tests/fleet/src/world.rs` 1320, `tests/core/src/effects.rs` 1289, `crates/jig-core-fleet/src/tests.rs` 1268, `crates/jig-client/src/tests/conversations.rs` 1171, `tests/tasks/src/world.rs` 1165, `tests/client/src/joint/packets.rs` 1104, `tests/core/src/observations.rs` 1080, `tests/people/src/world.rs` 1062, `tests/host/src/world.rs` 1048, `tests/client/src/joint/wire/cursor.rs` 1036, `tests/ops/src/joint/front.rs` 1035, `tests/boundary/src/tests.rs` 1009, `tests/ops/src/conformance/mod.rs` 975, `crates/jig-core/src/tests/acts.rs` 973, `crates/jig-core-tasks/src/tests/revisions.rs` 960, `tests/client/src/harness/world.rs` 840, `examples/ops/domain/src/tests.rs` 835, `tests/core/src/world.rs` 810.

The workspace has 70 `.rs` files over 800 lines in all (`find crates testing tests examples -name '*.rs' | xargs wc -l`); tech-debt.md sections 3–7 hold them, and only those an increment edits need splitting under the plan's rule.
