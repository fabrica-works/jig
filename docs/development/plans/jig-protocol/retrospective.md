# Retrospective: jig-protocol

2026-10-13. The run of jig's `jig-protocol` on carbon-forge, claimed at
2026-10-12T14:57:02Z and done at 2026-10-13T20:27:04Z: the second run of
development.md, and its first large one. Written by a fresh session that
took no part in the run (development.md, section 15).

Nine read-only sub-agents, one per source, read:
- the plan with `decisions.md`;
- `log.md`;
- `supervision.md`;
- the supervisor's two Claude Code transcripts;
- the designer's 36;
- the git history of skein, smith, jig and temper;
- `logs/heavy.tsv` with the gate logs;
- the coordinator's Codex rollout;
- the 154 implementer rollouts.

Times are UTC. "H n" is n hours after the claim. This document proposes;
the user decides, and the designer applies what is adopted.

## 1. In one page

- **Done in 29.8 hours.** 217 steps landed: 166 planned and 51 added,
  with none dropped. All eight "Done when" items hold on jig's `main` at
  `d91a89c`. Two of them hold only because the run changed what they
  asked (section 2).
- **The code is there.**
  - jig has its protocol layer: the 17 crates and 11 worlds that
    code-organization.md names, and one crate more.
  - One process passed its process proof on the first run: the engine, a
    worker, a smith agent as its child, a store on disk and a native
    client. A live chat was answered on the fifth live run.
  - Across the four repositories: about +51K lines of hand-written
    production code and +77K of tests, net. jig also gained 187K
    generated lines.
  - Real components found real defects, and a small added step fixed
    each:
    - the store's directories never made durable;
    - the cumulative read fence;
    - refused starts;
    - a busy loop;
    - shipped profiles the store's codecs refused;
    - the commit's bound.
- **It cost twice as much per step as tls-cleanup.**
  - The run used about 4.66B input tokens, 21.5M per landed step against
    tls-cleanup's 9.9M. The implementers used 59% of it, the coordinator
    15%, the supervisor 15% and the designer 11%.
  - Three contexts drove the cost:
    - the supervisor ran 9 hours past half its context, and 67% of its
      input came above 500K;
    - the designer's wakes each copied a history that grew to 966K;
    - 70% of implementer threads peaked above 200K, most of them inside
      their first increment.
  - Empty polls took 15.7% of the implementers' input.
  - The supervisor hit the account's monthly and weekly spend limits.
- **The plan was written ahead of its code.**
  - The supervisor made 183 rulings, 0.84 per step. 69 were plan defects.
    About 55 of those would have shown up at planning time, had the
    sketch been compiled, the symbol grepped, the schema generated or the
    check run. But most sessions' sketches named code that earlier
    sessions had not yet landed.
  - The increments that asked about their Size took 2.4 times it.
  - The README's rules grew from 9 to 39. 21 are general enough for
    development.md. The card's limit on edits outside Files was rewritten
    about 20 times.
- **The machine set the pace.**
  - A gate runs alone and pauses the build lane. Build commands waited
    45.7 hours in all, 40.8 of them behind gates. `queue_wait_ratio` was
    2.24, against tls-cleanup's 0.66.
  - jig's focused suite reached 14.3 s of its 15 s budget within two
    hours, and 55 gates failed on its timeout.
  - Two passes on test cost bought about 5 s, and the run spent it again.
  - The user doubled the budgets at H18 (ruling 249). They are still
    doubled on jig's and skein's `main`.
  - carbon-forge's laptop CPU runs the suites 2.5 to 3.5 times slower
    than smith's machine. The code-graph indexer competed with the gates
    all run.
- **Answers came fast, and were checked less and less.**
  - 210 escalations were answered in a median of 1.6 minutes.
  - The designer answered its 36 questions in a median of 3.5 minutes,
    and none of its answers was reversed.
  - 37 of the supervisor's answers were reversed, against 2 in
    tls-cleanup. Near the end of its context, its answers read no code
    at all.
- **The coordinator kept to its card.** It made 3,567 requests and was
  compacted 9 times. As its context grew, the notes it wrote to files
  lost their spaces.
- **The main proposals** (section 8):
  - long-lived roles hand over by size, and start their successors
    themselves;
  - each session's Shapes are checked against `main` when it becomes
    ready;
  - a plan has at most about 60 steps;
  - only the suites run alone;
  - this run's general rules on edits outside Files go into the card;
  - budgets are set on the machine that gates;
  - `plan metrics` counts what it says.

## 2. Outcome

The plan's "Done when", against jig's `origin/main` at `d91a89c`:

| # | Item | Result |
|---|---|---|
| 1 | Every step landed or dropped | Holds: 217 of 217 landed. Two rows are not what "landed" usually means. `temper-repin` landed only temper's STATUS gap (`25399479`, ruling 282), not a repin. `jig-10.1` has its trailers in its body (ruling 105), so `plan status` still says that no commit on `origin/main` names it. |
| 2 | jig's gate passes on `main` | Holds. `2048f39b7`'s Gate-Result reads "scripts/check.sh passed: 3813 tests / 15.041 s; 109 tests / 15.679 s; 42 s in all". The focused suite is past its 15 s budget. It passes because ruling 249 doubled the global timeout to 30 s. |
| 3 | The crates carry their new names | Holds: the grep prints nothing. |
| 4 | jig's worlds run on the real store | Holds: the grep prints nothing. `testing/jig-fake-store` (354 lines) stays for `ops`, as the plan said: `testing/jig-conformance` keeps its `Durable` impl for it (`harness.rs:27`), and `tests/ops` uses it. later.md's `ops` row retires it. |
| 5 | The process proof passes once | Holds: `jig-26.5` `dddb2ead` passed in one run (1 test, 241 ms). `logs/jig-26/proof-one-process.log` records the real binary, the smith agent, the native client, the kill and restart, and the chat with its answer. |
| 6 | The live proof ran once within its budget | Holds only after a change: the proof ran five times. Runs 1 and 2 failed on the fixture: a 69-byte prompt against the native client's 32-byte text limit. Runs 3 and 4 failed because the model answered "No brief provided". Run 5 passed in 1,889 ms with plainer wording. Rulings 279 and 280, and the user's 283, replaced "once". Each run stayed within 5 completions and 5 minutes; the run used 20 of its 40 completions (`sessions/jig-26.md:148`). |
| 7 | No browser or wasm32 entered jig | Holds: the grep prints nothing. |
| 8 | The aims are these items | Holds by its own words. One aim of the README's "Repin" did not happen: temper was not repinned. Its repin needs 288 errors fixed across 23 files, far past what a repin may change, as the plan foresaw (section 5, item 13). |

## 3. Numbers

`plan metrics --history` has one earlier run, tls-cleanup: 13 steps, 3
sessions, 1.3 hours. This run is 17 times its size, so per-step numbers
compare better than totals.

| Metric | jig-protocol | tls-cleanup | Notes |
|---|---|---|---|
| `wall_h`, `running_h` | 29.8, 29.5 | 1.3, 1.3 | No pause, no coordinator restart. The supervisor was stopped twice by spend limits; no metric counts that. |
| `steps_planned`, `steps_added`, `steps_dropped` | 166, 51, 0 | 11, 2, 0 | Added: 21 design follow-ups, 11 defects the worlds found, 10 for test cost, 6 repins, 3 codegen lint fixes. |
| `step_h` | 0.14 | 0.10 | 10.7 landings an hour over the first 15 hours, 3.8 over the last 15. |
| `escalations` | 255, in 21 kinds | 12: infrastructure 7, plan defect 5 | **Not comparable** (below). 210 distinct escalations, 0.97 per step against 0.92. |
| `escalation_wait_min` | 1.6 | 1.3 | p90 4.5. |
| `blocked_h` | 8.2 | 0.2 | 153 episodes, median 1.5 minutes. The longest, 113 minutes, waited for the user. Holds are not counted (below). |
| `answers_reversed` | 38 | 2 | 37 distinct: 0.17 per step, against 0.15. |
| `steers` | 24 sent, 0 points refused | 1 sent, 0 refused | 27 sent, with 32 points (below). The user withdrew one. |
| `restarts` | none | none | |
| `tokens_coordinator` | 699M in (99% cached), 1.5M out | 15.9M in, 55K out | 3.2M per step, against 1.2M. |
| `tokens_implementers` | 2,761M in (98% cached), 9.8M out | 43.2M in, 175K out | 12.7M per step, against 3.3M. 154 threads. |
| `tokens_claude` | 5,161M in (98% cached), 10.1M out: 0 before the claim, 5,153M in the run, 8.3M after | 227.6M in, 1.09M out | **Wrong:** about 4.3 times too high (below). Each message counted once gives 1,197M in all, 1,184M of it in the run: the supervisor 681M and the designer 514M. |
| `tokens_per_step` | 39.8M | 22.2M | With Claude Code's tokens counted once, 21.5M, against tls-cleanup's 9.9M for its run alone. |
| `coordinator_requests` | 3,567, 67 after a wait ran out | 147, 4 | 89% followed a report or a ring, a median of 4 to 5 requests each. 205 followed 60-second sleeps while it held for the user. |
| `compactions` | 9 | 0 | Each at 300K to 400K tokens, taking 1.5 to 4 minutes. |
| `gates_per_step` | 2.2 | 2.5 | 415 land gates, of which 134 failed (32%). 51 gates were kept across docs-only commits. 58 passing gates were thrown away when a code landing moved `main`. |
| `queue_wait_ratio` | 2.24 | 0.66 | Gate lane 0.51, build lane 4.68. |
| `suites` | first (skein's): 1963 tests / 9.911 s; 98 / 43.341 s; 6 / 3.402 s. Last: 3813 / 15.041 s; 109 / 15.679 s | 1912 → 1940 tests, 9.99 → 5.00 s | **Not comparable:** "first" is skein's gate, "last" is jig's. jig's first was 2,173 tests in 9.128 s and 86 in 12.054 s. Its last ran under a 30 s timeout. |
| `proofs` | unit 92, world 71, joint 15, process 1, live 1; 2 below plan | unit 2, world 5, joint 2, process 3, live 1; 0 below | No proof was below plan: both are parsing artefacts (below). 42 Proof lines start with a word that is not a layer ("gate" 31 times). |
| `report_lines` | 10 median, 11 p90 | 10, 10 | Reports were short in lines but not in characters: one line runs to 4,305 characters. |
| `lines` | code +355,989 −129,879; tests +114,950 −18,650 | code +1,445 −1,461; tests +2,041 −1,731 | **Not comparable:** 82% of the code additions are generated. Hand-written: production +59.2K −7.9K, tests about +95K −18K. |
| `pin_lag` | smith: skein 8; jig: skein 1, smith 0; temper: skein 250, smith 718, jig 1,439 | — | Read from stale local checkouts (below). |
| `contexts` | sessions' peaks: 238K median, 319K p90; 70 handovers | not measured (peaks of 255K and 315K) | 108 of 154 threads peaked above 200K. |

**Not comparable, and why:**

- **`tokens_claude`** (and so `tokens_per_step`):
  - **Forked sessions count several times.** `plan wake` resumes a Claude
    Code session under a new id, and the new transcript copies the whole
    earlier history. `claude_tokens` deduplicates message ids within one
    session but not across sessions, so each of the designer's 35 wakes
    counted everything before it again. Counted once per message across
    `claude.tsv`, the run used 1,184M, not 5,153M.
  - **Planning is not counted.** "Before the claim" is 0 because the
    planner's session is not in `claude.tsv`, and it was not found on this
    machine.
- **`escalations`:**
  - **An escalation counts once per section that lists it.** It stays
    listed in every "Blocked and unblocked" section until it is answered:
    jig 22.3 is listed 7 times.
  - **The Kind field is free text.** The coordinator wrote 20 different
    Kind strings, where section 10.2 allows 6 ("size exception",
    "own-session outside-Files scope >20lines", and so on).
  - By content, the 210 distinct escalations were:

    | Kind | Count |
    |---|---|
    | plan defect | 65 |
    | scope or outside Files | 45 |
    | design | 32 |
    | size | 17 |
    | test cost | 17 |
    | infrastructure | 16 |
    | process | 11 |
    | live proof | 5 |
    | temper | 2 |
- **`answers_reversed`** counts one unblock twice: it is quoted inside a
  later blocked report.
- **`steers`** misses the three steers whose heading the coordinator
  wrote as "## Steer," instead of "## Steered,".
- **`blocked_h`** leaves out holds. An answer that says "hold" closes the
  escalation, so these stalls count for nothing:
  - smith-repin, held for the user by ruling 186 for 7.1 hours, from 01:09
    to 08:16;
  - jig 30.1 and 19.1, held for about 1.5 hours;
  - the 20:32 hold of the whole jig line.
- **`proofs`:**
  - **`planned_layer` finds no session numbered below 10.** It looks for
    the heading "6.1" where session files write "06.1", so 86 steps (skein,
    smith and jig 01 to 09) were never compared with their plan.
  - **Both "below plan" proofs are parsing artefacts.** jig-25.4's Proof
    begins "unit;", which the metric does not read as "unit". jig-30.s3's
    landed tip is ruling 249's timeout commit, whose Proof is "gate"; its
    work commit, `0809a04b7`, says "unit", as planned.
  - **jig-10.1 is missing** from the counts, because git cannot parse its
    trailers.
- **`suites`** takes its first and last gates from different
  repositories.
- **`lines`** counts generated code and golden files as code, and some
  generated test data as tests.
- **`pin_lag`** reads the local checkouts, which were stale. smith's local
  `main` is 38 commits behind `origin/main`. On `origin/main`, jig is now
  37 commits behind smith, and smith pins an *older* skein than
  smith-repin landed (section 5, item 13).

**What explains the largest differences:**

- **Tokens per step, 2.2 times tls-cleanup's.** Most of the difference is
  context carried on every request:
  - **The supervisor:** 550 of its second session's 1,223 requests ran
    above 500K, and used 381M of that session's 565M.
  - **The designer:** about 75% to 80% of its 514M re-sent earlier wakes'
    history.
  - **The implementers:** their mean request carried 160K. 433M (15.7%)
    went on requests that only saw an empty poll. About 5.8% went on the
    code graph's `list_projects`, which prints every project on the
    machine (8.5K tokens a call, 536 calls).
- **`queue_wait_ratio`, 3.4 times tls-cleanup's.** Up to eight sessions at
  once (a mean of 7.5 to 7.9 in the busiest hours, against tls-cleanup's
  three) shared one exclusive gate lane, and the build lane stops while a
  gate runs. For 76% of the run no build command was running. For 11.0
  hours, nothing ran in the queue at all.
- **`answers_reversed`: about the same per step, but 37 in all.** Of
  these:
  - 10 were the supervisor's own errors;
  - 10 came from a plan sketch or premise nobody had checked;
  - 10 widened a limit that had proved too narrow;
  - 5 were interim rulings replaced by the designer's answer;
  - 2 were the live proof.

  The share of the supervisor's answers that read any code fell as its
  context grew (section 5, item 2).
- **Escalations: about the same per step, from different sources.** In
  tls-cleanup they came from the machine and the plan's commands. Here
  they came from:
  - plans for code that did not exist yet;
  - the card's limits;
  - Size;
  - 32 real design questions, where tls-cleanup had none.
- **`step_h`, 1.4 times tls-cleanup's.** The first 15 hours ran 8 sessions
  at once and were bound by the gate lane. The last 15 ran 1 to 4 sessions
  along the plan's longest chain.

## 4. What worked

- **The plan was delivered without dropping a step.**
  - All 217 steps landed.
  - The 17 crates and 11 worlds of code-organization.md, section 2, are
    on `main`, with one crate more (`jig-test-client-records`, ruling
    219).
  - The code checks held:
    - no new file is over 800 lines;
    - no Pulse word was added;
    - there is no `todo!`, TODO or FIXME;
    - there is no `unsafe` in jig.
  - Test functions over 40 lines fell from 622 to 610, while tests grew
    from 2,250 to 3,802.
- **Real components found real defects, early enough to fix in the run.**
  Session 18's real store and session 27's real smith agent found:
  - **the store's directories never made durable.** skein's io now makes
    durable what it creates by a path (ruling 250, `63e81de`; steps 18.s1
    and 18.s2). The store world had started with its directories already
    made, which had hidden this.
  - **the cumulative read fence** (27.s1);
  - **refused starts acknowledged** (ruling 257);
  - **a busy loop** (27.s2);
  - **shipped profiles that the store's codecs refuse** (27.3);
  - **the commit's bound** (19.s1 to 19.s3).

  Each fix was a small added step, and the designer answered the design
  part within minutes.
- **The escalation channel held at 17 times the scale.**
  - 210 escalations were answered in a median of 1.6 minutes (p90 4.5).
  - **The coordinator reacted fast:** a median of 4 s from a doorbell to
    its first action, and 13 s from a report.
  - **The designer:**
    - answered 36 questions in a median of 3.5 minutes to `answer.md`, and
      2.3 minutes to the landing;
    - landed 39 design commits (jig 29, skein 8, smith 2);
    - had none of its answers reversed;
    - never watched the run: tls-cleanup's proposal 12 held.
- **The coordinator kept to its card at scale.**
  - None of its 2,710 shell commands ran cargo, `heavy`, `land`, the gate
    or `git commit`.
  - All 279 ancestry checks were written as `--is-ancestor … && echo
    on-main`, and printed their result. It ran 278 Step-trailer checks.
  - It spawned 154 fresh threads, one per session or handover. One spawn
    was refused, and the retry 22 s later succeeded.
  - Each of the 51 added steps traces to an unblock or a steer. It
    invented nothing.
  - `done.md` maps all eight "Done when" items. It removed 59 worktrees,
    after saving the two that held work as patches.
- **A rule, once made, held.**
  - **Clippy until it exits 0** (steer of 15:25:42): 4 of the first 15
    gates failed on lints, and none of the 400 or so after the steer.
  - **The Step-trailer check** (ruling 105) ran before all 368 later
    `land` attempts, and the anomaly did not recur.
  - **One test per behaviour** (ruling 219): new tests per step fell from
    a median of 6 to 3.
  - **Labelled table rows** (the steer logged at 07:28): no `case_0` …
    `case_N` after it.
- **Keeping a gate across Markdown made 246 docs landings affordable.**
  - 51 of jig's 195 code gates (26%) were kept across commits that changed
    only Markdown under `docs/`. That saved about 96 minutes of exclusive
    gate time.
  - Without it, each of the supervisor's and the designer's 246
    Markdown-only landings (8.3 an hour) would have thrown away the gates
    running at the time.
- **Test cost became design work at the user's word.**
  - At 22:12 on 10-12 the user asked the designer to make the tests
    cheaper rather than raise the timeout.
  - The analysis was ready in 22 minutes: about 35 s of the suite's test
    time was process start-up.
  - The suite had recovered 2 hours 10 minutes after the question, with
    six steps. Among them:
    - `jig-repin.s3` removed 1,388 generated tests;
    - jig-view went from 16.5 s to 1.7 s.
  - The rule "a new test costs the suite about 12 ms before its work" went
    into skein's testing-strategy.md (`cb3cefb`).
- **The hourly check ran every hour and caught what its numbers show.**
  29 checks, with no gap over 90 minutes. They found:
  - the clippy loop (15:22);
  - disk falling 35 GB an hour (17:22; 18.2 GB freed);
  - `logs/` at 661 MB (23:22);
  - near-copies of the wire's test fixtures across two crates (04:23,
    which led to ruling 219's shared kit);
  - unlabelled table rows (07:4x);
  - the focused suite at four fifths of its budget (05:26).
- **The first supervisor handed over on its own.**
  - At 549K it wrote its handover and started its successor itself, with
    `claude --bg` (19:44).
  - The new session had the watch running 32 s after joining, and
    answered its first block 8 minutes later.
- **The live budget and the borrowed login were respected.**
  - 20 of 40 completions were used, at most 5 a run.
  - The token file was deleted after each run, and nothing was written to
    `~/.codex`.
- **Implementers kept working while they waited.**
  - 80% of their `send_message` questions were followed by more work
    before the answer came.
  - They resolved about 35 rebase conflicts without escalating any.

## 5. What did not

Each item gives its cause, when it was first visible, and when it was
acted on. They are ordered by cost.

1. **The supervisor kept a context it should have handed over.**
   - **How it went.** The second session (S2) passed 500K at 22:35:34 on
     10-12. At 22:39:59 it wrote a handover "to a fresh session, once the
     user starts it", and told the user. Nobody started one.
     - It wrote 84 times that it was "covering the run until the fresh
       supervisor joins".
     - Its context was compacted at 06:08:49 on 10-13, from 969K to 6K.
     - It passed 500K again at 11:39, and ran for 9 more hours at 528K to
       696K without mentioning it.
   - **Cost.** 550 of S2's 1,223 requests ran above 500K and used 381M of
     its 565M input. Handing over at each 500K would have cost about 319M
     (−44%); at each 300K, about 62% less.
   - **Its checking fell as its context grew.** The share of answers that
     read any code:

     | S2's context | Answers that read code |
     |---|---|
     | under 200K | 50% |
     | 400K to 800K | 7% to 14% |
     | over 800K | 0% |

     127 of S2's 155 answers read no code at all.
   - **After the compaction, the record suffered:**
     - its entries carried guessed times ("2xZ");
     - its steers carried times 1 to 22 minutes ahead of the
       coordinator's headings;
     - its checks dropped the contexts line;
     - three checks (09:3x to 11:3x) read the suite against 15 s, after
       ruling 249 had doubled the budget.
   - **Cause.** Section 2.5 says the supervisor "hands over to a fresh
     session", but not who starts it. S1 started S2 itself; S2 left it to
     the user, who was not watching.
   - **First visible:** 22:35 on 10-12. **Acted on:** never.
2. **37 of the supervisor's answers were reversed.**
   - **The supervisor's own errors: 10.** For example:
     - ruling 258 named the tasks domain's Files, but the core holds the
       read. Ruling 260 reversed it 6 minutes later.
     - ruling 265 named an input that does not exist (266, 2 minutes
       later).
     - for ruling 192, a `grep … | head -14` cut off an existing variant
       (193).
   - **A plan sketch or premise nobody had checked: 10.**
   - **A limit widened again: 10.**
   - **Interim rulings replaced by the designer's answer: 5.** One cost a
     step: ruling 160 ruled ahead of the designer, and jig 21.2 landed on
     it 50 s before the answer reversed it, so step 21.s1 was added.
   - **The live proof: 2.**
   - **Cause.** Section 4.5 says "Check an answer against the code before
     sending it", and tls-cleanup's proposal 11 asked for numbers to be
     checked. The checks were skipped, more often the larger the context
     (item 1). Each reversal cost a round trip, and some cost a step.
   - **First visible:** 17:54 on 10-12 (118 reversing 115). **Acted on:**
     11:14 on 10-13. After 260 the supervisor saved a memory rule, "grep
     main for every symbol a new step names". This retrospective found no
     reversal for that cause after it.
3. **The designer's wakes carried its whole history.**
   - **How it grew.** Each `plan wake` resumed the session, under a new
     id, with every earlier message. Its context at the start of each
     wake:

     | Time | Context |
     |---|---|
     | 14:57 on 10-12 | 44K |
     | 22:13 | 485K |
     | 07:24 on 10-13 | 896K |
     | 07:52 | compacted automatically, from 968K to 7K |
     | 16:37 | back to 699K |
   - **Cost.** 514M input tokens for 36 questions: a median of 10.6M for
     each wake that answered. About 75% to 80% of that re-sent history.
     - A wake at 800K reads 8 times what a wake at 100K reads.
     - Ten wakes came after the cache had expired, and wrote the whole
       history to the cache again.
   - **A fork lost history.** At 22:30, a wake sent while the designer was
     busy forked a second designer. `claude.tsv` recorded the fork, so
     every later wake resumed from it, without the first test-cost answer
     and the 20.5 answer in its history. From 22:52 to 23:15 the
     supervisor told the working copy to stand down. Questions 22.3 and
     07.s1 waited 29 to 32 minutes.
   - **Cause.** tls-cleanup's proposal 12 stopped the watching, and there
     was none in this run, but nothing bounds a wake's context.
   - **First visible:** the third wake (144K at 17:01). **Acted on:**
     never.
4. **The plan was written ahead of its code.**
   - **69 plan-defect rulings:**

     | Defect | Rulings |
     |---|---|
     | Shape against landed code | 17 |
     | Size | 15 |
     | Shape against the design | 14 |
     | a code-audit miss | 7 |
     | order | 5 |
     | Files missing users | 5 |
     | a check or command | 4 |
     | the plan against itself | 2 |
   - **About 55 were visible to section 5.4's checks:**
     - **compiling the sketch or grepping the symbol: 22.** Ruling 189:
       "on `main` has no component `Limits`". Ruling 180: `JournalLimits`
       "has no such field".
     - **generating the schema: 3.** Ruling 172's sketch came to "about
       550 GB at the ceilings"; ruling 151's refusals reached 99 types that
       nest themselves.
     - **running the check: 3.** Ruling 150's Done-when regex matched a
       `pub(crate) struct Facts` that was already on `main` when the plan
       landed (`dded87e6c`).
     - **reading for consistency: about 17.**
   - Decision 44 says so: "The plan's commands were not all run before it
     landed". The seven sessions whose sketches were compiled (skein 01 to
     03, smith 01, jig 01, 02 and 05) had 3 Shape defects among them.
   - **Cause.** The plan has 38 session files, and most name code that
     earlier sessions land. Their sketches cannot compile at the plan's
     base, so section 5.4's check cannot run for them before the run.
     Nothing in the process runs it when they become ready.
   - **First visible:** from H0. The supervisor's 19:44 handover named it:
     "a plan written before most of its code", with about 30 blocks over
     Files and Size. **Acted on:** one block at a time.
5. **Sizes were less than half of what the work took.**
   - **The numbers.** The 17 increments that asked about Size landed
     18,309 lines against 7,630 planned: 2.4 times, with a median of 2.1.
     The worst was 32.1, at 2,961 lines against 600.
   - **Why the sizes were low,** in the rulings' words: exhaustive
     translations were "priced at about half what they take" (130), and
     composition "runs about twice its Size" (252).
   - **18 rulings moved the margin:**
     1. 1.2 times (103);
     2. 2 times for codecs (132, 151, 188, 214);
     3. the client wire "at their count" (194);
     4. 1.5 times for everything, at 02:53 on 10-13 (196);
     5. 2 times for jig 25 (252).
   - Large increments made large contexts (item 7).
   - **First visible:** H1 (ruling 103). **Acted on:** with a general rule
     at H12 (196).
6. **The card's limits on edits outside Files did not fit sessions that
   share code.**
   - **The mismatch.** Section 8.2 lets an increment edit its Files and its
     own session's files, and section 8.5 makes a gate failure outside
     Files a question. Here 41 sessions shared test worlds, exhaustive
     matches, fixtures and expected values.
   - **The volume.** About 45 scope escalations and 34 process rulings.
     About 20 rulings widened the rule, from 104 to 262, over 19.5 hours.
     Ruling 112 calls itself "the fourth block in two hours", ruling 114
     "the sixth block in three hours".
   - **The README.** Its rules grew from 9 to 39, and at the end two were
     out of date:
     - "A generated test's ceiling is 200 ms" still stands, although
       ruling 169 ended it;
     - rulings 140 and 249 give two answers to the same timeout.
   - **Questions already answered.** About 23% of the decisions cite a rule
     that already existed ("no question").
   - **First visible:** 15:xx on 10-12 (104). **Acted on:** rule by rule,
     until 11:37 on 10-13 (262).
7. **Implementers' contexts passed 200K inside their first increment.**
   - **How a thread grows:**

     | Moment | Context |
     |---|---|
     | a fresh thread | 18K |
     | after the setup reads | 53K |
     | after reading code, at the first edit | 105K |
     | at the first landing (median) | 163K |
     | each later increment adds (median) | 52K |
   - **The peaks.** 108 of 154 threads peaked above 200K, 62 of them
     before landing anything. Codex compacted 35 times, at a median of
     317K.
   - **Why the handover rule did not help.** It comes between increments,
     so it could not help these threads. Threads landed 1.42 steps each,
     against the 4 or 5 that section 5.3 assumes.
   - **Successors cost as much as fresh threads.** A successor had 106K of
     context at its first edit, the same as a fresh thread. Its
     checkpoint repeated the standing pass-ons (7 of jig-25.md's 15 lines)
     more than the files and the traps.
   - **First visible:** H0 (skein 02 handed over at 213K, at 15:28).
     **Acted on:** with handovers only.
8. **Implementers polled, and listed the code graph's projects.**
   - **The cost.** Requests that saw only an empty poll carried 433M input
     tokens, 15.7% of the implementers' input. The median thread lost
     16.3% this way, the worst 46%.
   - **How they waited:**
     - 2,623 waits on running commands, only 23 of them with a 300 s wait;
     - 4,866 `write_stdin` calls with empty input;
     - `ps` 129 times and `pueue status` 70 times.
   - **Cause.** Section 8.3 says "the longest wait your tool allows", but
     does not name the call, and Codex's own instructions say to avoid
     waits over 60 seconds. Section 7.4 overrides that guidance for the
     coordinator only. A Codex command cell also yields after about 30 s,
     whatever is asked.
   - **The code graph.** `list_projects` printed every project on the
     machine: 536 calls, 5.8% of the implementers' input. All the graph's
     output was 14.5%. The coordinator carried 24K of the graph's listings
     in 620 requests.
   - **First visible:** no metric or pulse line counts polls. **Acted on:**
     never.
9. **The gate lane was the run's bottleneck.**
   - **How busy it was.** A gate runs alone and pauses the build lane.
     With up to eight sessions running, the gate lane was busy 74% to 84%
     of the first ten hours. Build commands waited 45.7 hours, 40.8 of them
     behind a gate.
   - **What a gate spent its time on.** A passing gate ran alone for a
     median of 98 s: 65 s of formatting, clippy and building, and 24.5 s
     of suites.
   - **Failed gates.** 134 of 415 land gates failed, using 3.05 hours of
     the lane:

     | Cause | Gates |
     |---|---|
     | timeouts | 55 |
     | assertions | 38 |
     | jig-boundary guard tests, which are cheap and a session could run before `land` | 19 |
     | a test's wall-clock bound | 7 |
     | build errors | 10 |
     | clippy | 4 |
     | formatting | 1 |
   - **Thrown away.** 58 passing gates were discarded when a code landing
     moved `main`: 98 minutes.
   - **What a change would save.** Replaying the run's queue: if only the
     suites ran alone, build waits would fall from 45.7 hours to about 6.8.
   - **First visible:** at the 16:22 check (82% busy). The supervisor put
     three kit changes to the user and to this retrospective: rebase
     inside the gate task, compile in the build lane, and a timeout
     margin. **Acted on:** not made, because the kit is shared with
     smith's reliability run.
10. **The focused suite outgrew its budget on this machine.**
    - **The timeouts.** jig's focused suite took 14.3 s by 17:00 (H2). The
      first timeouts came at 17:58 (smith) and 20:18 (jig). There were 55
      in all, 17 of them in the 22:00 hour.
    - **The rules loosened first:**
      - ruling 140 (20:22): three attempts;
      - ruling 153: the drift tests' ceilings;
      - ruling 165: 200 ms for generated tests.
    - **The first test-cost pass.** At 22:12 the user wrote: "please
      escalate to the designer, most likely there are inefficiencies in
      the tests themselves". Ruling 169 followed at 22:35, and the suite
      had recovered by 00:20.
    - **The second pass** (ruling 219, 05:35).
    - **What the passes bought.** About 5 s. The median focused time was
      9.75 s over the first 20 landings and 14.9 s over the last 20.
    - **The doubling.** At 08:42 the user wrote: "If the budget starts
      biting, we will double the budget temporarily". It bit at 09:08, and
      the budget was doubled at 09:11 (ruling 249). The last gate took
      15.041 s.
    - **Still doubled.** The timeout is still 30 s on jig's and skein's
      `main`. skein's commit (`886a7d4`) does not name the plan's end, and
      rewrote its header to say "within 30 seconds".
    - **The machine:**
      - carbon-forge's i5-8265U laptop CPU runs the suites 2.5 to 3.5
        times slower than smith's machine. This was found at 08:3x on
        10-13, at H17.5.
      - the code-graph indexer was the busiest process at 183 of 488 gate
        starts, using up to 778% CPU. It was reniced at 20:35 (ruling
        144), was back at 590% at 08:50, and the user had it wrapped at
        09:2x.
      - `plan check` measured nothing on this machine before the run.
    - **First visible:** 17:00 on 10-12. **Acted on:** 20:22 (retries),
      22:12 (the user), 09:11 on 10-13 (the doubling): 16 hours from
      first visible to the doubling.
11. **The coordinator's context and its record.**
    - **Requests and compactions.** It made 3,567 requests, a median of 4
      to 5 for each wake-up. It was compacted 9 times, at 300K to 400K,
      with 23 minutes of dead time in all.
    - **Text without spaces.** The text it wrote to files lost its spaces:
      15% of characters were spaces up to 21:00, and 3.5% to 8% from 22:42
      on 10-12, the first writes after compaction 3. "all217authorizedsteps
      are closed" in `context.md` is one example. The same style is in its
      ledger notes, in one message to an implementer (which was resent),
      and in implementers' checkpoints. It correlates with context size;
      `context.md` had reached its 60-line cap at 26 KB.
    - **Retyped reports.** It retyped 497 reports as string literals (641K
      characters) to append them to `log.md`. 147 of 600 report headings
      share a time stamp, because they were logged in batches.
    - **Holding for the user.** From 17:28 to 20:13 on 10-13 it slept 60 s
      at a time, 103 times, as section 7.4 says. That made 205 requests
      and used 52M tokens while nothing ran.
12. **The tail ran one session at a time.**
    - **The pace.** Over the last 15 hours the run landed 3.8 steps an
      hour, against 10.7 over the first 15. From 10:00 on 10-13, the mean
      was 1.7 to 2.4 sessions running.
    - **Why.** The chain 31 → 33 → 26 was serial. Steps whose own Needs
      had landed waited for their session's earlier steps:
      - the 10:25 steer could start only 31.5;
      - 26.5 waited 30 minutes behind 26.2's block, until the steer
        logged at 16:27.
    - **Cause.** The ledger and the card track readiness by session, not
      by step.
13. **Downstream breakage was found late, and one repin has been undone.**
    - **`plan downstream` never ran.** Section 14.1 asks each check to run
      it once the plan has landed upstream code; it was run 0 times.
    - **The breakage surfaced only at the repins:**
      - **smith, 23:09 on 10-12:** 510 lint errors from the tests that
        skein-02.s4 generated.
      - **smith, 23:48:** its settle test took 2.05 s against its 2 s.
        This plan's base64 decoder costs about 200 ms in every smith
        process, and smith's test person reads 24,940 bytes one at a time.
        Ruling 186 held smith-repin for the user from 01:09 to 08:16 (7.1
        hours). The user's question at 07:14 was "Please elaborate on
        ruling 186, I didn't get it". smith-repin landed at 08:56.
      - **temper, 19:35 on 10-13:** 288 errors in 23 files. Ruling 282
        recorded the gap.
    - **After the run, smith-repin was undone.** smith's `83122ea0`, from
      the reliability plan on the other machine at 19:35, moved smith's
      skein pin from `8a34235` (smith-repin's) back to the older
      `cb3cefb`, calling it "current upstream main". smith's `main` no
      longer has smith-repin's skein, and skein-02.s8's fix is out of its
      lock.
14. **"Once" was the wrong plan for a live model.**
    - **Runs 1 and 2** failed on the fixture, which an offline run would
      have caught: a 69-byte prompt and charter against the 32-byte text
      limit. Run 2's answer check also matched the echo of the request.
    - **Runs 3 and 4** failed because the model answered "No brief
      provided", although the brief was there.
    - **The wait.** The step waited for the user from 18:21 to 20:13, 113
      minutes and the run's longest block, with no push notification. The
      whole run sent 3 push notifications.
15. **Spend limits stopped the supervisor.**
    - The monthly limit stopped it from 17:24 to 17:39 on 10-13. The weekly
      limit stopped it at 20:23, until the user exited and resumed it at
      20:44.
    - The user had written "I'm low on credit" at 08:36.
    - No pulse line shows the tokens spent.
16. **The record.**
    - **`plan metrics`** is wrong or not comparable in places (section 3).
    - **STATUS rows.** smith's `STATUS.md` still says jig-protocol is
      "active on carbon-forge". Only jig's (`d91a89c`) and skein's
      (`2068257`) rows were marked done.
    - **jig-10.1** is still flagged by `plan status`. Four steps landed
      twice (jig 30.2, 23.2, 35.2 and 32.2), and the ledger names only the
      second commit of each.
    - **The kit:**
      - `plan set --help` added a ledger row named `--help`;
      - `plan watch` exits 1 when `blocked.md` holds only its title, which
        happened twice;
      - `plan pulse` timed out at 280 s four times, and after the doubling
        read the suite against 15 s ("99%");
      - five pueue tasks lost their `heavy` and have no `heavy.tsv` line,
        and two passing gates were lost with them.
    - **Left in `logs/` (703 MB):**
      - 415 MB in 10 logs of smith's settle failures;
      - two ELF binaries (33 MB);
      - a scratch crate with its `target/` (17 MB);
      - two copies of generated code (8.4 MB), from before the 23:22 steer.
    - **kache's store** holds 92.2 GiB against its 74.5 GiB cap.

## 6. Decisions taken during the run

183 rulings, 101 to 283. `done.md` gives each one's design commit, as
section 7.6 asks. Where each belongs:

| Rulings | What | In the design | Proposal |
|---|---|---|---|
| 37/162, 170, 176, 183/185, 187, 190, 202, 207/209/210, 224, 228/234, 244, 250, 256, 257, 264, 269/273, 275, 276, 277, 278 | The designer's answers: grants, chats, links, walls, undecodable turns, first start and first period, UTF-8 texts, a service's credential, an activity page's turns, durability by path, own watch subjects, refused starts, sign-ins while stopping, bounded pieces, the world's power loss | Yes: 39 design commits. `done.md`'s table names them, for example jig `87195d801`, `e484056a1` and `2fa034fbd`, and skein `63e81de`, `b638df3` and `2f89cf7` | — |
| 180, 201, 200, 203–206, 215, 217/218 | StoredTurn's 8 MiB bound; the clamp of smith's internal ceilings; typed links and staged activation; the caller's connector room as a sum; an unadmitted Busy; the map's sizes without transcripts; the keeper's versions | **Not in the design yet** (`done.md`) | For the designer |
| 138, 149, 151, 152, 154–156, 175, 182/184 | Codegen: boxed variants, minima, recursive facts, sums as pieces | skein `482691a`, `58cc905`; jig `3c5f607c6` | — |
| 108, 112, 114, 116, 122–125, 129, 145, 195, 197, 198, 209, 225, 227, 230, 247, 262 | Edits outside Files, expected values, removed mechanisms, private fields, files that moved, fixtures, visibility, version sweeps, harnesses | The plan's README only | 5 |
| 103, 110, 130, 132, 151, 188, 194, 196, 214, 252, 271 | Size: what it counts, and its margin | The README and code-organization.md | 6 |
| 121, 140, 153, 165, 167–169, 173, 174, 179, 186, 219, 242, 248, 249, 253, 268 | Test cost and budgets | 219 in skein's testing-strategy.md, 8 (`cb3cefb`). 249 is the user's, and still in force on `main` | 7, section 9 |
| 101, 120, 144 | Infrastructure: a primary checkout fast-forwarded; the indexer at idle priority | Not in development.md | 7, 17 |
| 105 | jig-10.1's trailers accepted as they are | — | 17 |
| 279, 280, 283 | The live proof's runs | This plan only | 14 |
| 254, 281, 282 | jig 25.6's pins; temper's gap | temper's STATUS.md, section 3; later.md | 12, 13 |
| The other plan-defect rulings, 69 in all | Shapes, Files, order, checks | The session files, amended in the run | 2, 3 |
| 118, 142, 147, 158, 163, 191, 193, 205, 231, 232, 243, 246, 260, 266, 270, 272 | Corrections of the run's own rulings and added steps | — | 1, 2 |

**Decided by the user during the run** (decisions.md, "For the user"):
- not yet running each crate's tests in one process;
- keeping optimized tool builds, since the build cache absorbs them;
- accepting the real store's test cost;
- measuring budgets on the slowest machine;
- doubling a budget that bites until the plan ends (249);
- moving smith's byte-at-a-time reader into a skein plan of its own,
  option (a), not started;
- more live runs (283).

**Still with the user:**
- the test-cost pass after the plan, which ruling 249's doubling waits on;
- a turn too large for a page;
- the gate lane's three kit changes;
- signing out on a shared machine;
- retention;
- the browser and wasm32;
- live text from workers.

## 7. The proposals adopted after the last retrospective

tls-cleanup's eighteen proposals were adopted on 2026-10-12 (skein
`a01c2cf`, 08:03Z), with three more changes that morning: a repin phase
(`49392c3`), a handover past 200K (`d89fd56`), and a fresh thread per
session (`582a506`, `02bfa45`). This run, claimed at 14:57Z, was the first
under them.

| Proposal | Meant to move | tls-cleanup | jig-protocol | Moved |
|---|---|---|---|---|
| 1. `plan check` runs heavy and the gate | `escalations` (infrastructure) | 7 of 12 (0.54 a step) | 8 of 210 (0.04 a step) | Yes. The 8 were load during the run, not setup. It measured nothing about the gate's time on this machine (section 5, item 10). |
| 2. The planner runs the commands and compiles the sketches | `escalations` (plan defect) | 5 (0.38 a step) | 69 rulings (0.32 a step) | Little. Done for seven sessions, which had 3 defects; the others could not compile at the plan's base (section 5, item 4). |
| 3, 4. Small edits beyond Files; boundary types (8.2) | `escalations` | 6 such questions | about 45 scope escalations and 34 process rulings | No. The rule is written for one session's own files (section 5, item 6). |
| 5. A timeout retried once | `escalations` (infrastructure), `blocked_h` | 2 timeouts | 55; ruling 140 raised it to three attempts, then 174 and 249 | Held as written. The cause was the suite's size, which no retry fixes. |
| 6. clippy before `land` | `gates_per_step`, `queue_wait_ratio` | 5 of 9 failed gates; 2.5 | 4 of 134, all before the 15:25 steer made it "until it exits 0"; 2.2 | Yes, once "once" became "until it exits 0". |
| 7. A gate kept across `docs/` Markdown | `gates_per_step`, `queue_wait_ratio` | 6 of 9 thrown away after Markdown | 51 kept (96 minutes); 58 thrown away after code landings | Yes, for what it targets. `queue_wait_ratio` rose for other reasons. |
| 8. The pulse prints rising words | `steps_added` for words | 1 step | none; "legacy" rose and was explained | Held. |
| 9. A removed word names its replacement | `steps_added` | 1 step | none needed | Not tested. |
| 10. Each aim is a "Done when" item | `lines` toward the aim | +270 against −500 | 8 items; item 6's "once" did not survive a live model | Partly. |
| 11. Answers keep the plan's checks, and their numbers are checked | `answers_reversed` | 2 (0.15 a step) | 37 (0.17 a step) | No (section 5, items 1 and 2). |
| 12. The designer does not watch | `tokens_claude` | 49.3M watching, no question | no watching; 514M over 36 questions | The watching stopped, but the cost moved to the wakes (section 5, item 3). |
| 13. One wait, no polls | `tokens_implementers` | up to 43% of a session | 15.7% overall, 46% at most | Little (section 5, item 8). |
| 14. Every time read from `date -u` | — | 2 to 12 minutes ahead | 1 to 22 minutes ahead after the compaction; "2xZ"; batched headings | No. |
| 15. A ruling that drops part of the gate reaches the user | gates without a suite | 13 | ruling 173 was pushed at 22:49; 249 was the user's own | Held. |
| 16. heavy records the load at a gate's start | `suites` comparable | — | recorded at 488 starts; the indexer was the busiest process at 183 | Partly. The cause was found from a block (20:35), not from the record, and no pulse line reads it. |
| 17. `tokens_claude` in three parts | comparability | — | split, but planning reads 0 and forks count 4.3 times | Not yet. |
| 18. The kit keeps the record true | — | — | ancestry checks print (279 of 279); `plan retro` found the released run; heavy still leaves no line when killed (5 tasks) | Partly. |
| The repin phase | `pin_lag` | jig 338 behind smith | jig 0 behind smith and 1 behind skein at the end; temper's gap recorded; smith-repin undone after the run | Partly. `plan downstream` never ran. |
| Handover past 200K | `contexts` | 255K, 315K | 238K median, 319K p90; 62 threads past 200K before their first landing | No. The handover comes between increments, and the growth came inside them. |
| A fresh thread per session | thread reuse | — | 154 spawns, 1 refused and retried, none reused | Yes. |

## 8. Proposals

Each gives a rule in one sentence, its evidence, the document it changes,
and the metric it should move. They are ordered by expected effect.

1. **The supervisor starts its own successor once its context passes
   300K; `plan wake` starts a fresh designer from a short brief when the
   last designer's context is past 200K or idle for over an hour; a busy
   designer gets a message, never a second wake.**
   - The brief names the role, the question, the plan, and the earlier
     answers and commits that touch the question.
   - **Evidence:**
     - The supervisor's 381M above 500K, and its answers checking less
       code as its context grew (section 5, items 1 and 2).
     - The designer's 514M, 75% to 80% of it re-sent history, and the fork
       at 22:30 (section 5, item 3).
     - S1 already started S2 with `claude --bg`. `plan` could do it with
       `plan handover supervisor`.
   - **Changes:** development.md, sections 2.5, 10.4 and 10.5; `plan wake`,
     and a `plan handover`.
   - **Moves:** `tokens_claude` down by about half; `answers_reversed`
     down.
2. **When a session's Needs have landed, and before the coordinator starts
   it, a read-only sub-agent of the supervisor checks the session file
   against `main`: it greps every symbol, file and signature the file
   names, compiles its sketches in a scratch worktree and generates its
   schemas. The supervisor lands the corrections, and only then does the
   session start.**
   - **Evidence:**
     - 69 plan-defect rulings, about 55 of them visible to these checks.
     - The sessions compiled at the plan's base had 3 defects.
     - The supervisor's own memory note: "grep main for every symbol"
       (section 5, items 2 and 4).
   - **Changes:** development.md, sections 5.4 (what can be compiled at
     the base, and what is checked at the start), 7.2 and 2.5.
   - **Moves:** `escalations` (plan defect) down; `answers_reversed` down.
3. **A plan has at most about 60 steps. A larger design is planned as
   several plans run in turn, each written against the code the last one
   left.**
   - **Evidence:**
     - Late sessions' sketches named code that did not yet exist (section
       5, item 4).
     - 51 steps were added.
     - The rules grew from 9 to 39.
     - The record outgrew every reader: `log.md` reached 1.6 MB, and
       `decisions.md` 170 KB, which 93 implementer threads read in part
       and 17 read whole.
     - The tail ran at 3.8 steps an hour.
     - This plan falls into four such plans along its own layers: the
       domain's debts, the store, the workers' channel and the service,
       and the client with the one process.
   - **Changes:** development.md, sections 5.1 and 5.3.
   - **Moves:** `escalations` per step and `step_h` down; `contexts` down.
4. **`land` builds, formats and runs clippy on the gate's crates in the
   build lane, and rebases there, before it takes the gate lane for the
   suites alone.**
   - **Evidence:**
     - A passing gate spent 65 s of its 98 s alone on building and lints.
     - The build lane was paused 76% of the run.
     - Replaying the queue, build waits fall from 45.7 hours to about 6.8.
     - The supervisor's three kit changes of 16:22 (section 5, item 9).
   - **Changes:** development.md, sections 9.1 and 11.1; `land` and
     `heavy`.
   - **Moves:** `queue_wait_ratio` and `step_h` down.
5. **The card takes this plan's general rules on edits outside Files:
   rulings 108, 112, 114, 116, 122–125, 129, 145, 195, 197, 198, 209, 225,
   227, 230, 247 and 262, worded as the plan's README words them; and
   section 8.2's "never merge tests to fit a budget" gives way to
   testing-strategy.md, 8's table tests.**
   - **Evidence:**
     - About 45 scope escalations and 34 process rulings; about 20
       rulings widened one rule in 19.5 hours.
     - About 23% of decisions cited a rule that already existed.
     - Ruling 219 already contradicts section 8.2 (section 5, item 6).
   - **Changes:** development.md, sections 8.2, 8.5 and 8.7.
   - **Moves:** `escalations` (cross-session scope, process) down.
6. **An increment is priced from the rates of code already landed, an
   exhaustive translation per type it translates; one priced past 600
   hand-written lines is split; and every increment lands within 1.5
   times its Size without asking (ruling 196).**
   - **Evidence:**
     - 17 increments took 2.4 times their Size, and 18 rulings moved the
       margin.
     - Increments of 1,200 to 2,961 lines drove the implementers'
       contexts (section 5, items 5 and 7).
   - **Changes:** development.md, section 5.3; the planner's
     code-organization.md template.
   - **Moves:** `escalations` (size) down; `contexts` down.
7. **`plan check` times the plan's repository's focused suite on the
   machine that will gate it, and fails when the suite takes more than
   two thirds of its budget there; it also fails when a background
   service such as the code-graph indexer is not at idle priority.**
   - The indexer's wrapper (`codebase-memory-mcp-idle`) moves into the
     kit.
   - **Evidence:**
     - The suite stood at 14.3 s of 15 s by H2.
     - 55 timeouts.
     - The laptop CPU was found only at H17.5.
     - The indexer was the busiest process at 183 of 488 gate starts
       (section 5, item 10).
   - **Changes:** development.md, sections 6 and 11.4; `plan check`.
   - **Moves:** gate timeouts and `gates_per_step` down.
8. **An implementer waits on a running command with one wait of 300,000
   ms on its cell, and this takes precedence over Codex's guidance
   against waits longer than 60 seconds, as section 7.4 already says for
   the coordinator; it never sends empty input, and never runs `ps` or
   `pueue status`.**
   - **Evidence:** 433M tokens (15.7%) in empty polls. Only 23 of 2,623
     waits used 300 s (section 5, item 8).
   - **Changes:** development.md, section 8.3; the spawn prompt.
   - **Moves:** `tokens_implementers` down.
9. **An implementer whose context passes 250K inside an increment commits
   its work on its branch, writes its checkpoint and hands over; a
   checkpoint names files, fixtures and traps, and never repeats the
   standing pass-ons.**
   - **Evidence:**
     - 62 threads crossed 200K before their first landing.
     - A successor needed 106K before its first edit.
     - Checkpoints repeated the pass-ons (section 5, item 7).
   - **Changes:** development.md, sections 7.3 and 8.6.
   - **Moves:** `contexts` and `tokens_implementers` down.
10. **The coordinator hands itself over to a fresh thread at about 250K,
    from `context.md`. A `plan log` command appends a report with its
    receipt time, so that reports are not retyped. When no session runs,
    a `plan wait` blocks until the doorbell rings, in place of 60-second
    sleeps.**
    - **Evidence:**
      - 9 compactions with 23 minutes dead.
      - Notes that lost their spaces as the context grew.
      - 641K characters of reports retyped, and 147 shared time stamps.
      - 205 requests (52M tokens) of sleeps (section 5, item 11).
    - **Changes:** development.md, sections 7.3, 7.4 and 7.5; `plan`.
    - **Moves:** `tokens_coordinator` and `compactions` down.
11. **The Codex roles start without the code graph's project listing: the
    coordinator's profile leaves out the codebase-memory server, and an
    implementer's prompt names the graph's project for its repository.**
    - **Evidence:**
      - `list_projects` took 5.8% of the implementers' input over 536
        calls.
      - The coordinator carried 24K of listings in 620 requests (section
        5, item 8).
    - **Changes:** the workspace's AGENTS.md and
      `coordinator.config.toml`.
    - **Moves:** `tokens_implementers` and `tokens_coordinator` down.
12. **A session file marks each increment that does not build on the one
    before it, and the ledger makes such an increment ready when its own
    Needs have landed, so that it starts in its own thread beside its
    session.**
    - **Evidence:** the tail's 1.7 to 2.4 sessions running; 26.5 waited
      30 minutes; the steers logged at 10:25 and 16:27 (section 5, item 12).
    - **Changes:** development.md, sections 5.2, 7.2 and 7.3; `plan set`
      and `plan status`.
    - **Moves:** `step_h` down in the tail.
13. **`plan pulse` runs `plan downstream` itself once the plan has landed
    upstream code, and prints which repositories it found broken; and
    `land` refuses a `Cargo.lock` change that moves a git source to an
    ancestor of its current pin.**
    - **Evidence:**
      - `plan downstream` ran 0 times, and smith's breakage surfaced only
        at the repins.
      - smith-repin was held 7.1 hours.
      - smith's `83122ea0` moved its skein pin backwards and undid
        smith-repin (section 5, item 13).
    - **Changes:** development.md, sections 5.7, 9.1 and 14.1; `plan
      pulse` and `land`.
    - **Moves:** `pin_lag` and holds down.
14. **A question for the user is pushed at once, in one line with the
    choice and the default; a hold stays an open escalation until the
    user answers; and a live proof first runs against the fake provider
    with the same fixture and answer check, and counts its budget in
    completions, not runs.**
    - **Evidence:**
      - 26.6 waited 113 minutes with no push notification.
      - Ruling 186 waited 7.1 hours overnight, and was then not
        understood.
      - Runs 1 and 2 failed on what an offline run would have shown
        (section 5, items 13 and 14).
    - **Changes:** development.md, sections 5.5 and 10.4.
    - **Moves:** `blocked_h` down, with holds counted.
15. **`plan pulse` prints the tokens spent since the claim, by role, beside
    the plan's budget: its steps times the history's tokens per step.**
    - **Evidence:** the spend limits stopped the supervisor twice, and the
      user wrote "I'm low on credit" (section 5, item 15).
    - **Changes:** development.md, sections 5.2 and 14.2; `plan pulse`.
    - **Moves:** `tokens_per_step` down.
16. **`plan metrics` counts what section 15.2 says. It:**
    - counts Claude Code messages once across forked sessions;
    - counts the planner's session, which `plan join planner` records
      before the claim;
    - counts escalations once each, by the six kinds of section 10.2,
      which a `plan block` writes from a fixed list;
    - counts steers under either heading;
    - counts holds in `blocked_h`;
    - reads a Proof's layer with its punctuation removed, and finds
      sessions numbered "06";
    - gives suites per repository;
    - computes `pin_lag` from a fresh fetch of `origin/main`.

    - **Evidence:** section 3.
    - **Changes:** the kit; development.md, section 15.2 (holds).
    - **Moves:** makes the metrics comparable.
17. **The kit keeps the run's record true:**
    - `plan set` refuses a step named like an option;
    - `plan watch` reads a `blocked.md` with only its title as empty;
    - `land` refuses a commit whose `Step:` git cannot parse;
    - `plan status` honours an accepted anomaly's note;
    - `heavy` removes its pueue task when killed;
    - `plan pulse` reads the budget the profile now holds;
    - `plan release` lists every repository's STATUS row still marked
      "active".

    - **Evidence:** section 5, item 16.
    - **Changes:** the kit; development.md, section 7.6.
    - **Moves:** no metric. The record stays true.

## 9. Debts left in the code

For jig's `docs/development/tech-debt.md`. None of these is listed there
yet. Paths and lines are at `d91a89c`.

- **Ruling 249's doubled timeout.**
  - `.config/nextest.toml:9` reads 30 s, while its header (lines 1–2) and
    `workflow.md:41` still say 15 seconds.
  - The last gate took 15.041 s. After the doubling, 10 of 37 passing
    gates took over 15 s.
  - Restoring 15 s needs the test-cost pass that is still with the user.
- **Numbered splits with a copied loop.**
  - `jig-protocol-client-link/src/translate/` has `documents_part0` to `8`,
    `pages_part0` to `8` and `facts_part0` to `2`.
  - The same piece-decoding loop is written 51 times in 21 files, for
    example `documents_part0.rs:98`.
- **Orphaned goldens.** `jig-protocol-store/golden/{core,people,tasks}/v1`
  hold 2,078 files that no test reads; `tests/store-codecs/tests/drift.rs:45-57`
  and `fuzzy_decode.rs:42` read only v2.
- **Long translation functions.**
  - `jig-protocol-client-link/src/translate/asks.rs:574` `ask` (182
    lines).
  - `jig-protocol-engine-clients/src/translate/asks.rs:9` `ask` (160
    lines).
  - `jig-protocol-store/src/codec/people/asks.rs:348` `encode_ask` (156
    lines) and `:506` (120 lines).
  - Production functions over 60 lines went from 292 to 389. Of the 94 new
    ones, 29 are in engine-clients, 23 in the store and 12 in client-link.
- **Lint exceptions in production code** went from 214 to 360.
  `#[expect(clippy::manual_map)]` went from 33 to 159 (for example
  `jig-domain-accounts/src/domain.rs:343`), and `too_many_lines` from 69
  to 76.
- **`struct_excessive_bools` in test worlds**, against the plan's "Don't"
  list: `tests/channel/src/referee.rs:10`, `relay.rs:42`, `world.rs:39`
  and `processes.rs:19` (jig 27.1 and 27.3).
- **Arms that the types could remove.**
  - `jig-protocol-accounts/src/translate.rs:40,47,48,54,56,82,98`: arms
    for variants the engine never meets.
  - `jig-shell-native/src/files.rs:280` and `shell.rs:181`: jig's first
    `panic!`s in production code.
  - `testing/jig-test-worker/src/domain.rs:285,431`: `unreachable!`.
- **Files past 800 lines that grew:**
  - `tests/ops/src/conformance/mod.rs` (+104 lines, now 1,079);
  - `jig-domain-core/src/tests.rs` (2,987);
  - `jig-view/src/owner_deployment_words.rs` (1,109);
  - `jig-domain-core/src/routing/tasks.rs` (1,449).
- **Tests over 40 lines, written or grown in the run:**
  - `jig-protocol-charter/src/tests.rs:11` (57);
  - `jig-domain-host/src/tests.rs:140` (51);
  - `tests/client/src/joint/stories/fault_stories.rs:143` (45);
  - `tests/boundary/src/tests/graph.rs:294` (43);
  - `jig-domain-core/src/tests.rs:1691` (43);
  - `tests/core/tests/fuzzy_watches.rs:11` (41).
- **Steps that added many tests after ruling 219**, candidates for table
  tests:

  | Step | Tests added |
  |---|---|
  | 27.1 | 28 |
  | 31.5 | 20 |
  | 32.4 | 19 |
  | 33.2 | 18 |
  | 31.3 | 17 |
  | 31.6 | 17 |
  | 18.2 | 15 |
- **Generated volume.**
  - 187K generated lines in 9 files;
    `jig-protocol-client-wire/src/generated/v7.rs` alone is 99,451 lines.
  - The client wire went from v1 to v7 in the run.
  - Its build cost was not measured.
- **The fake store** stays for `ops` (later.md).

Outside jig:
- **skein:**
  - `.config/nextest.toml` also reads 30 s (`886a7d4`), and does not say
    when it ends;
  - skein's fuzzy suite reached 56.8 s of its 60 s (skein-02.s6).
- **smith:**
  - its STATUS row is still "active";
  - its skein pin went backwards (`83122ea0`);
  - its test person still reads one byte at a time, which waits for the
    skein plan of option (a).
- **temper:** its reconciliation, with 288 errors, is later.md's last row.
- **The run's directory:** with this retrospective landed, `logs/`'s
  binaries, scratch crate and copies, and the 415 MB of settle logs, may
  go. kache's store is over its cap.
