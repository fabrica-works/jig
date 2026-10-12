# Session 06: a conversation's first attempt and its current window

The core records, at each claim, the conversation's first attempt: the
claim's own attempt when the run starts fresh, the previous one's when it
resumes, never zero; a claim that fails gives the task back the record
the store still holds. The assignment names that first attempt, from
which the run's identity is derived, the same across a conversation's
attempts. The core keeps each task's current window, where the last turn
that opened one stands, carried across claims and restarts, and a run
that resumes reads from that window's first turn within the resume
limit. Last, the claim records the kind of host the core chose for it,
and the fleet places it on a host of that kind, so a restart settles an
engine's claim at once and waits for a worker's. It pays item 7 and
6(f) of findings/domain.md, and gives jig 17
(transcripts' files) the window it reads from. It changes the core, the
roots' assembly and their transcript loads; never the hub's or the
fleet's boundaries (decisions.md, 6), whose turns jig 05 gave the window
flag and whose assignment jig 05 gave the identity's bytes.

Paths carry jig 01's names (`crates/jig-domain-core/` was
`crates/jig-core/`, `crates/jig-protocol-charter/` was
`crates/jig-charter/`). Line numbers are as built at jig `1cdfeb62`; jig
01, jig 04 and jig 05 may move them by a few lines.

## Read first

- jig's `domain/engine.md`, 5.4 (what the store keeps: "each live agent
  task's conversation: its first attempt, and where its current window
  starts"), section 6 (restart: engine claims settled at once), 7.1
  (step 3: the claim commits "the host kind it is placed on"; step 4:
  the conversation's first attempt and the run's identity), 7.2
  (transcripts: "Committed turn by turn", "Resuming") and section 8
  ("A run goes to a host with a free slot of a kind its charter
  allows").
- The designer's decision of 2026-10-12, relayed by the coordinator: the
  claim commits the host kind the core chose at the claim (a free slot
  of a kind the charter allows), and placement goes to a host of that
  kind.
- This plan's `jig/07-charters-and-budgets.md`, 07.4 (`Charter::hosts`,
  `Ask::ClaimDone`'s kinds, `RunProof::claimed`'s host) and its question
  "A charter that both kinds of host may run", which the decision
  answers.
- jig's `domain/hosts.md`, 6.4 ("A resumed run is given its current
  window's turns only").
- jig's `protocol/channel.md`, 6.1 ("Down, a resumed run's transcript is
  its current window's turns") and section 7 ("A run's identity").
- jig's `protocol/store.md`, 8.3 and 15, item 4 ("turns as the store
  keeps them").
- smith's `protocol/transcript.md`, section 3 (windows), and smith's
  `domain/host.md`, section 2 (the identity "the same across the run's
  starts, resumes included").
- findings/domain.md, item 7, 6(f), "Defects found on the way" (items 7
  and 6(f)).
- This plan's `jig/05-hub-link.md`, 05.2 (`RunAssignment.identity`) and
  05.3 (`Turn.opens_window`, `core::Event::AcceptedTurn.opens_window`).

## What the design asks

- **The conversation's first attempt** (engine.md, 7.1): the assignment
  carries "its conversation's first attempt, from which the engine's
  protocol layer derives the run's identity ...: 16 bytes from the
  deployment's id, the task's number and that attempt, the same for
  every attempt of a conversation ... and never shared by two. No domain
  holds it". channel.md, 7: "the store keeps nothing for it but the
  conversation's first attempt."
- **The current window** (engine.md, 7.2): "Its conversation is the
  turns from the last run that started fresh, and its current window the
  conversation's turns from the last that opens a window: a window turn,
  which a compaction made, or the conversation's first." "The core
  commits the turn ... and, when it opens a window, with its task's
  current window starting there".
- **Resuming** (engine.md, 7.2): "A run that resumes is given its
  current window only, within the resume limit (the deployment's bound
  on the current window's bytes, across the attempts it spans) ... One
  past the limit starts fresh from a brief that carries the transcript's
  tail. Earlier windows' turns stay readable by parties ..., and are
  never sent to a run."
- **Turns as the store keeps them** (store.md, 15, item 4): "saved with
  whether each opens a window ...; a resume reads the current window,
  from the turn the core keeps as its start".

- **The claim's host kind** (engine.md, 7.1, step 3): the claim
  commits "the host kind it is placed on"; by the designer's decision,
  the kind the core chose at the claim, where placement then goes.

## Choices this plan makes

- `RunProof.transcript_from` becomes `RunProof.conversation`, the conversation's first attempt, never zero.
- The current window's start is `RunProof.window: Option<(u64, u32)>`, the attempt and turn of the last turn that opened one; `None` until the conversation's first turn commits.
- A proof replaced by a claim in preparation is kept aside until the claim commits or fails, and a failure puts it back.
- The run's identity is derived by `jig_protocol_charter::run_identity`, the engine's protocol layer's function, which `ops`'s root calls until it hands the assignment out (`ops` composes no protocol layer of its own).
- The root loads a task's turns from the window's start key; a task with no window loads nothing.
- The brief's transcript tail is the current window's tail.
- The core chooses a claim's host kind with the fleet's own placement rule (`channel::choose`: a worker holding the workstream, then the most free slots); with no free slot, `Worker` when the charter allows workers, `Engine` otherwise.

## Rules that bind it

- From README, "Rules of this plan": "A durable record's codec follows
  its record. Once jig 14 or jig 19 has landed a family's codec, an increment of
  any session that changes a durable record of that family changes its
  schema, its generated codec and its translation in the same increment,
  and the record's family's version moves at the first such change in
  this plan".
- From code-organization.md, section 6: "No increment of this plan: ...
  leaves a legacy or compatibility path past the increment that its
  session file names to remove it; ... decodes or encodes bytes in a
  domain crate".
- From code-organization.md, section 4: "the core's vocabulary toward
  the root (`routing/vocabulary.rs`) | `jig-domain-core` | each session
  adds its variants; hotspot".
- For this session: a file already over 800 lines (named in each
  increment's Files) must not grow by more than 20 lines in an
  increment; new code goes in a module of its own (code-organization.md,
  section 7). `git diff --stat` in the commit body shows it.

## Increments

### 06.1 The conversation's first attempt is recorded at each claim

Scope: a claim records the conversation's first attempt: its own attempt
when the run starts fresh (a first claim, a charter that does not
resume, a conversation past the resume limit, a transient retry), the
previous proof's when it resumes a conversation that has turns; and a
claim that fails before it commits (refused, writer waiting, a refused
payload) restores the proof the store holds, so the next resume reads
the same conversation (engine.md, 7.1 and 5.4; findings/domain.md, item
7, defects 1 and 2).
Needs: nothing.
Files:
- `crates/jig-domain-core/src/stored.rs` (`RunProof`, :517-534;
  `RunProof::claimed`, :576-586).
- `crates/jig-domain-core/src/routing/run_preparation.rs`
  (`workspace_prepared`, :89-96; `route_claim_refused`, :386-390;
  `route_claim_prepared`, :407; shared with jig 04 and jig 07).
- `crates/jig-domain-core/src/routing/tasks.rs` (1,414 lines; the
  `WriterWaiting` arm at :227-230, the refused payload at :1119-1121).
- `crates/jig-domain-core/src/run.rs` (`begin_transcript`, :137-158),
  `crates/jig-domain-core/src/lib.rs` (707 lines; `valid_proof`,
  :671-680; `quiescent`, :615; a new map beside `proofs`).
- `RunProof` literals: `crates/jig-domain-core/src/tests.rs:455-458,1719-1722`
  (2,936 lines); `crates/jig-domain-core/src/tests/activity.rs:68,220`.
- When jig 19 has landed the core family's codec: its schema, codec and
  translation in `jig-protocol-store`.
Shape:
- `RunProof.conversation: u64`, doc "The conversation's first attempt:
  this claim's attempt when its run starts fresh, the previous proof's
  when it resumes; never zero", replacing `transcript_from`;
  `valid_proof` refuses `conversation == 0` or
  `conversation > attempt`.
- `workspace_prepared`: `conversation` is `transcript.from` only when
  the run resumes and the loaded conversation has a turn
  (`transcript.last.is_some()`); otherwise `attempt`.
- `Core::replaced: Map<u64, RunProof>`: the proof a claim in
  preparation replaced, by task, at most `limits.tasks.tasks`; put back
  in `proofs` by the three failure paths, dropped when the claim's
  decision commits (`route_claim_prepared`'s tasks `Done`); empty in
  `quiescent`.
Test shape:
- `tests/core/tests/runs.rs` (scenario functions called from
  `runs_scenarios`, :236), with `World::new`
  (`tests/core/src/effects.rs:161`), `World::turn` (:1215), `World::end`
  (:1228), `World::say` (:1193):
  `a_resuming_charters_first_claim_records_its_own_attempt` (the
  proof in `world.store.rows` under `core::CoreKey::RunProof(task)`
  names the first assignment's attempt as its conversation);
  `a_conversations_attempts_name_one_first_attempt`. Each under 0.5 s.
- `tests/core/tests/restart.rs` (253 lines, existing kind), with a
  connector that refuses the claim (`core::Event::ClaimRefused`,
  `crates/jig-domain-core/src/routing/run_preparation.rs:386`):
  `a_refused_claim_keeps_the_conversation_for_the_next_resume`. Under
  0.5 s.
- `crates/jig-domain-core/src/tests.rs`'s two `RunProof` literals
  (:458, :1722) name `conversation: 1`, their claims' attempt; no test
  asserts zero.
Proof: world, `heavy cargo nextest run -p jig-core-world --test runs`,
once: a chat's first, second and third runs name one conversation's
first attempt, the first run's own; under 0.5 s per scenario.
Don't: keep zero as a sentinel for "from the task's first turn"; read the store from the core.
Size: at most 60 lines of production code.
Removes: `RunProof.transcript_from` and its zero sentinel; the three
`drop(core.proofs.remove(..))` of a failed claim.

### 06.2 The assignment names the conversation's first attempt

Scope: the core's assignment parts name the conversation's first
attempt; the test application's root passes it to its world, whose
scripted workers see the same one across a conversation's attempts; the
engine's protocol layer derives the run's identity from the deployment's
id, the task and that attempt, and `ops`'s root puts it in its hub's
assignment, replacing jig 05's zeros (engine.md, 7.1; channel.md, 7;
findings/domain.md, item 7, gap 3).
Needs: jig 05's assignment (`jig_domain_host::RunAssignment.identity`).
Files:
- `crates/jig-domain-core/src/routing/vocabulary.rs` (`Now::RunPrepared`,
  :254-264; hotspot, the assignment), its producer
  `crates/jig-domain-core/src/routing/run_preparation.rs:131`.
- `crates/jig-protocol-charter/src/lib.rs` (:15-20), new module
  `crates/jig-protocol-charter/src/identity.rs`;
  `crates/jig-protocol-charter/Cargo.toml` takes `sha2` (the workspace's,
  `Cargo.toml:73`). Shared with jig 10, which adds the result and
  call-key readers to the same crate.
- `ops`'s root: `examples/ops/domain/src/route.rs:1245-1258` (1,585
  lines), `examples/ops/domain/src/assemble.rs` (`assignment`,
  :48-112), `examples/ops/domain/src/domain.rs` (the deployment's id
  kept from its `core::Config`).
- The test root: `Assignment` (`testing/jig-test-domain/src/lib.rs:133-146`),
  built at `testing/jig-test-domain/src/routes.rs:421-437`; read by
  `tests/core/src/peers.rs:255-262`, `tests/core/src/world.rs:634`,
  `tests/core/src/effects.rs:719`, `tests/core/src/observations.rs:835-846`,
  `tests/conformance/src/lib.rs:507`.
- `testing/jig-fake-workers/src/lib.rs` (`Assignment`, :17-22),
  `testing/jig-fake-workers/src/tests.rs:4`.
- The conformance referee: `Observed::Assigned`
  (`testing/jig-conformance/src/referee.rs:273,874-895`), observed at
  `tests/core/src/observations.rs:835-846` and
  `tests/ops/src/conformance/mod.rs:337`.
Shape:
- `Now::RunPrepared` gains `conversation: u64`, the claim's
  `RunProof.conversation`.
- `jig_protocol_charter::run_identity(deployment: &[u8; 16], task: u64, conversation: u64) -> [u8; 16]`:
  the first 16 bytes of the SHA-256 digest of the deployment's id, the
  task's number and the conversation's first attempt, each number as 8
  bytes big-endian (channel.md, 7).
- `ops`'s `assemble::assignment` sets `RunAssignment.identity` from
  `run_identity`.
- The test root's `Assignment` gains `conversation: u64`;
  `jig_fake_workers::Assignment` gains `conversation: u64`, which
  `tests/core/src/peers.rs` fills.
- The referee's promise: "a conversation's attempts name one first
  attempt; a fresh start names its own".
Test shape:
- `crates/jig-protocol-charter/src/tests.rs` (existing): 
  `a_conversations_attempts_share_one_identity` (same deployment, task
  and conversation, any attempt) and
  `two_conversations_of_a_task_have_two_identities`; the expected bytes
  are literals computed once with `sha256sum` and written into the
  test. Each under 20 ms.
- `tests/core/tests/runs.rs`: `a_resumed_run_names_its_first_runs_attempt`
  and `a_fresh_start_after_a_transient_failure_names_its_own`, with
  `World::retry_due` (`tests/core/src/effects.rs:1258`). Each under
  0.5 s.
Proof: world, `heavy cargo nextest run -p jig-core-world --test runs`,
once: a chat's resumed runs carry the first run's attempt as their
conversation, and a fresh start its own; under 0.5 s per scenario. (The
identity's bytes are shown by `jig-protocol-charter`'s unit tests; the
channel's world, jig 25, carries them to an agent.)
Don't: keep the identity or its digest in any domain's state or record;
read it in the hub.
Size: at most 60 lines of production code.
Removes: the zero identity in `ops`'s `assemble::assignment` (jig 05's
transitional).

### 06.3 The core keeps each task's current window

Scope: a committed turn that opens a window moves its task's current
window to start there, in the same commit; the start is kept with the
task's claim, carried to a resuming claim, cleared by a fresh start, and
checked at a restart; each saved turn says whether it opens a window
(engine.md, 7.2 and 5.4; store.md, 15, item 4; findings/domain.md,
6(f), gaps 1 and 2).
Needs: jig 05's turns opening a window (`core::Event::AcceptedTurn.opens_window`).
Files:
- `crates/jig-domain-core/src/stored.rs` (`TurnRecord`, :461-478;
  `RunProof`, :517-534).
- `crates/jig-domain-core/src/routing/runs.rs` (`route_accepted_turn`,
  :681-719, which jig 05 left dropping `opens_window`; 784 lines).
- `crates/jig-domain-core/src/run.rs` (`Transcript`, :25-32;
  `begin_transcript`, :137-158), `crates/jig-domain-core/src/routing/run_preparation.rs`
  (`workspace_prepared`, :89-96), `crates/jig-domain-core/src/lib.rs`
  (`valid_proof`, :671-680).
- `TurnRecord` constructions: `crates/jig-domain-core/src/routing/runs.rs:699`;
  `tests/core/tests/store_peer.rs:36`; `tests/core/tests/runs.rs:123,136`;
  `tests/client/src/joint/archive.rs:218`; `tests/client/src/joint/wire/activity.rs:332`
  (and the updates at :342, :365). Readers:
  `crates/jig-domain-core/src/reads/rows.rs:345,1359` (1,744 lines);
  `crates/jig-domain-core/src/run.rs:163`; `examples/ops/domain/src/restart.rs:96`;
  `examples/ops/domain/src/store.rs:67`; `testing/jig-test-domain/src/lib.rs:59,553`.
- `RunProof` literals as in 06.1.
- When jig 19 has landed the core family's codec: its schema, codec and
  translation in `jig-protocol-store` (the turn's and the proof's).
Shape:
- `TurnRecord.opens_window: bool`, from `AcceptedTurn.opens_window`.
- `RunProof.window: Option<(u64, u32)>`, doc "The current window's first
  turn, by attempt and turn: the last committed turn of this
  conversation that opened a window"; set by `route_accepted_turn` for a
  new turn that opens one; `Transcript` gains `window`, read from the
  previous proof in `begin_transcript`; `workspace_prepared` writes it
  into the new proof when the run resumes, and `None` when it starts
  fresh; `valid_proof` refuses a window before `(conversation, 1)` or
  past the proof's own `(attempt, turn)`.
Test shape:
- `tests/core/tests/runs.rs`, with a new world helper
  `World::window_turn(number, spent, read, body)` (a turn whose
  `opens_window` is true; `tests/core/src/effects.rs`, beside `turn`
  :1215): `a_window_turn_moves_the_tasks_window_in_its_commit`,
  `a_fresh_start_clears_the_window_until_its_first_turn`. Each under
  0.5 s.
- `tests/core/tests/restart.rs`: `a_restart_keeps_the_window_a_turn_opened`,
  beside `a_restart_after_a_turn_committed_resumes_from_it` (:134).
  Under 0.5 s.
- `crates/jig-domain-core/src/tests/activity.rs` (`RunProof` at :68,
  :220): `a_proof_whose_window_passes_its_turn_is_refused_at_restore`.
  Under 20 ms.
Proof: world, `heavy cargo nextest run -p jig-core-world --test restart -E 'test(a_restart_keeps_the_window_a_turn_opened)'`,
once: a task whose run took a window turn keeps that window across an
engine restart; under 0.5 s.
Don't: read a turn's body to find a window; erase earlier windows'
turns (store.md, 8.5: a task's transcript is dropped in one piece).
Size: at most 60 lines of production code.
Removes: the dropping of `opens_window` in `route_accepted_turn` (jig
05's transitional).

### 06.4 A resume reads its current window within the resume limit

Scope: preparing a run loads its task's turns from the current window's
first turn; the resume limit bounds the window's bytes across the
attempts it spans; a window within it is sent in full to a resuming run,
and one past it starts the run fresh with the window's tail in its
brief; earlier windows' turns are never sent (engine.md, 7.2; hosts.md,
6.4; channel.md, 6.1; findings/domain.md, 6(f), gaps 3 and 4). A
transcript the store answers as unread (a window that starts at another
turn than the core keeps, or a frame that fails its checksum) fails the
run as transient, and the next run of its task starts fresh
(engine.md, 7.2; store.md, 8.3): the core's transcript load gains that
answer here.
Needs: nothing beyond 06.3.
Files:
- `crates/jig-domain-core/src/routing/vocabulary.rs` (`Now::StartPreparation`,
  :238; hotspot), its producer `crates/jig-domain-core/src/routing/run_preparation.rs:335-336`,
  and the fresh-start rule at :245.
- `crates/jig-domain-core/src/run.rs` (`append_transcript`, :163-209;
  `transcript_oversized`, :213-215; `resumed_turns`, :220-229).
- `crates/jig-domain-core/src/brief_text.rs` (`BriefPart::TranscriptTail`,
  :224, :388).
- `Now::StartPreparation`'s matches (11): `examples/ops/domain/src/route.rs:1215,1222`;
  `testing/jig-test-domain/src/routes.rs:368,375`; `tests/core/src/effects.rs:507`
  (the loader, :507-528); `tests/conformance/src/lib.rs:461` (the
  loader); `tests/ops/src/conformance/mod.rs:547` (the loader, :547-566;
  975 lines); `tests/ops/src/joint/front.rs:743`;
  `tests/ops/src/joint/engine.rs:334`; `tests/client/src/joint/session.rs:788`;
  `tests/core/src/client_peer.rs:151`;
  `crates/jig-domain-core/src/tests/completion_retry.rs:44`.
- `ops`'s load: `examples/ops/domain/src/restart.rs:89-103` (`Store::Transcript`).
- `tests/core/tests/runs.rs` (:7-41, :109-116, :118-147, :149-166).
Shape:
- `Now::StartPreparation { task: u64, transcript_waiter: Option<Token>, from: Option<(u64, u32)> }`:
  the window's first turn; the root loads `core::CoreKey::Turn { task,
  attempt, turn }` from it, in key order; `None` loads nothing.
- `append_transcript` refuses a turn before the window's start (a load
  that ignored `from` is a root's bug, failing the preparation as a bad
  page does) and counts only the window in `Transcript.bytes`;
  `transcript_oversized` and `resumed_turns` judge the window.
- The unread answer (ruling 127): `core::Event::TranscriptUnread { task:
  u64 }`, which a root sends when the store answers the task's transcript
  as unread, routed to `tasks::Event::PreparationUnread { task: u64 }`:
  a preparing task's attempt fails as transient, under the existing retry
  policy, and its next run starts fresh, as the fresh-start rule at
  `run_preparation.rs:245` starts one. The roots and worlds that load
  transcripts send it where their loader finds a transcript unread; the
  matches the new variants need are added as ruling 112 says. Production
  lines for this count toward 06.4's Size, now at most 120.
Test shape:
- `tests/core/tests/runs.rs`: new scenarios
  `a_chat_resumed_after_a_compaction_is_sent_only_its_window` (engine.md,
  15: "a chat resumed from its current window after a compaction") and
  `a_window_past_the_resume_limit_starts_fresh_with_its_tail`;
  `a_chat_answers_parks_and_resumes` (:7) keeps its assertion that the
  third run gets both turns (:23-28): no turn after the first opens a
  window there, so both are its current window;
  `loaded_transcript_pages_reject_repeated_and_reversed_turns` (:118)
  gains a turn before the window's start, refused. Each under 0.5 s.
- `tests/core/tests/restart.rs`: `a_restart_between_a_window_turn_and_its_resume_sends_the_window`.
  Under 0.5 s.
- `tests/core/tests/runs.rs`: `a_transcript_read_as_unread_fails_the_run_and_the_next_starts_fresh`
  (ruling 127). Under 0.5 s.
Proof: world, `heavy cargo nextest run -p jig-core-world --test runs -E 'test(runs_scenarios)'`,
once: a chat that compacted resumes with its window turn first and none
of the turns before it; under 0.5 s.
Don't: send a run an earlier window's turns; drop the calls committed
since the last turn from a resume (they stay its note, engine.md, 7.2).
Size: at most 120 lines of production code (ruling 127).
Removes: the resume's count of the conversation's turns before its window in
`append_transcript`.

### 06.5 The claim records the host kind it chose

Scope: when the core claims a run, it chooses the kind of host it goes
to, a free slot of a kind the task's charter allows, by the fleet's
placement rule, and commits that kind with the claim; the fleet places
the attempt on a host of that kind only; a restart settles an engine's
claim at once and keeps a worker's for the grace (engine.md, 7.1 step
3, section 6, section 8; the designer's decision of 2026-10-12;
findings/domain.md: `RunProof::claimed`, `stored.rs:576`, records
`Worker` for each claim). This replaces jig 07.4's rule (`Engine` iff
the charter's hosts are the engine's alone).
Needs: jig 07's hosts by charter (`Charter::hosts`, the kinds
`core::Ask::ClaimDone` carries, `RunProof::claimed`'s host).
Files:
- `crates/jig-domain-fleet/src/domain.rs` (379 lines; a query beside
  `host_views`, :140), `crates/jig-domain-fleet/src/channel.rs`
  (`choose`, :214-232).
- `crates/jig-domain-core/src/routing/run_preparation.rs`
  (`workspace_prepared`, :89-96, where the proof is made; shared with
  jig 04 and jig 07), `crates/jig-domain-core/src/stored.rs`
  (`RunProof::claimed`, :576-586).
- `crates/jig-domain-core/src/routing/tasks.rs` (1,414 lines; the
  `Ask::ClaimDone` jig 07.4 builds at :647, :665).
- `crates/jig-domain-core/src/routing/children.rs` (`record_host`,
  :105-119, and its call in the `fleet::Request::Assign` arm, :232).
- `crates/jig-domain-core/src/restart.rs` (:55, the engine's claims
  settled at once).
- Tests: `tests/core/tests/charters.rs` (jig 07's file),
  `tests/core/tests/restart.rs`, `crates/jig-domain-fleet/src/tests.rs`.
Shape:
- `jig_domain_fleet::Domain::free_kind(&self, kinds: Kinds, workstream: u64) -> Option<HostKind>`:
  the kind of the channel `channel::choose` picks now, or `None`.
- `workspace_prepared` takes
  `free_kind(charter.hosts, workstream)`, or the fallback of the
  choices above, passes it to `RunProof::claimed(.., host)`, and keeps
  it for the claim; `Ask::ClaimDone` carries `Kinds::Workers` or
  `Kinds::Engine` for it, which the roots put in the fleet's `Start` as
  jig 07.4 has them do.
- `record_host` goes: the fleet's `Assign` names the claimed kind, which
  the arm asserts.
Test shape:
- `tests/core/tests/charters.rs`, with jig 07's charters fixture and the
  engine's slots of `tests/core/src/world.rs`:
  `a_charter_both_kinds_may_run_claims_the_engine_when_only_its_slot_is_free`,
  `a_claim_for_the_engine_is_placed_only_on_the_engine`. Each under
  0.5 s.
- `tests/core/tests/restart.rs`: `a_restart_settles_an_engine_claim_of_a_both_kinds_charter_at_once`.
  Under 0.5 s.
- `crates/jig-domain-fleet/src/tests.rs`: `free_kind_follows_the_placement_rule`.
  Under 20 ms.
Proof: world, `heavy cargo nextest run -p jig-core-world --test restart -E 'test(a_restart_settles_an_engine_claim_of_a_both_kinds_charter_at_once)'`,
once: a run claimed for the engine's slot of a charter both kinds may
run is settled at once by a restart instead of waiting a worker's
grace; under 0.5 s.
Don't: let a root choose the kind; change the fleet's boundary (the
kind travels in `Event::Start`'s `kinds`, as built); write the proof a
second time at placement.
Size: at most 50 lines of production code.
Removes: `record_host` and its second write of the proof; jig 07.4's
`Engine` iff the charter's hosts are the engine's alone.

### 06.s1 A window counts each turn with its length

A step not in the plan (decisions.md, ruling 176), from the designer's
answer on jig 22.3 (jig `e7eb8486`: `domain/engine.md`, 7.2, the resume
limit counts "each turn ... with the four bytes of its length, as the
window crosses to a host"); its text is the designer's.
Shape: `run.rs`'s window, from `begin_transcript` on, counts each kept
  turn as its length plus 4 against `resume_bytes`, in what it keeps and
  in when a window past the limit starts fresh.
Test shape: a core unit test of a window of many one-byte turns that
  starts fresh one turn earlier than when counted without lengths.
Proof: the gate.
Don't: add a turn-count limit.
Size: at most 40 lines of production code.

## Interface sketches

```rust
// crates/jig-domain-core/src/stored.rs (jig_domain_core)
use skein_lib::Wall;

/// Root's current claim evidence (as built, but:).
pub struct RunProof {
    pub task: u64,
    pub attempt: u64,
    /// The conversation's first attempt: this claim's attempt when its run
    /// starts fresh, the previous proof's when it resumes; never zero.
    pub conversation: u64,
    /// The current window's first turn, by attempt and turn: the last
    /// committed turn of the conversation that opened a window.
    pub window: Option<(u64, u32)>,
    // … host, offered, turn, terminal as built.
}

/// One accepted turn, saved with its commit (as built, plus:).
pub struct TurnRecord {
    pub task: u64,
    pub attempt: u64,
    pub turn: u32,
    pub spent: u64,
    pub read: Option<u64>,
    /// A window turn, or the conversation's first.
    pub opens_window: bool,
    pub at: Wall,
    pub transcript: Box<[u8]>,
}
```

```rust
// crates/jig-domain-core/src/routing/vocabulary.rs (jig_domain_core)
use skein_lib::Token;

/// Core output that follows no mutation (as built, but:).
pub enum Now {
    /// Start a task's preparation; the root loads its turns from the
    /// current window's first turn, `from`, in key order, and answers
    /// with its transcript pages. `None` loads nothing.
    StartPreparation { task: u64, transcript_waiter: Option<Token>, from: Option<(u64, u32)> },
    // … RunPrepared gains `conversation: u64`; the other variants as built.
}
```

```rust
// crates/jig-domain-fleet/src/domain.rs (jig_domain_fleet)
impl crate::Domain {
    /// The kind of host the fleet would place a start of `kinds` and
    /// `workstream` on now, by its placement rule; `None` with no free
    /// slot. Read by the core as it claims a run (06.5).
    #[must_use]
    pub fn free_kind(&self, kinds: crate::Kinds, workstream: u64) -> Option<crate::HostKind> {
        let _ = (kinds, workstream);
        todo!()
    }
}
```

```rust
// crates/jig-protocol-charter/src/identity.rs (jig_protocol_charter)
/// A run's identity: the first 16 bytes of the SHA-256 digest of the
/// deployment's id, the task's number and its conversation's first
/// attempt (channel.md, 7). Called by the engine's protocol layer for each
/// assignment; no domain holds the result.
#[must_use]
pub fn run_identity(deployment: &[u8; 16], task: u64, conversation: u64) -> [u8; 16] {
    let _ = (deployment, task, conversation);
    todo!()
}
```

The core owns the conversation's first attempt and the window's start,
kept in the claim's proof and saved with it; the roots load turns from
the window and derive nothing but the identity, which no domain keeps. A
window ends at the next turn that opens one, or at a fresh start.

## Questions to raise when reached

- **A turn that opens a window in a run that starts fresh:** smith's
  first turn of a conversation opens a window; if a world's agent sends a
  first turn without the flag, the window stays `None` and the next
  resume loads nothing. Raise it if a world shows it; do not infer the
  flag in the core.

## Done when

- No zero conversation: `git -C ~/src/rust/jig grep -n 'transcript_from' -- crates testing tests examples`
  prints nothing.
- The core keeps the window: `git -C ~/src/rust/jig grep -n -E 'window: (Some|None)|opens_window' -- crates/jig-domain-core/src/routing/runs.rs`
  prints at least one line.
- The assignment names the conversation: `git -C ~/src/rust/jig grep -n 'conversation' -- testing/jig-test-domain/src/lib.rs testing/jig-fake-workers/src/lib.rs`
  prints at least two lines.
- `ops` derives the identity: `git -C ~/src/rust/jig grep -n 'run_identity' -- examples/ops/domain/src`
  prints at least one line, and `git -C ~/src/rust/jig grep -n -F '[0; 16]' -- examples/ops/domain/src/assemble.rs`
  prints nothing.
- A claim's host kind is written once: `git -C ~/src/rust/jig grep -n 'fn record_host' -- crates/jig-domain-core`
  prints nothing.
- The gate passes on the last landed commit:
  `git -C ~/src/rust/jig log -1 --format='%(trailers:key=Gate-Result,valueonly)' origin/main`
  names `scripts/check.sh passed`.
