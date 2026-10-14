# Retrospective: client-composition

2026-10-14. The run of jig's `client-composition` on carbon-forge,
claimed at 2026-10-14T12:34:19Z and done at 16:50:53Z. It was the first
run after the user decided every proposal of the last retrospective
(test-budgets). Written by a fresh session that took no part in the run
(development.md, section 15).

Six read-only sub-agents, one per source, read:
- `log.md`, the reports, the ledger and the checkpoints;
- jig's, skein's and smith's git history, with the trailers;
- the coordinator's Codex rollout;
- the 13 implementer rollouts;
- the supervisor's two Claude Code transcripts and their 13 sub-agents;
- the designer's four sessions, the planner's five, and the session that
  designed and planned the run.

This session read:
- the plan, with `decisions.md` and `later.md`;
- `done.md`, `supervision.md`, `context.md` and `logs/heavy.tsv`;
- test-budgets' retrospective, and the reliability retrospective's
  section 10.

Times are UTC. Costs are at Opus 5.5's list prices. This document
proposes; the user decides, and the designer applies what is adopted.

## 1. In one page

- **Done in 4 h 16 min, and every "Done when" item holds.**
  - 36 steps landed, all of them planned, with none added or dropped.
  - 8 escalations, answered in a median of 1.9 minutes, and 2 steers.
  - Three design questions, each answered in 4 to 6 minutes, with the
    design changed to match.
  - Nothing waited on the user.
- **The work is done, and cheaply in test time.**
  - jig's pages are a library an application arranges.
  - Owners are jig or an application's number, and every page keeps one
    outline, which the referee checks.
  - jig has a text shell, and `ops` uses each piece.
  - The focused suite's median is 9.18 s, against 8.49 s at the base and
    an aim of 12 s: the plan allowed 3.5 s and used 0.7 s.
- **The adopted proposals worked where they were in force.**
  - Plan defects fell from 0.28 escalations a step to 0.08. The
    supervisor's ready checks put about 51 corrections into session
    files before their sessions started, and no block was a missing user
    or copy.
  - The queue all but emptied: tasks waited 0.06 seconds for each second
    they ran, against 2.13. `land`'s build-lane pass did most of that.
  - The coordinator re-typed nothing and never compacted. It used 1.16M
    input tokens a step, against 1.84M.
  - The supervisor handed over at 296K and never passed 300K.
- **What cost time: on the chain that set the run's length, about 70 of
  its 256 minutes went to waiting for ready checks.**
  - A session's check starts only once what it needs has landed. The
    supervisor runs it while its watch is off, one check at a time.
  - The coordinator's Codex thread unloaded six times while idle. Each
    time, the supervisor loaded it again by hand.
  - The supervisor's handover lost a pending check. jig 08.2 waited 27
    minutes for it.
- **The tail ran one session at a time,** as in jig-protocol.
  - jig 08.2 needed only 06.1 and 06.2, both landed at 14:48. It stood
    second in a session that waited for jig 07, and started at 16:07.
  - About 45 minutes could have been saved (inferred).
- **The planning cost more than the run.**
  - The planner's sessions cost $55, and the run's Claude Code roles
    $46. The design phase cost another $24 and the session that ran them
    both $4; the metrics count neither.
  - The planner ran to 617K in one context.
- **A commit's trailers were unreadable for the third run in a row**
  (jig-06.1). `land`'s refusal waits on skein's `kit-after-parity`.
- **Ten proposals** (section 8). The first five target the run's
  critical chain; the next two, the implementers' and the planner's
  tokens.

## 2. Outcome

The supervisor ran the plan's "Done when" at 16:53Z, in a worktree at
`origin/main` 9cba0a532 (`done.md`). This session checked items 2, 3, 4,
6 and 8 again on `origin/main` b254cd2e6.

| # | Item | Result |
|---|---|---|
| 1 | Every step landed or dropped | Holds: 36 of 36 landed, none added or dropped. `plan status` still says that no commit on `origin/main` names jig-06.1. Its commit `cdfce095a` joined its trailers with a literal `\n` (ruling 110; section 5, item 7). |
| 2 | Owners are jig or an application's number | Holds: the grep prints nothing. It has held since jig-01.1 landed at 12:45Z. |
| 3 | The referee checks the outline | Holds: `tests/client/src/harness/world.rs:631`. |
| 4 | A page is its sections | Holds: `pages.rs:22` (`pub enum Section`), `:39` (`pub struct Arrangement`), `sections.rs:10` (`pub fn section(`). |
| 5 | The new stories and the text shell pass on `main` | Holds: "55 tests run: 55 passed, 381 skipped", in 0.293 s. |
| 6 | Commits for each session | Holds. 35 commits carry the `Plan:` trailer, and jig-06.1's is counted by ruling 110. By session, jig 01 to 10: 4, 4, 4, 3, 3, 4, 4, 3, 3, 4. |
| 7 | The aim, about 12 s | Holds: 8.846, 9.182 and 9.183 s, a median of 9.182 s. |
| 8 | No budget raised | Holds: the diff of `.config/nextest.toml` is empty. |
| 9 | The aims are these items | Holds. |

## 3. Numbers

`plan metrics --history` has four earlier runs:
- test-budgets, 39 steps in 3.7 h, is the nearest in size and in kind;
- jig-protocol had 217 steps in 29.8 h;
- tls-cleanup had 13 steps in 1.3 h;
- reliability had 108 steps in 36 h, in smith, with live probes.

Per-step numbers compare better than totals.

| Metric | client-composition | test-budgets | jig-protocol | tls-cleanup | Notes |
|---|---|---|---|---|---|
| `wall_h`, `running_h` | 4.3, 4.3 | 3.7, 3.7 | 29.8, 29.5 | 1.3, 1.3 | No pause. Landings by hour: 12, 8, 8, 8. |
| `steps_planned`, `added`, `dropped` | 36, 0, 0 | 36, 3, 0 | 166, 51, 0 | 11, 2, 0 | The first run with no step added. |
| `step_h` | 0.12 | 0.09 | 0.14 | 0.10 | At most two sessions could run at once, by the plan's dependencies; the tail was one session (section 5, item 3). |
| `escalations` | 8: plan defect 3, design gap 2, process 2, cross-session scope 1 | 21 by content | 255 | 12 | 0.22 a step, against 0.54, 0.97 and 0.92. One more question, 06.3's, came from a ready check, not a block. |
| `escalation_wait_min` | 1.9 | 1.8 | 1.6 | 1.3 | Process and plan-defect blocks took 0.5 to 4.6 min; design gaps 7 to 9 min, with an interim ruling for the first. |
| `blocked_h` | 0.4 | 0.6 | 8.2 | 0.2 | 9% of running hours, against 16%, 28% and 15%. |
| `answers_reversed` | 0 | 3 | 38 | 2 | Ruling 114 corrected 113's reading of a size limit, without a `Reverses:` line. |
| `steers` | 2 sent, 0 refused | 4 | 24 | 1 | |
| `restarts` | none | none | none | none | The six times the supervisor loaded the coordinator's thread by hand are not counted (section 5, item 2). |
| `tokens_coordinator` | 41.6M in (99% cached), 74K out | 71.8M, 162K | 699M, 1.5M | 15.9M, 55K | 1.16M a step, against 1.84M, 3.22M and 1.23M. Peak 258K at its last request. |
| `tokens_implementers` | 168.4M in (98% cached), 369K out | 201.9M, 479K | 2,761M, 9.8M | 43.2M, 175K | 4.68M a step, against 5.18M, 12.7M and 3.3M. Requests that saw nothing were 24% of the input, against about 36%. |
| `tokens_claude` | 107.4M in the run; 187.9M before the claim; 63.1M after | 62.7M in the run; planning not counted | 1,184M counted once | 227.6M | 2.98M a step in the run, against 1.61M (below). |
| `cost_claude` | $132: planner $55, supervisor $37, designer $9, retrospective $31 | not measured | not measured | not measured | The run's part, $46, is $1.28 a step. The retrospective's is this session's and its readers' when `metrics.tsv` was written. |
| `tokens_per_step` | 8.84M | 8.65M | 21.5M counted once | 9.9M for its run | Flat: the coordinator and implementers fell, Claude Code rose. |
| `coordinator_requests` | 292, 12 after a wait ran out | 398, 9 | 3,567, 67 | 147, 4 | 8.1 a step, against 10.2. The 12 empty wakes were 9% of its input. |
| `compactions` | 0 | 1 | 9 | 0 | |
| `gates_per_step` | 1.3 | 1.7 by this definition | 2.2 | 2.5 | 46 land gates for 36 landings: 4 failed, 6 ran again after main moved. |
| `gate_lane_other` | 1 | 56 | — | — | The final measurement. No Proof was measured in the gate lane. |
| `queue_wait_ratio` | 0.06 | 2.13 | 2.24 | 0.66 | By hour: 0.09, 0.08, 0.06, 0.01, 0.01. |
| `suites` | jig: 3,517 / 8.585 s → 3,636 / 9.535 s; fuzzy 109 / 12.988 s → 13.707 s | | | | Single gates. The final median of three is 9.182 s. |
| `proofs` | unit 13, world 18, joint 3; "1 below plan" | unit 15, world 12, process 9 | | | Wrong by two (section 5, item 11): none is below its plan. |
| `report_lines` | 10 median, 10 p90 | 11, 14 | 10, 11 | 10, 10 | 55 reports, all logged by their implementers. |
| `lines` | code +3,767 −930; tests +4,422 −244 | **not comparable** | | | By hand: production +3,799 −932; tests +4,496 −246, of them goldens +760 and test kit +735 −118. |
| `pin_lag` | jig: skein 9, smith 31; temper: jig 115 | jig: 1, 1 | | | No repin, by decision 2. Other runs moved skein and smith. temper is the user's (section 9). |
| `contexts` | 189K median, 241K p90; 5 handovers | 142K, 209K; 3 | 238K, 319K; 70 | not measured | Ten sessions of three or four increments of new code, against 14 small ones. |

**What explains the largest differences:**

- **The queue fell from 2.13 to 0.06, for three reasons.**
  - **`land`'s build-lane pass,** adopted after test-budgets, ran 47
    times (3,925 s) outside the gate lane. The 46 land gates then ran
    39 s each on a warm build.
  - **No Proof was measured in the gate lane.** test-budgets measured
    every one there: 56 of its 121 gate-lane tasks.
  - **At most two sessions ran at once,** where test-budgets started
    seven together.
- **Claude Code's tokens a step rose from 1.61M to 2.98M,** for two
  reasons, though the supervisor's own requests were smaller:
  - **13 supervisor sub-agents, $20.32 of its $36.90:**
    - eight ready checks, $13.09;
    - four hourly checks, $6.41;
    - an investigation, $0.82.

    test-budgets had neither ready checks nor hourly checks in
    sub-agents.
  - **Three design questions, $8.97.** test-budgets had none.
- **The coordinator's tokens a step fell by 37%.** It re-typed no report
  and never compacted.
  - Its output went to: writing `context.md`, the ledger, `done.md` and
    `blocked.md` (37%); checks (23%); relaying rulings word for word
    (14%); deciding (6%).
- **The implementers' tokens a step fell by 10%, and their contexts
  rose.**
  - Requests that saw nothing fell from about 36% of the input to 24%.
  - Each thread read 64 to 95K before its first edit, against 80 to 125K,
    and every later request carried it: 52% of all input.
  - Sessions were larger: 101 to 150 requests at average contexts of 107
    to 161K.

**Not comparable, and why:**

- **`cost_claude`** is new. No earlier run has it.
- **`tokens_claude` before the claim** counts the planner's sessions
  only, by the plan's `Planner-Session:` trailers.
  - The design phase, $24.35 in three sub-agents, is not counted.
  - Neither is the session that ran design and planning and started the
    run, $3.78.
  - test-budgets' planning was 412M input tokens, not counted by its
    metric.
- **`gates_per_step`** now counts `land`'s gates alone (development.md,
  15.2). test-budgets' 3.1 counted every gate-lane task; its land gates
  were 65 for 39 steps, 1.67.
- **`proofs`** reads 35 steps, not 36, and misreads one (section 5, item
  11).

## 4. What worked

1. **Every proposal was decided before the run, and most were in force.**
   - test-budgets' section 10 landed at 09:24Z (jig f4f0feb). The
     designer applied the proposals in skein 479f836, 63fff2d and 7f4b0e8
     and in smith b7da2b25, all before the claim at 12:34Z.
   - Two kit changes wait on skein's `kit-after-parity`: `land`'s turn
     lock and its trailer refusal (section 7).
   - Of test-budgets' six recurring items, one came back: unreadable
     trailers. That was one of the two still waiting.
2. **Ready checks moved plan defects before the start.**
   - The supervisor checked eight session files against `main` before
     their sessions started, and 08.1 again after 08.2 moved two of its
     lines.
   - The checks compiled the sketches under clippy and corrected about 51
     points: stale lines, users missing from Files, closures and derives
     the lints refuse, missing bounds. These became rulings 101–104,
     107, 108, 112, 116 and 119.
   - No block was a missing user or copy; test-budgets had nine. Plan
     defects fell to 3 blocks, 0.08 a step.
   - No answer was reversed.
   - One check, of jig 06, found a design question before anyone wrote
     code (ruling 109).
3. **The gate lane ran only gates, and they passed.**
   - 4 of 46 land gates failed (9%, against 14%). Each was a real test
     failure: a role check, a forbidden word, three refusal notices, and
     the fuzzy acknowledgement that became ruling 118.
   - No gate failed on clippy, a dirty tree or a timeout. `land` refused
     an uncommitted tree at once (jig-06.1, exit 2), not after a wait.
   - No `land` gave up, and no rebase conflicted. That includes jig 05's
     `git mv` of five pages into `sections/` and the six shared files.
4. **The coordinator kept up with less.**
   - Implementers logged all 55 reports with `plan log`, and the
     coordinator typed none.
   - It set a session to `check` within 12 s of its prerequisite's
     report, and spawned within 8 to 26 s of the ready ring.
   - It ended its turn with `plan idle` eight times, and the doorbell
     restarted it each time.
   - No session waited for a slot or for the coordinator.
5. **The supervisor stayed small, and its rulings were written once.**
   - It handed over at 296K, as the 15:33 check advised. The handover
     cost $0.80, and the successor peaked at 195K. In test-budgets, 55%
     of the supervisor's input was above 300K; here none was.
   - All 20 rulings went through `plan ruling`, and none was re-typed
     into `decisions.md` or `supervision.md`.
   - Both sessions used Claude Code's 1-hour cache. None of their 20
     gaps over 5 minutes re-wrote the context. With a 5-minute cache they
     would have re-written about 3.7M tokens, about $18.7.
6. **Design questions reached the designer, and the design changed.**
   - Three questions were each answered within 4.4 to 6.4 minutes of
     being asked:
     - the shell's name is a node's text (view.md 15.1, 346aaa4c7);
     - a card a shown conversation holds is never owed (view.md 4.1,
       38c0a1c7);
     - only the engine's restart excuses a taken delivery
       (`protocol/client.md` 7, `engine.md` 11, `testing.md` 6,
       74bce1ac2).
   - Each answer held to the end.
   - Ruling 105 let 09.1 go on with the work that did not rest on the
     answer, as test-budgets' proposal 12 asked.
   - No `later.md` row was left unchecked against the design.
   - The designer cost $8.97. Two of its sessions were fresh from a
     brief; the third resumed a warm cache.
   - The third answer rejected both the supervisor's and the
     implementer's reading, and checked its own once on a scratch copy
     before answering.
7. **The sides were connected from the start** (skein e30fe58, the
   user's change of that morning).
   - jig 03.2's meeting-point story ran the domain's sections against the
     real view and the engine's domain before jig 04 started.
   - No defect came from the domain's and the view's sections meeting
     later.
   - The one surprise between sides came from `ops`'s joint world, given
     the arrangement only in 08.2. It found a real gap in the design.
8. **Test time stayed low.**
   - The focused median rose 0.69 s, of 3.5 s allowed. The fuzzy
     suite's last gate, 13.707 s, is below the base's median of
     13.96 s.
   - No budget was touched, and no test moved to the fuzzy suite.
9. **Implementers followed the card's reading rules.**
   - No thread read development.md, and none called the code graph's
     `list_projects`. Every graph call named its project.
   - Edits outside Files were named in commit bodies (implementer.md
     8.2): 04.2's two fixtures, 07.3's line, 09.3's 11-line renderer
     fix, 10.3's `clippy.toml` line. None became a block.

## 5. What did not

1. **Sessions waited for their ready checks: about 103 minutes in all,
   about 70 of them on the chain that set the run's length.**
   - **What:** each wait ran from the coordinator setting the session to
     `check` until the supervisor rang `plan ready`:

     | Session | Wait (min) |
     |---|---|
     | jig 02 | 14.8 |
     | jig 04 | 7.6 |
     | jig 05 | 10.1 |
     | jig 09 | 11.0 |
     | jig 10 | 8.6 |
     | jig 06 | 16.5 |
     | jig 07 | 6.6 |
     | jig 08.2 | 26.7 |
     | jig 08.1 | 1.2 |

     The binding chain was 01 → 02 → 05 → 06 → 07 → 08 → 10.4. During
     four of the waits no implementer was running at all.
   - **Cause:**
     - **A check starts only once the session's prerequisites have
       landed** (development.md, 7.2), so it always sits on the chain.
     - **Each check runs in a sub-agent while the supervisor's watch is
       off.** Checks ran one after another: jig 06's was printed at
       14:22:46, behind jig 10's, although it had been in `check` since
       14:15:44.
     - **The coordinator's thread had unloaded** (item 2), which added
       29 to 80 s to four starts.
   - **The fix is shown by ruling 119.** Re-checking 08.1 after 08.2 had
     landed took 43 s, because the check against `main` was already done
     and only 08.2's moved lines needed reading.
   - **First visible:** 13:08–13:12, jig 02's start. The 14:40 check
     noted that "ready checks held starts". **Acted on:** not.
2. **The coordinator's thread unloaded six times while idle.**
   - **What:** at 13:11, 13:46, 14:22, 15:12, 16:19 and 16:32, a ring
     from `plan ready` or `plan ruling` found no loaded thread: "thread
     not found".
     - `plan resume`, which the card tells the supervisor to run, failed
       the same way. Its `thread/read` answers from the rollout, so it
       skips `thread/resume`, and `turn/start` then finds nothing.
     - The supervisor loaded the thread by hand with `codex.sh`'s
       `thread/resume`, then rang again. That was 28 requests and $1.25.
     - At 16:19 the handover's copied command named `~/src/rust/`, which
       this machine does not have, and failed once.
   - **Cause:** Codex's daemon unloads an idle thread, and `plan ring`
     and `plan resume` do not load it again (`plan`'s `ring` and
     `resume`, lines 842 and 979).
   - **First visible:** 13:11. **Acted on:** each time by hand. The
     handover entry wrote down the procedure; the kit fix was left for
     after the parity run.
3. **The tail ran one session at a time.**
   - **What:** from 15:06 one session ran, through jig 07, 08 and 10.4.
     - jig 08's file gave 08.2 "Needs: jig 06.1 and 06.2", both landed
       by 14:48. The README's "jig 08 once 07 has" held the whole
       session for 08.1.
     - The 15:33 check saw it. Ruling 114 and the 15:40 steer put 08.2
       first, but its check then crossed the handover (item 4), and
       08.2 started at 16:07.
     - Five of the coordinator's twelve empty wakes fell between 15:17
       and 15:46, while jig 07 ran alone.
   - **Effect (inferred):** with 08.2 started at about 14:50, it would
     have landed by about 15:40, its 20 minutes of questions included.
     08.1, 08.3 and 10.4 would then have followed 07.3 (15:45), and the
     run would have ended at about 16:05, 45 minutes sooner.
   - **Cause:**
     - The session table names what each session needs at its start,
       and which increments need more. It has no column for an
       increment that needs less.
     - The planner put 08.2 second.
     - jig-protocol's proposal 12 named this case; it was never decided.
   - **First visible:** in the plan, at 12:28. In the run, at the 15:33
     check. **Acted on:** 15:40.
4. **The supervisor's handover lost a pending check.**
   - **What:** the coordinator set 08.2 to `check` at 15:40:22, and the
     supervisor handed over at 15:40:34.
     - The handover entry said "the successor's watch prints it again".
       It does not: `plan watch` records each event once in the run's
       `.watch`, which the two sessions share (`plan`, lines 721–728).
     - The successor read the ledger at 15:40:57 but started the check
       only at 16:00:27, when a block brought it to look. `plan ready`
       came at 16:07:04.
     - A slot sat idle from 15:50 to 16:07, on the chain.
   - **Cause:** `plan watch` does not show a new supervisor what its
     predecessor left open.
   - **First visible:** 16:00. The 16:17 check noted the idle slot.
     **Acted on:** not.
5. **Defects in increments no ready check read.**
   - **What:** a ready check reads a session's first increment, or its
     first two. That covered 12 of 36. All three plan-defect blocks
     came from the other 24:
     - **10.3 (14:48):** the plan said skein-world gives no pipes.
       jig 10's check had found the same premise, for 10.1, and missed
       `World::run_with_inputs`; its correction stopped at 10.2.
       Ruling 111.
     - **07.2 (15:30):** 204 lines against a Size of 120. Ruling 113.
     - **09.3 (15:59):** `for_world` overflowed on the inbox's limits,
       and the fixture took 263 lines against 120. Ruling 115.
   - **One correction was not run.** jig 08's check (ruling 116) told
     08.2 to configure the arrangement in `ops`'s joint world. The check
     did not apply it, and ran the three packages' default profile, not
     the fuzzy suite. 08.2's gate then failed in the fuzzy joint world
     at 16:16, ten minutes after the ruling. Rulings 117 and 118, about
     20 minutes on the chain, found a real design gap. Run by the check,
     the failure would have shown before 08.2 started, and the question
     could have gone to the designer before the session's work began.
   - **First visible:** 14:48. **Acted on:** each in its ruling.
6. **Implementers' tokens went to carrying their reading and to
   waiting.**
   - **Reading:** each thread had read 64 to 95K before its first edit
     (median context 91K), and every later request carried it: 52% of
     the implementers' input.
     - All 13 read skein's `programming-model.md` whole: 13.1K, past
       the tool's cap, so it was cut and read again by range. All 13
       also read `testing-strategy.md`, and 12 read `notes.md`. That is
       14 to 23K a thread.
     - The card names the first two as rules to follow, and no
       sections.
     - Six threads opened by reading three to six files in one call,
       which was cut; the card says one at a time.
     - The three successors after a handover read the card first, not
       their checkpoint, then 64 to 74K of the same start-up set.
   - **Waiting:** requests that saw nothing were 24.3% of input (40.9M):
     - 172 empty polls;
     - 108 launches with a 1-second yield that came back empty at once.

     Codex's exec cell returns at 31 s, and the thread then waits about
     25 s more, so each minute of a running command costs two requests.
     One thread waited 10 s at a time, 22 times.
   - **First visible:** the 13:37 check, at 3.4M input a landing.
     **Acted on:** not.
7. **A commit's trailers were unreadable, for the third run in a row.**
   - **What:** jig-06.1 committed with `-m "Proof: …\nPlan:
     client-composition\nStep: jig-06.1"`. git read one `Proof:` trailer
     and no `Plan:` or `Step:`.
     - Ruling 110 accepted it by a grep of the body.
     - `plan status` will disagree for good.
     - The metrics leave the step out of `proofs`.
     - jig-02.1 (test-budgets) and jig-10.1 (jig-protocol) did the same.
   - **Cause:** test-budgets' proposal 7, `land`'s refusal, is adopted
     but waits on `kit-after-parity`. implementer.md 8.4 says the shape,
     but nothing checks it before `land`.
   - **First visible:** 14:41, by the coordinator's landed check.
     **Acted on:** 14:43, ruling 110, passed on to every later session.
     No later commit repeated it.
8. **The planning cost more than the run.**
   - **What:** the planner's five sessions cost $55.11. The run's Claude
     Code roles cost $45.87.
     - The planner itself, $38.99, ran from 31K to 617K in one context.
       216 of its 373 requests were above 400K, and cache reads were $30
       of its cost.
     - While its two reviewers ran (12:11 to 12:24), it made 145
       requests at 465 to 567K: $16.96. About 57 of them polled the
       reviewers' files or its own monitors, though a background agent
       notifies when it ends.
     - The design phase's designer slept 600 s waiting for its reviewers.
       Its sub-agent's 5-minute cache expired, and it re-wrote 419K.
     - Design and planning together cost $83.24, $28.13 of it not
       counted by the metrics.
   - **What it bought:** sketches compiled at the base under clippy,
     and every Proof run once there.
   - **What it missed:**
     - **Users still missing from Files,** which the ready checks added
       (rulings 103, 104, 107, 112 and 116). The planner searched by
       name, but never opened `ops`'s joint world, whose second client
       copies the application's setup (116).
     - **Every design gap.** The first answer's text was there to read
       (view.md 15.1, bounded by the view's limits, beside a name with
       no bound). A design reviewer looked at that line, and asked only
       for defaults.
   - **First visible:** 13:37, the first check: "Claude Code has cost
     $61 since the planning". **Acted on:** not.
9. **Test-time shares were smaller than one gate's noise.**
   - **What:** shares were 0.2 to 0.8 s a session. One gate's focused
     time swings by about 1.3 s:
     - 04.1 at 9.861 s, then 04.2 at 8.534 s;
     - 10.1 at 9.881 s, then 10.2 at 8.898 s.

     By the README's rule, jig 04 passed its share at 04.1, by 1.14 s
     beyond the 0.2 s. Neither the report nor any check noticed, and it
     was noise.
   - **Effect:** none on the suite: the whole run added 0.69 s. But the
     rule measured nothing it could act on.
   - **First visible:** 13:22. **Acted on:** 10.1's case, by the 14:40
     check ("likely the gate's variance").
10. **Two sizes asked, and two landed at their margin.**
    - **Asked:**
      - 07.2: 216 production lines against a Size of 120.
      - 09.3: 263 lines of test kit against 120.
    - **At 1.5× exactly:**
      - 09.1: 447 lines, its commit says, against a Size of 300;
      - 10.1: 375 lines against a Size of 250.
    - **Far under:** changes to existing code, such as 05.3 (27 lines
      against 200) and 04.3 (3 against 100).
    - **Cause:** new code was sized low, and moved code high.
    - **First visible:** 15:30. **Acted on:** rulings 113 and 115.
11. **Small things in the record:**
    - **`plan metrics`:**
      - reads 02.4's `Proof: world-and-joint` as no layer, so "1 below
        plan" (numbers.sh, line 64);
      - leaves jig-06.1 out (item 7);
      - counts none of the six times the thread was loaded by hand as a
        restart.
    - **The coordinator:**
      - asked three sessions to hand over just after their last
        increment had landed (jig 05, 06, 08): about 1.5M tokens for
        nothing;
      - sent the 15:40 steer's text to jig 08 in a message after its
        spawn, not in the prompt (development.md, 7.2);
      - wrote `done.md` with "remains to be appended" and a "Pending"
        section, then appended to it after the release;
      - fixed `thread=/root/wN` in the ledger by ten more `plan set`
        calls.
    - **The release:** `plan release` refused at 16:51 because of
      `wt/temper-downstream`, the scratch worktree `plan downstream`
      leaves in the run's directory. Ruling 120 removed it.
    - **Handovers:** jig 06's handover has no `Handover` line in
      `log.md`.
    - **The supervisor's hourly-check entries** were re-typed from the
      sub-agents' output, 2.8 to 3.2K characters each.
    - **This retrospective's own cost:** $31 and 63.1M input tokens when
      `metrics.tsv` was written, two-thirds of the run's Claude Code
      cost. Most of it went to its six readers of the traces.
      test-budgets' was 66.7M.

## 6. Decisions taken during the run

Rulings 101 to 120 and two steers. The supervisor marked each in
`done.md` with its design commit, or "not in the design yet".

| Rulings | What | Reached the design | Proposal |
|---|---|---|---|
| 105, 106 | The shell's name is at most a node's text; `Terminal::new` refuses a longer one | Yes: `view.md` 15.1, 346aaa4c7 | — |
| 108's question, 109 | A card a shown conversation holds stays there and is never owed; `Conversation::shows` moves into the domain | Yes: `view.md` 4.1, 38c0a1c7 | — |
| 117, 118 | Only the engine's restart excuses telling the core of a taken delivery; the referee forgets at the cold hook | Yes: `protocol/client.md` 7, `domain/engine.md` 11, `client/domain/testing.md` 6, 74bce1ac2 | — |
| 101–104, 107, 108, 112, 116, 119 | Session files corrected against `main` by the ready checks | Not a design matter: the session files, amended in the run | 1, 3 |
| 110 | A commit's paragraphs and trailers written with real newlines | implementer.md 8.4 says the shape; `land`'s refusal is on `kit-after-parity` | 8 |
| 111 | 10.3 reaches the simulator through `World::run_with_inputs` | Not a design matter: skein-world's own interface | 3 |
| 113, 115 | 07.2 at up to 215 lines; 09.3 fixes `for_world` at up to 200 | Not a design matter | Section 5, item 10 |
| 114, steer of 15:40 | jig 08 starts at 08.2 | The plan only | 2 |
| 120 | `plan downstream`'s scratch worktree removed for the release | The kit | Section 9 |
| Steer of 14:40 | One behaviour per test name | Already in implementer.md 8.2 and development.md 5.6; a test before the steer broke it | — |

**Decided by the user during the run:** nothing; nothing was asked.

**Still with the user:** `decisions.md`'s "For the user", four
long-term choices the planner made under the user's delegation:
- the shortcut's modifier and key;
- region names as an owner and a `u16`;
- the owner number in the older names `Part.connector` and `family`;
- the terminal's interface.

None has been vetoed.

## 7. The proposals adopted after the last retrospective

test-budgets' fourteen, as its section 10 decided them. Twelve were in
force at the claim. Proposals 4 and 7 wait on skein's `kit-after-parity`.

| Proposal | Decision | Meant to move | test-budgets | client-composition | Moved |
|---|---|---|---|---|---|
| 1. Decide before the next run | adopted | the next section 7; what recurs | none decided; six recurred | all decided by 09:24Z; one recurred, from a deferred proposal | Yes |
| 2. Search a changed thing's name, copies and users; the ready check | adopted | plan-defect escalations; `answers_reversed` | 0.28 a step; 3 | 0.08 a step, none a missing user; 0 | Yes. About 51 corrections reached session files before their starts. The checks also cost 103 minutes of waiting (section 5, item 1). |
| 3. Generated code proves itself downstream | adopted | `steps_added` | 3 | no generated code changed | Not tested |
| 4. `land` holds the repository's turn | after the parity run | land gates a landing; `queue_wait_ratio` | 1.67; 2.13 | 1.28; 0.06 | Moved without it: 6 gates ran again after main moved, against 17 |
| 5. Nothing dirty or unbuilt enters the gate lane | adopted | failed gate-lane runs; queue | 5, after 24 minutes of waits | 0; one uncommitted `land` refused at once | Yes |
| 6. The prompt and card say what to read and how to wait | adopted | `tokens_implementers`; `contexts` | 5.18M a step; about 36% empty; 80–125K read | 4.68M; 24% empty; 64–95K read | Yes, in part. Contexts rose with larger sessions (section 5, item 6). |
| 7. `land` refuses unreadable trailers | after the parity run | `proofs`; `plan status` | jig-02.1 | jig-06.1 | No: not in force, and it recurred |
| 8. The landed check reads the branch's tip | adopted | process escalations | ruling 104 | none of that kind | Yes |
| 9. `plan log`; the coordinator types no report | adopted in part | `tokens_coordinator`; `compactions` | 1.84M a step; 1 | 1.16M; 0 | Yes |
| 10. The supervisor hands over at 300K | adopted | `tokens_claude` | 425K; 55% of input above 300K | handed over at 296K; none above | Yes per request. Claude Code's tokens a step rose, from its sub-agents and the designer (section 3). |
| 11. `plan metrics` counts what 15.2 says | adopted | comparability | planning, escalations, proofs, suites | planning counted; escalations once; suites by repository | Mostly. The design phase, a hyphenated Proof and unparsed trailers are still miscounted (proposal 9). |
| 12. A design gap goes to the designer while the step goes on | adopted | unchecked `later.md` rows | 1 | 0 | Yes: rulings 105 and 106 |
| 13. An unverified finding is verified first | adopted | `steps_added` | 1 | 0; no finding was marked unverified | Held |
| 14. smith's counting tests | adopted | — | — | not exercised | — |

**The same adoption, beyond these proposals:**
- **The hourly check by a sub-agent:** four checks, $6.41. The 14:40
  and 15:39 checks led to the two steers and the supervisor's handover.
- **A fresh designer past 150K or an hour idle:**
  - two fresh sessions, and one resumed within the hour at 119K;
  - the resumed one grew to 212K and cost about $0.9 more than a fresh
    one would have (inferred).
- **Rulings written once:** all 20, by `plan ruling`.
- **An idle coordinator ends its turn:** eight times, cleanly. But the
  thread then unloads (section 5, item 2).
- **Edits outside Files, in the card:** five, each named in its commit
  body, none a block.
- **Planning in a fresh session:** a sub-agent of the session that also
  designed and started the run. $55; 617K in one context (proposal 7).
- **The pulse's cost by role:** quoted in every hourly check.

**The reliability retrospective's proposals in force here** (skein
479f836 and 7f4b0e8):
- **5, a handover at the next landing past the limit:** five handovers.
  Three were asked of sessions that had just finished (section 5, item
  11).
- **9, each document names its wait:** followed, but Codex's exec cell
  returns at 31 s, so a minute of waiting is two requests.
- **20, the supervisor runs `plan resume` once when the ring finds no
  turn:** it failed each time (section 5, item 2).

**skein e30fe58** (the user's change, the morning of the run): a proof
is a smoke check on top of the tests, and parts connect from the start.
It worked (section 4, item 7).

## 8. Proposals

Each gives a rule in one sentence, its evidence, the document it changes,
and the metric it should move, ordered by expected effect. Proposals 4, 5
and 9 change the kit, and would wait for `kit-after-parity`, as the user
decided for the others; the rest change text only.

1. **The coordinator sets a session to `check` as soon as the last
   increment it needs is running. The supervisor checks it against
   `main` in a background sub-agent while its watch goes on, and, once
   that increment lands, re-checks only the lines it moved.**
   - **Evidence:**
     - 103 minutes waiting for checks, about 70 on the chain.
     - Ruling 119's re-check after a landing took 43 s.
     - jig 06's check waited behind jig 10's while the watch was off
       (section 5, item 1).
   - **Changes:** development.md, 7.2 and 10.5; supervisor.md, 5.
   - **Moves:** `wall_h` and `step_h` down. A session's start after its
     prerequisite lands, a median of about 11 minutes here, down to one
     or two.
2. **An increment whose Needs land before its session's comes first in
   its session, and the session starts on that increment's Needs.**
   This is jig-protocol's proposal 12, narrowed; it was never decided.
   - **Evidence:**
     - 08.2 needed 06.1 and 06.2 (landed 14:48) and started at 16:07,
       second in a session that waited for jig 07.
     - The tail ran one session at a time from 15:06.
     - About 45 minutes (section 5, item 3).
   - **Changes:**
     - development.md, 5.3: the sessions table's "Needs at start" is its
       first increment's;
     - 5.4: the consistency review checks the order.
   - **Moves:** `wall_h` down; `step_h` in the tail down.
3. **A ready check reads every increment of its session against
   `main`, and runs, through `heavy`, every suite its corrections reach,
   the fuzzy one included.**
   - **Evidence:**
     - All three plan-defect blocks came from increments no check read
       (10.3, 07.2, 09.3). jig 10's check had found 10.3's premise wrong
       and stopped at 10.2.
     - Ruling 116's correction to `ops`'s joint world was neither
       applied nor run in the fuzzy suite. The failure came in 08.2's
       gate, ten minutes later (section 5, item 5).
   - **Changes:** supervisor.md, 5.
   - **Moves:** `escalations` (plan defect) from 0.08 a step toward 0;
     `blocked_h` down. A larger check costs more, and proposal 1 takes
     it off the chain.
4. **`plan ring` and `plan resume` load the coordinator's thread with
   `thread/resume` whenever `turn/start` finds it not loaded, and try
   once more.**
   - **Evidence:**
     - Six unloads, 28 supervisor requests, $1.25, and 29 to 80 s on
       each start.
     - `plan resume` skipped the load because `thread/read` answers from
       the rollout (section 5, item 2).
   - **Changes:** `plan`; development.md, 7.4 and 10.5; supervisor.md, 2.
   - **Moves:** the supervisor's tokens down. `restarts` counts a reload
     as one.
5. **At its first call, `plan watch` prints to a new supervisor session
   every open event, whatever `.watch` has recorded:**
   - every step in `check`;
   - every block and question.
   - **Evidence:** 08.2's check waited 20 minutes past the handover, and
     a slot sat idle 17 minutes on the chain. The handover entry expected
     the watch to print it again (section 5, item 4).
   - **Changes:** `plan` (`watch`, `handover`); development.md, 10.5.
   - **Moves:** `wall_h` down; no event waits on a handover.
6. **The implementer reads skein's `programming-model.md` and
   `testing-strategy.md` only by the sections its session file names,
   and after a handover reads its checkpoint and its increment, not the
   start-up set again. It starts a heavy command with the tool's longest
   yield, never one second.**
   - **Evidence:**
     - The start-up reading was 52% of the implementers' input, 14 to
       23K of it these two files and `notes.md`.
     - Successors re-read 64 to 74K.
     - 108 launches with a 1-second yield came back empty: 10.2% of the
       input (section 5, item 6).
     - The parity plan already takes the first rule as its own (the
       reliability retrospective's proposal 10).
   - **Changes:**
     - implementer.md, 8.1 (reading) and 8.3 (yields);
     - development.md, 5.2: a session file names those sections.
   - **Moves:** `tokens_implementers` down from 4.68M a step; `contexts`
     down.
7. **The planner, and the designer of a new design, wait for their
   sub-agents' notifications without polling or sleeping, and hand over
   to a fresh session from their files past 300K.**
   - **Evidence:**
     - The planner ran to 617K in one context, with 216 requests above
       400K.
     - It spent $16.96 at 465 to 567K while its reviewers ran, about 57
       of those requests polls.
     - The designer's 600-second sleep re-wrote 419K.
     - Planning cost $55 against the run's $46 (section 5, item 8).
   - **Changes:** development.md, 2.1 and 2.2.
   - **Moves:** `cost_claude`'s planning part down.
8. **Before `land`, the implementer checks that git reads its tip's
   trailers:** `git log -1 --format='%(trailers:key=Step,valueonly)'`
   prints its step. `land`'s own refusal stays on `kit-after-parity`.
   - **Evidence:**
     - jig-06.1 here, jig-02.1 in test-budgets, jig-10.1 in
       jig-protocol.
     - Ruling 110.
     - A step missing from the metrics, and a lasting `plan status`
       disagreement (section 5, item 7).
     - This is a change to text only, so it need not wait for the kit.
   - **Changes:** implementer.md, 8.4.
   - **Moves:** `proofs` counts every step; process escalations down.
9. **`plan metrics` counts the design's sessions, named by
   `Designer-Session:` trailers on the design's landing commit. It reads
   a Proof's layer to its first hyphen, and counts each time the
   supervisor loads the coordinator's thread by hand as a restart.**
   - **Evidence:**
     - $28.13 of design and orchestration uncounted.
     - 02.4 read as below plan.
     - Six reloads counted as none (section 5, items 8 and 11).
   - **Changes:** numbers.sh; development.md, 4.4 and 15.2.
   - **Moves:** comparability: a plan's whole cost becomes visible,
     design included.
10. **A plan sets per-session test-time shares only where a share is
    larger than one gate's variance on the gate machine (about 1.3 s on
    carbon-forge). Otherwise it states the run's aim alone, which the
    hourly checks follow and "Done when" measures by a median of three.**
    - **Evidence:**
      - Shares of 0.2 to 0.8 s against single-gate swings of 1.3 s.
      - The rule fired on noise at 04.1, unnoticed.
      - The run added 0.69 s in all (section 5, item 9).
    - **Changes:** development.md, 5.6.
    - **Moves:** none directly: `suites` stays under the aim, with fewer
      report lines and checks spent on noise.

## 9. Debts left in the code

For jig's `tech-debt.md`. skein's rows are for the designer.

**jig:**
- **`crates/jig-domain-client/src/pages/shown.rs` (07.2):** its 19-bit
  `section_context` is a field set before each open and reset after it,
  a hidden parameter, and the bit loop is written three times (the
  16:17 check).
- **`tests/client/src/stories/text_pages.rs` (09.3):** its `special`
  helper (46 lines) dispatches five pages on strings, with an
  `unreachable!` arm, where an enum would name them (the 16:17 check).
- **06.3's two tests** carry 73 lines of setup for bodies of about 9, as
  ruling 109 prescribed (the 15:39 check).
- **07.1's 25-line inline `fits` match** would read better as a function
  (the 15:39 check).
- **Files past 800 lines that grew in the run:**
  - `domain.rs` 1,278 (from 1,241);
  - `pages/routing.rs` 1,153 (from 1,111);
  - `sections/project/owner.rs` 1,202;
  - `tests/client/src/harness/world.rs` 948 (from 866);
  - `tests/client/src/stories.rs` 943 (from 880).

  `jig-view/src/tests/work_pages.rs` crossed 800 in the run (842).
- **10.3's terminal story** takes 0.21 to 0.62 s by the machine's load.
- **jig-06.1** (`cdfce095a`) carries no trailers git reads, for good.
- **From `later.md`:**
  - the field names `Part.connector` and `family` carry an owner number
    (decision 5);
  - the composer is in the confirmation's dialog, where `pages.md` 12.2
    lists it inside the activity's section;
  - `Conversations { project }` is ignored when the page opens;
  - the project's overview sets the inbox's filter but asks no read;
  - drawings are acted on in their place only.

**skein, the kit:**
- `plan ring` and `plan resume` with an unloaded thread (proposal 4).
- `plan watch` across a handover (proposal 5).
- **`plan downstream` leaves `wt/temper-downstream`** in the run's
  directory with a regenerated `Cargo.lock`. `plan release` then refuses
  (ruling 120).
- **`plan contexts`** marks a session that has finished for a handover.
- `numbers.sh`'s Proof layer and Step trailer (proposal 9).

**Workspace:**
- temper's pins are 115 commits behind jig's `main`.
- temper fails against jig's `main` with 16 to 18 errors. Nine of them
  are this plan's: the owners' new names and `Act::Drawing`'s arm.
- The user repins after this run (decision 2).

## 10. Decisions

| Proposal | Decision | Where |
|---|---|---|
| 1 | open | |
| 2 | open | |
| 3 | open | |
| 4 | open | |
| 5 | open | |
| 6 | open | |
| 7 | open | |
| 8 | open | |
| 9 | open | |
| 10 | open | |
