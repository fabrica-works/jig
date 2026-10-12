# Findings: the store (`jig-protocol-store`, store.md section 15, the worlds' move)

Code audit for the plan `jig-protocol` (skein's development.md, 5.4). This is
read-only, and no file in any repository was changed.

- **Revisions:**
  - jig is `main` = origin/main `2fae240a`;
  - skein's checkout is `5c9eedd`, but jig's `Cargo.lock` pins skein `30dc6f7`. In that range `crates/skein-kv`, `skein-io/src/file*.rs` and `testing/skein-fake-machine` are identical; only `docs/design/io.md` moved (OpenWrite/Cut);
  - smith is at its `main`.
- **Paths** are relative to each repository's root. When a path has no repository, it is jig's. Hit counts come from `grep -rnE` over `crates examples testing tests` (`*.rs`).

---

## 1. The store vocabulary as built

There is **no store component and no byte encoding anywhere in jig**. The core speaks typed records and keys, the roots wrap them typed, and the worlds answer every load from a typed `BTreeMap`.

### 1.1 Commits

- The core's write is `Request::Write(Write)`, where `Write::Save(Record) | Write::Erase(Key)` (`crates/jig-core/src/routing/vocabulary.rs:18-29,102-107`).
  - It has no "drop a task's transcript whole" (store.md 3) and no turn write distinct from `Save(Record::Core(CoreRecord::Turn))`.
  - `Write::(Save|Erase)` has 139 hits in 42 files: jig-core 39, tests/core 44, ops domain 13, tests/ops 13, fake-store 7, test-domain 6, notes world 8, tests/client 5, tests/conformance 4.
- **Wrapping:** `core::Record` is `Core(CoreRecord)|Tasks(tasks::Stored)|People(people::Stored)|Notes(notes::Record)`, and `core::Key` is `Core(CoreKey)|Tasks|People|Notes` (`crates/jig-core/src/stored.rs:205-227`).
  - Hits: `core::Record::` 322 in 66 files; `core::Key::` 98 in 29 files; `CoreRecord::` 159 in 41; `CoreKey::` 70 in 19.
- **Numbering:**
  - `Counters::next_commit` bumps `Deployment.commits` and returns the header for every commit (`crates/jig-core/src/numbers.rs:133-142`). store.md 6's check, header `commits` equal to skein-kv's last, is therefore possible as built.
  - `Deployment.id: [u8;16]` (`numbers.rs:7-34`) is the format row's deployment id.
- **Ops's root** (`examples/ops/domain/src/boundary.rs`):
  - `Output::Commit{number, writes: Queue<Write>}` (`:227`);
  - `Store::Committed{number}` / `Failed{number}` (`:103-105`), which feed `domain.journal.committed/failed` (`domain.rs:203-204`); the journal is skein-lib's `Journal`;
  - its own `Write::Save(Record)|Erase(Key)` and `Record::key()` (`store.rs:41-59`), with `core_key` mapping each `CoreRecord` to its `CoreKey` (`store.rs:60-85`).
- **The testing root** (`testing/jig-test-domain/src/lib.rs`) has the same shape:
  - `Key`, `Record` and `core_key` (`:25-79`);
  - `Write` (`:81-86`), `Request::Commit{number, writes}` (`:150-162`), `Event::Committed/Failed` (`:271-277`).
  - `core_key` is duplicated verbatim in both roots, so it belongs in jig-core or the component.
- **No fill:** nothing in `crates/` or `examples/` carries a fill or a high-water mark (grep "fill|high.water"). skein-kv's `Committed` carries no fill either (§5); the component must read `Store::used()`.
- Commit plumbing hits: `Output::Commit{|Request::Commit{` 33 in 12 files; `Event::Committed{|Failed{` 33 in 18; `Store::Committed|Failed` 12 in 4.

### 1.2 Restart loads

- **The script** (`crates/jig-core/src/restart.rs`):
  - `RestartStep{LoadCore, LoadInbox, RestoreConnector{connector}, AdoptRuns, ReadAfresh{connector}, SettleOutbox{connector}, Open}` (`:7-23`);
  - `RestartRequest{Step, Idle, Refused{step}}` (`:25-30`);
  - `restart_begin` (`:66`), `restart_refuse` (`:77`) and `restart_done` (`:114`). `LoadCore` is refused while `restoring_proofs` is non-empty (`:121-125`).
  - `RestartStep::` has 141 hits in 20 files.
- **Restore entry points.** None is an event; the root calls them, then feeds child events:
  - `restore_core(CoreRecord) -> Restored{Deployment{commits}|Live|Archive|Rejected}` (`stored.rs:329-383,231-240`). Turn, Terminal and the decisions answer `Archive`, i.e. they are ignored (`:378-381`);
  - `restore_task_row(&tasks::Stored) -> bool` (`stored.rs:274-325`). It returns false for Ended, History, Activity and Milestone (`:320-323`), and fills `restoring_proofs` for live agent tasks (`:301-315`);
  - `restore_proof` (`lib.rs:655-668`) and `restore_call` (`lib.rs:402-413`);
  - `restore_people_row` (`routing/application_routes.rs:154`), `restore_policy` (`policy.rs:355`), `restore_resource_role` (`resources.rs:234`, only during `RestoreConnector`);
  - `restore_inbox_row` (`inbox.rs:418`, only during `LoadInbox`);
  - `inbox_restore_key` (`inbox.rs:397-414`), which admits `Tasks::Live|Ended|PersonProposal`.
  - `restore_(core|task_row|people_row|inbox_row|policy|proof|call|resource_role)` have 33 hits in 13 files.
- **Ops's root:**
  - `restart::route` maps `LoadCore` to `Output::Load{step, range: Range::Core}`, `LoadInbox` to `Range::Inbox`, and `RestoreConnector` to `Range::Observability|Infrastructure` (`examples/ops/domain/src/restart.rs:14-68`, `store.rs:8-18`);
  - the world answers with `Store::Restore(Record)`… `Store::Restored(step)` (`boundary.rs:107-109`);
  - `restart::store` (`restart.rs:70-176`) dispatches: Core rows go to `restore_core`; Tasks to `restore_task_row` then `tasks::Event::Restore`; People to `restore_policy` / `restore_people_row` then `people::Event::Restore`; Notes to `notes::Event::Restore` (which notes ignores: `crates/jig-core-notes/src/domain.rs:136-139`); connector rows to their `Restore`;
  - `Milestone` is accepted and dropped (`restart.rs:122`);
  - after `LoadCore` the root sends `people::Event::Restored` itself (`:80-85`), and `tasks::Event::Restored` at `AdoptRuns` (`:37-39`).
  - Hits: `Output::Load|Range::(Core|Inbox|…)` 27 in 5 files; `Store::(Restore|Restored|Loaded)` 9 in 4.
- **The testing root** has no `Load` output. It uses `Request::Restart(step)` (`lib.rs:152`) and `Event::RestartBegin` / `RestartDone(step)` (`lib.rs:204-205,534-544`); the world calls the pub fn `restore_record` (`lib.rs:1006-1056`, 4 hits in 3 files) directly. Notes rows are accepted and dropped there (`:1052`). `RestartBegin|RestartDone` have 33 hits in 15 files.

### 1.3 Party reads

- **Admission** (`crates/jig-core/src/reads.rs`):
  - `start_jig` (`:308-385`) emits `Held::ReadLoad{load: Token, range: Box<reads::Range>}` (`vocabulary.rs:117`) and waits for `Event::ReadLoaded{load, result: reads::Loaded}` (`vocabulary.rs:335`);
  - `loaded` (`:387-411`) checks visibility again and answers `Held::Read{reply, answer}` (`vocabulary.rs:119`);
  - the bounds are `Limits.load_slots` / `load_bytes` (`vocabulary.rs:287-289`), and `documents.rows` gives the rows per page.
- **Types** (`crates/jig-core/src/reads/vocabulary.rs`):
  - `Scope`, 17 variants (`:61-96`): Task, Proposal, Escalation, Question, PersonTask, Result, Reply, Turn, EndedDelegates, History, Activity, Inbox, Conversations, Goals, Notes, ScopedNotes, Spend;
  - `Position{Owner(Key) | Inbox{at, task, object, key}}` (`:101-106`), where the inbox is a *virtual* index;
  - `Cursor` (`:110-121`): Live, Store, ConversationsBefore, ConversationsLiveBefore, ConversationsStart;
  - `Range{party, scope, after: Option<Position>, backwards, rows, bytes}` (`:174-181`);
  - `Row{value: Record, owner: Option<Box<TaskRecord>>, ancestors: Box<[u64]>, related: Option<Record>, linked: Box<[Box<TaskRecord>]>, inbox: Option<documents::Cursor>}` (`:185-196`). These are the joins the store must make;
  - `Loaded{Page{rows, next: Option<Position>} | Busy | Gone}` (`:208-215`).
- **The core's per-scope expectations:**
  - `rows::position_allowed` / `key_kind` (`reads/rows.rs:1427-1546`) say which family each scope reads;
  - `object` (`:538-826`) and `page_row` (`:828-1000`) read the joins;
  - `backwards()` (`reads.rs:124-148`): Activity, Conversations, Escalation, Proposal and the Updates, Results and Replies inbox groups;
  - `needs_store` (`rows.rs:81-143`): the Needs, Proposals, Escalations, Questions and PersonTasks inbox groups are served live and never loaded.
- Hits: `Scope::<17>` 296 in 17 files (jig-core 191, tests/client 88, tests/core 16, tests/ops 1); `reads::Range` 7 in 4; `Loaded::*` 39 in 7; `reads::Row|Row{` 15 in 7; `Position::*` 14 in 4; `ReadLoad` 33 in 16; `ReadLoaded` 8 in 6.
- **Root routing:** both roots pass `Held::ReadLoad` through to the world (ops `route.rs:327`, test-domain `routes.rs:78`).

### 1.4 Notes' loads

- The notes child emits `Request::Load{owner, range: notes::Range{Entry{name} | Lines{scope, after: Option<u64>}}}` (`crates/jig-core-notes/src/boundary.rs:98-103`, `Request::Load` `:181`). The core lifts it to `Held::NotesLoad{owner, range}` (`vocabulary.rs:162`), and it is answered by `notes::Event::Loaded{owner, rows: Rows{records: List<Record>}, more: bool}` (`notes boundary.rs:162`).
- There is no cursor in the answer: `more` plus the caller's `after` name.
- `NotesLoad` has 16 hits in 16 files; `Event::Loaded{` has 26 in 10 (17 inside notes).
- How the worlds answer it:
  - tests/core scans every row (`tests/core/src/world.rs:569-595`, `effects.rs:796-820`);
  - tests/ops **answers empty** (`tests/ops/src/conformance/mod.rs:540-546`);
  - tests/conformance scans (`lib.rs:554-576`).

### 1.5 Historical point reads (store.md 3, "get")

| Core asks (`Now`, held as a load) | Core hears (`Event`) | Key it needs | Answered by |
|---|---|---|---|
| `SeenChat{request, task}` (`vocabulary.rs:240`) | `SeenChatLoaded{request, row: Option<Box<TaskRecord>>, busy}` (`:455`) | `Tasks::Ended(task)` | tests/ops `joint/engine.rs:234-241` (scan); tests/core `effects.rs:547-560`. **test-domain routes it to `unreachable!`** (`testing/jig-test-domain/src/routes.rs:513-517`) |
| `HistoricalProposal{request, person, project, proposer, proposal}` (`:242`) | `HistoricalProposal{…, row: Option<ProposalDecisionRecord>, busy}` (`:457-465`) | `CoreKey::ProposalDecision{proposer, proposal}` | tests/core `historical.rs:10-32`, `world.rs:519-528`, `effects.rs:578-594` |
| `HistoricalPersonTask{request, person, project, task}` (`:244`) | `HistoricalPersonTask{…, row, busy}` (`:467-474`) | `Tasks::Ended(task)` | tests/core `world.rs:504-518`, `effects.rs:561-577` |
| `HistoricalEscalation{request, person, project, task, revision}` (`:266`) | `HistoricalEscalation{…, row: Option<EscalationDecisionRecord>, busy}` (`:476-484`) | `CoreKey::EscalationDecision{task, revision}` | **no world answers it**; test-domain routes it to `unreachable!` (`routes.rs:514`) |

- Ops routes all four to `Output::CoreLoad(Now)` (`examples/ops/domain/src/route.rs:1325-1330`).
- Hits: SeenChat 16 in 11 files; HistoricalProposal 17 in 14; HistoricalEscalation 12 in 10; HistoricalPersonTask 16 in 12; `CoreLoad` 18 in 6.
- **A fifth, half-built get:** `Event::DelegateValidated{to, key, batch, stubs: Box<[tasks::Stub]>}` (`vocabulary.rs:490`) is documented as "loaded and validated by the root's store transport". Ops always passes `stubs: []` (`examples/ops/domain/src/assemble.rs:476`), and the tasks child refuses non-empty inputs (`Refusal::Inputs`, `crates/jig-core-tasks/src/boundary.rs`, "no current root historical-input creation route"). It has 17 hits in 14 files.

### 1.6 Transcripts

- **Asking:**
  - `Now::PrepareAgent{context}`: both roots answer `Event::PreparedAgent{context, transcript_waiter: Some(Token(task)) if context.ever_turned, busy: false}` (ops `route.rs:1211-1214`, test-domain `routes.rs:363-366`). The "read slot" is never busy today;
  - the core then calls `begin_transcript(task, previous_attempt)` (`crates/jig-core/src/run.rs:136-158`), with `from = RunProof.transcript_from`, and emits `Now::StartPreparation{task, transcript_waiter}` (`routing/run_preparation.rs:322-339`, `vocabulary.rs:238`);
  - ops sends the `Some` case as `Output::CoreLoad` (`route.rs:1222-1228`); test-domain pushes it to `domain.now` (`routes.rs:375-380`).
- **Answering:**
  - ops: `Store::Transcript{task, rows: Box<[TurnRecord]>, done}` (`boundary.rs:111`), handled at `restart.rs:89-104`;
  - test-domain: `Event::TranscriptLoaded{task, rows, done}` (`lib.rs:262-266,545-558`);
  - each row goes through `core.append_transcript(task, TurnRecord)` (`run.rs:163-210`), and `done` triggers `StartBrief`.
- **The core reads every turn of the task, from attempt 1,** and skips turns of attempts below `from` (`run.rs:176-179`). There is no notion of a window: `TurnRecord` has no window field (`stored.rs:461-478`), the core keeps no window start (`RunProof.transcript_from` holds an attempt, `stored.rs:523`), and the host and fleet carry none (grep in `jig-host`, `jig-core-fleet`).
- **Turn input:** `Event::AcceptedTurn{task, attempt, turn, accepted, cumulative, read, transcript}` (`vocabulary.rs:437-445`; 16 hits in 11 files), saved as `CoreRecord::Turn(TurnRecord)` in `routing/runs.rs`. Turns and terminals are never erased; only `Call` and `RunProof` are (`routing/runs.rs:709`, `routing/tasks.rs:957-960`).
- **Worlds:** tests/ops scans all turns, sorts by (attempt, turn) and answers once with `done: true` (`conformance/mod.rs:547-565`). tests/core sends one `TranscriptLoaded` per row (`effects.rs:507-526`). tests/conformance answers once (`lib.rs:461-475`).
- Hits: `StartPreparation` 14 in 12 files; `transcript_waiter` 14 in 5; `PreparedAgent` 5 in 4; `Store::Transcript|TranscriptLoaded` 11 in 9; `TurnRecord` 19 in 12; `append_transcript|begin_transcript` 9 in 5.

### 1.7 What store.md 3's boundary table needs that does not exist

1. **Fill on `committed`.** It needs a route to the core, since skein-lib's journal takes only `committed(number)`.
2. **"Drop a task's transcript whole".**
3. **The transcript request's start turn:**
   - "a task's current window, from the turn the core names", "a turn by its task, attempt and number", "a page from either end";
   - today there is one load of every turn.
4. **Restore of "the application's intervals as bytes":**
   - both roots carry connector records typed: ops `Record::Observability/Infrastructure` (`store.rs:33-40`), test-domain `Record::Connector{number, record}` (`lib.rs:34-40`);
   - the record kinds are ops observability 6 (`examples/ops/domain-observability/src/boundary.rs:262-293`), infrastructure 8 (`domain-infrastructure/src/boundary.rs:393-431`) and the testing connector 12 (`testing/jig-test-connector/src/boundary.rs:600-655`);
   - all are derived-`Ord` keys with no encoder.
5. **The application's loads and gets.** No connector issues a store load or get today; the only path is the restart range.
6. **The secrets' requests.** There are none.

---

## 2. jig's families and keys against store.md 4.4

### 2.1 The core (`CoreKey`, `stored.rs:183-200`, in declaration order)

| # | Key | Record | 4.4 | Status |
|---|---|---|---|---|
| 0 | `Projection(ProjectionKey)` | `ProjectionRecord` | live | ✓ live. `ProjectionKey` is variant-first: `Goal(g) < Task{g,t} < Lifecycle{g,t,pos} < Revision{g,t,rev}` (`projections.rs:78-87`), so every Goal row precedes every Task row; restore_goal must precede its rows (`projections.rs:420-470`). 4.4 says "a goal's, then its own", which needs goal-first or stays variant-first; both satisfy the restore order |
| 1 | `Deployment` | `Deployment` | at start, first | **Out of place:** it sits between Projection and RunProof; 7.2 wants it first. Reorder it to variant 0, or point-get it |
| 2 | `RunProof(task)` | `RunProof` (`:517-534`) | live | ✓ |
| 3 | `Call(CallKey{task, attempt, completion, position})` | `CallRecord` | live | ✓ (4.4's key "task, attempt, completion, position" matches `:447-454`) |
| — | *(missing)* conversation | — | live, "task" | **Missing family.** engine.md 5.4 says "each live agent task's conversation: its first attempt, and where its current window starts". Today only `RunProof.transcript_from` (first attempt) exists, and there is no window start (15 item 4) |
| 4 | `Turn{task, attempt, turn}` | `TurnRecord` | on demand, in transcripts' files | It moves out of the map into frames plus the component's places. `TurnRecord` lacks "opens a window" |
| 5 | `Terminal{task, attempt}` | `TerminalRecord` | on demand | ✓ history after live |
| 6 | `ProposalDecision{proposer, proposal}` | `ProposalDecisionRecord` | on demand | ✓ |
| 7 | `EscalationDecision{task, revision}` | `EscalationDecisionRecord` | on demand | ✓ |

The core already has live (0-3) before history (4-7); only Deployment's place and the missing conversation family break 4.4.

### 2.2 The tasks child (`tasks::Key`, `crates/jig-core-tasks/src/boundary.rs:978-999`, in declaration order)

| # | Key | 4.4 group | Change needed |
|---|---|---|---|
| 0 | `Activity{task, position}` (`:980`) | history | move after the live families |
| 1 | `PersonProposal(n)` (`:982`) | **mixed** | split it into pending (live) and decided (history). One `Stored::PersonProposal` holds `Pending\|Accepted\|Rejected` (`proposals.rs:143-158`). Restore admits decided rows and **discards them** (`jig-core-tasks/src/stored.rs:709-725`), so every decided proposal is read at every start today |
| 2 | `History{task, revision}` (`:984`) | history | move after |
| 3 | `Milestone{task, position}` (`:986`) | history | move after |
| 4 | `Live(n)` (`:988`) | live | first |
| 5 | `Ended(n)` (`:990`) | history | **it sits between Live and Stub:** move after |
| 6 | `Stub(n)` (`:992`) | live ("stubs naming ended results", 4.4; 15 item 1 "stubs among these") | keep with the live families |
| 7 | `Ledger(Funder)` (`:994`) | **mixed** | split it into live and past periods. `FundingRecord.closed` (`funders.rs:367-385`) marks a past one, but closed rows **stay in `domain.funding` forever**: `retire` sets `closed = true` and saves (`funders.rs:633-683`), only `recurring.rs:131` removes, and `restore_funding` re-admits closed rows into the bounded table (`funders.rs:594-617`). Past periods therefore fill `funding`'s capacity across restarts. The domain change must evict closed ledgers from memory as well as re-key them |
| 8 | `Writer(n)` (`:996`) | live | ✓ |
| 9 | `Pool(n)` (`:998`) | live | ✓ |

- **Target order** (15 item 1): `Live, Stub, Ledger(live), Writer, Pool, PersonProposal(pending)` | `Ended, Ledger(past), PersonProposal(decided), History, Milestone, Activity` | then indexes (§3).
- `Funder` is `Task(n) | Pool{project, person, period} | Recurring{project, task, period} | Period{project, period}` (`value.rs:151-171`). A project's periods are therefore one interval within each Ledger family (Spend, §3).
- Blast radius: `tasks::Key::` 118 hits in 27 files; `Key::Ended` 59 in 29; `Key::PersonProposal` 9 in 7; `Key::Ledger` 23 in 11; `Stored::PersonProposal` 44 in 22; `Stored::Ledger` 72 in 32 (tasks world 34); `Stored::Ended` 111 in 52.
  - Reordering the variants alone touches no call site; it changes only the derived `Ord`.
  - The splits touch the counted sites.

### 2.3 The people child (`people::Key`, `crates/jig-core-people/src/boundary.rs:532-547`)

- The order is `Person(n), ReadResults{party}, ReadChat{party, task}, SignIn(n), Roles(project), Policy(project), Answer(RequestKey{person, key:[u8;16]})`. All of them are at start in 4.4, so one interval ✓.
- `Person` first matches ops's sort (§2.6).
- **Answered-key retention:** `Answer.at` + `Limits.request_retention` (`crates/jig-core-people/src/domain.rs:481`, `limits.rs:51`) exists. Chats are forgotten by `forget_chat` (`crates/jig-core/src/inbox.rs:461-490`).
- **No index rows yet:** 4.4's last row, "tasks, people: indexes", has no key variant in either child.
- `people::Key::` has 34 hits in 10 files.

### 2.4 The notes child (`notes::Key`, `crates/jig-core-notes/src/boundary.rs:80-86`)

- `Entry{name}` comes before `Line{scope: Scope, name}`.
- `Scope` is `Deployment | Project{p} | Goal{p, goal} | Resources{p, connector, pattern: Pattern{segments: Box<[Box<[u8]>]>, last: Exact|Open(bytes)}}` (`:8-33`). The key needs store.md 4.1's "list of byte strings" encoding.
- **Notes keep nothing at start:** `Restore` is a no-op (`notes domain.rs:136-139`). Yet ops routes notes rows into it at `LoadCore` (`examples/ops/domain/src/restart.rs:154-157`), and the ops world's `Range::Core` includes notes (`tests/ops/src/conformance/mod.rs:355-376`). The real store must not include notes in the core's live intervals (7.2: "notes' entries … never read at start").
- `notes::Key::` has 14 hits in 6 files.

### 2.5 The component's own rows

The format row and the transcripts' places (task; task, attempt) are new.

### 2.6 The restoration order: what the code needs, against 7.2

**Hard dependencies in code:**
1. The **deployment header first.** It sets the counters (`stored.rs:338-341`) that `restore_task_row` (`:277`), `restore_call` (`lib.rs:403`), `restore_people_row` (`application_routes.rs:159`) and `restore_goal` (`projections.rs:437`) validate against. The testing root rebuilds its journal from it (`jig-test-domain/src/lib.rs:1015-1018`).
2. **Live tasks before RunProof.** `restore_task_row` inserts `restoring_proofs` (`stored.rs:301-315`); `restore_proof` consumes it (`lib.rs:655-668`); `LoadCore` refuses with leftovers (`restart.rs:121-125`).
3. **RunProof before Call** (`lib.rs:409-412`).
4. **Projections after the deployment.** They need `authority.policy(project)`, which comes from config, and the connectors list (`projections.rs:437-461`). Goal rows come before their Task and Milestone rows.
5. **`people::Event::Restored` after the LoadCore range,** sent by the root (ops `restart.rs:80-85`) or the world (tests/core `effects.rs:1116-1118`). `LoadInbox` then requires `people.ready()` (`inbox.rs:420`).
6. **`tasks::Event::Restored` at AdoptRuns** (ops `restart.rs:38`), after the connectors restored resource roles.

**The worlds' sorts** that satisfy these today:
- tests/core `effects.rs:1083-1115` and tests/conformance `lib.rs:153-185`: Deployment 0; Tasks, People, Notes, Projection and connectors 1, in key order, so **Tasks before People**; RunProof 2; Call 3; archives skipped;
- ops `tests/ops/src/conformance/mod.rs:383-390`: Deployment 0; Person 1; People 2; Tasks 3; the rest of Core 4, including Turn, Terminal and the decisions, which the core answers `Archive`; connectors 5.

**Against 7.2** ("header first, then people's families, then the tasks child's live families, then the core's own live rows"): the order satisfies every hard dependency above. The core does **not** need people before tasks; the core world restores tasks first. The fixed interval list is:

1. `Core::Deployment`, a point read; 0 intervals if reordered first;
2. the People interval;
3. the Tasks live interval, one interval once reordered;
4. Core `Projection..=Call`, two intervals while Deployment stays at index 1; one interval plus Conversation after reorder.

Notes contribute nothing.

**Census (`LoadInbox`), 15 item 2:**
- Today it reads `Live`, `Ended` and `PersonProposal`, every row of each, via `inbox_restore_key` (`inbox.rs:397-414`), with `restore_inbox_row` (`:418-459`).
- Per row it computes: `remember_result` for an ended task with a person requester (`:441-450`); `ChatState` for chats (`:456-466`); `cache_archived_chat` (`:469`); waiting entries (`:471`); person-proposal entries (`:473`).
- Live rows are thus read twice, at LoadCore and at LoadInbox, and **every ended task is read at every start**.
- The index must yield what it needs:
  - ended tasks whose result or chat a person has not yet read, or not yet forgotten;
  - pending person proposals.
- Live rows can be recomputed from memory.

---

## 3. Each `reads::Scope` against store.md 7.3

- "Core expects" comes from `rows.rs` `position_allowed` (`:1427-1500`), `object` (`:538-826`) and `page_row` (`:828-1000`).
- "Fakes" means tests/core `reads.rs` `candidate` (`:228-327`) and `row` (`:177-219`), and tests/ops `joint/engine.rs` `read_page` (`:73-142`), which handles only `Proposal{Task}` and delegates the rest to tests/core.
- The fakes' `row()` join: owner, Live then Ended, by key (`reads.rs:166-175`); ancestry walked ≤64 (`:180-187`); `linked` = dependencies, spec.inputs, references, delegates and result_followups (`:188-199`); `related` = the `Turn` row for an activity `Turn` line, else the `Terminal{task, task.attempt}` row (`:200-212`).
- **Every fake page is a full scan of the map** (`reads.rs:331-338`).

| # | Scope | Core expects (row, joins) | Fake today | Store (7.3) | Index (7.4, 15.3) | Position in task record (15.7) |
|---|---|---|---|---|---|---|
| 1 | `Task{task}` | `Ended(task)`, owner, related `Terminal`, links (`rows.rs:540-597`) | scan for Ended (`reads.rs:230-236`) | get `Ended(task)` and `Terminal(task, attempt)` by key; links by key | — | — (`TaskRecord.attempt` names the terminal) |
| 2 | `PersonTask{task}` | same | same | same | — | — |
| 3 | `Result{task}` | same | same | same | — | — |
| 4 | `Proposal{Task(t), n}` | `History{t, rev}` whose `.proposal` is n (`rows.rs:601-613`) | core fake: none; **ops fake scans every History row** (`engine.rs:73-140`) | "the row of its proposer's history that keeps it" | — | **needs** the revision that keeps proposal n (15.7 "a task's proposal's row in its proposer's history"). `TaskRecord.proposal` names only the pending one (`boundary.rs:884`) |
| 4' | `Proposal{Party(p), n}` | `PersonProposal(n)` plus related `ProposalDecision{Party(p), n}` (`rows.rs:615-650`) | **none** | get both by key | — | — (after the split: the decided family's key) |
| 5 | `Escalation{task}` | newest `EscalationDecision{task, *}` (backwards), owner, related hold `Milestone` (`rows.rs:667-691`) | **none** | "its decision, by the revision its task's record names, with the hold's milestone" | — | the revision is in `TaskRecord.escalation` (`Escalation::{Unheld,Routing,Waiting,Rejected}{revision…}`, `jig-core-tasks/src/escalation.rs:151-180`); **the hold's milestone position is not named** |
| 6 | `Question{task, message}` | the Activity line `Words{kind: Question, message}`, related answer line `Answer{question}`, owner (`rows.rs:692-769`) | **none** | "the task's activity line, by its position, with the line that answers" | — | **needs both positions.** `QuestionCredit{number, answerer}` has none (`boundary.rs:198-201`), and the scope names a message number, not a position |
| 7 | `Reply{task, activity}` | `Activity{task, activity}`, related `TurnRecord` (`rows.rs:770-805`) | scan Activity (`reads.rs:244-250`), related `CoreKey::Turn` | get the activity by key; the turn from its transcript (8.3) | — | ✓ the pattern: `ReplyHint{number, activity, attempt, turn, at}` (`boundary.rs:832-838`) |
| 8 | `Turn{task, attempt, turn}` | `CoreRecord::Turn` plus owner (`rows.rs:806-814`) | scan Turn rows (`reads.rs:303-309`) | transcript, walking from the attempt's first turn (8.3) | — | — |
| 9 | `EndedDelegates{task}` | Ended rows with `requester == Task(task)` (`rows.rs:830-838`) | **none** | index | **ended delegates by requester** | — |
| 10 | `History{task, subtree}` | History and Milestone rows of the task, or of ancestors containing it (`rows.rs:839-868`) | **none** | History plus Milestone intervals: two families, so two intervals per task. Subtree by index | **a subtree's delegates** | — |
| 11 | `Activity{task}` | Activity rows, backwards, plus owner (`rows.rs:870-883`) | scan (`reads.rs:237-243`) | forward intervals computed from the last position (7.4) | — | ✓ `TaskRecord.activity` (`boundary.rs:869`) |
| 12 | `Inbox{Updates\|Results\|Replies}` | Ended results and activity replies, ordered by the virtual `Position::Inbox{at, task, object, key}` (`rows.rs:145-178,1056+`) | scan Ended and Activity (`reads.rs:251-290`) | index by time, newest first | **party's inbox** (also the census, §2.6) | — |
| 12' | `Inbox{Needs\|Proposals\|Escalations\|Questions\|PersonTasks}` | served live; `needs_store` is false (`rows.rs:107-111`) | none, never asked | live (7.3 "what needs them oldest first" from the index) | the same index | — |
| 13 | `Conversations` | Ended rows: requester `Person(party)`, untracked, not recurring, agent executor (`rows.rs:972-989`); cursors mix live and stored (`vocabulary.rs:115-120`) | scan every Ended (`reads.rs:223-226,302`) | index newest first | **party's conversations** | — |
| 14 | `Goals{project}` | ended tracked goals, plus `linked` subtree rows for phase counts (`rows.rs:923-971`) | **none** | index, live then ended newest first | **project's goals** | — (**gap:** phase counts need the ended goal's *whole subtree* joined, which 7.3's join list does not price; §8 Q2) |
| 15 | `Notes{project}` | `notes::Entry` whose scope's project matches (`rows.rs:896-910`) | scan every entry, no joins (`reads.rs:291-301`) | the lines of the project's scopes, each with its entry by name: 3 intervals, `Line{Project{p}}`, `Line{Goal{p,*}}` and `Line{Resources{p,*}}`, because Scope is variant-first | — | — |
| 16 | `ScopedNotes{scope, description}` | entries of that scope, description substring (`rows.rs:884-895`) | same scan | one scope's lines, filtered on the line's description, plus entries by name | — | — |
| 17 | `Spend{project}` | `Ledger(Period{project, *})` rows (`rows.rs:911-922`) | **none** | "the funders' periods of the project": one interval per Ledger family, so 2 after the live/past split | — | — |

- **No fake answer (9):** Proposal{Party}; Proposal{Task} in tests/core (ops has one); Escalation; Question; EndedDelegates; History; Goals; Spend; and the inbox Needs groups, which are never asked.
- **Need an index (6):** Inbox (plus census), Conversations, Goals, EndedDelegates, History{subtree}, and Activity's "reading backwards" (computed from `activity`, no index).
- **Need positions in the task record (3):** Question (the question's and answer's positions), Escalation (the hold's milestone), and Proposal{Task} (the history revision). Reply already has `ReplyHint`.

---

## 4. The fake store and the worlds

### 4.1 `testing/jig-fake-store`

`src/lib.rs` has 235 lines and `tests.rs` 94. It is generic over opaque `K: Ord+Clone, R: Clone` and never decodes.

- **Types:**
  - `Write<K,R>{Save{key,row} | Erase(K)}` (`:18`);
  - `Pending{number, writes, delay}` (`:27`);
  - `Page{rows, next: Option<K>, delay}` (`:38`), where `next` is the last key returned (exclusive);
  - `Fault{Hold{commits} | Fail{commit} | SlowPages{by}}` (`:49`);
  - `Store{pub applied, pub rows: BTreeMap<K,R>, pub pending, pub held, pub fail_at, pages, hold_next, page_delay, failed}` (`:60`).
- **Methods:**
  - `new` (96), `fault` (101);
  - `submit` (110), which asserts the next number;
  - `tick` (123), which applies a whole transaction;
  - `release_held` (158);
  - `crash` (164-171), which clears flights and keeps `rows`, `applied` and `page_delay`;
  - `restore` (174, 0 users), `page` (180, 0 direct users), `load` (208), `tick_pages` (216), `loading` (232).
- **Fault semantics:**
  - `Hold` withholds completions after apply, so the commit is durable but unacknowledged (147-154);
  - `Fail` fails before apply and asserts on any later submit (111, 130-134);
  - `SlowPages` adds ticks to later pages (203).
- **Dependents (4):** `testing/jig-conformance/Cargo.toml:10`, `tests/core/Cargo.toml:12`, `tests/conformance/Cargo.toml:18`, `tests/ops/Cargo.toml:22`. `jig_fake_store` has 48 hits in 30 files.
  - No child world uses it, nor tests/client, scripted-engine, test-system or ops-fake-production.
  - The children's worlds keep their own maps: tasks `tests/tasks/src/world.rs:597-600,976-1005`; people `tests/people/src/world.rs:845-848,214`; notes `tests/notes/src/world.rs:123-134,150-157,201-230`; test-connector `tests/test-connector/src/world.rs:268-296`; ops connectors `tests/ops/src/world.rs:261-288`, `infrastructure_world.rs:345-353`.
- **Call counts:**

| call | tests/core | tests/conformance | tests/ops | testing/jig-conformance |
|---|---|---|---|---|
| `store.rows` | 178 (src 43, tests 135) | 12 | 19 | 0 |
| `store.applied` | 132 | 9 | 20 | 7 |
| `store.fault(` (Hold / Fail / SlowPages) | 37 (30 / 4 / 4) | 5 (3 / 1 / 1) | 2 (2 / 0 / 0) | 0 |
| `submit` / `tick` | 11 / 9 | 0 | 0 | 1 / 1 |
| `load` / `tick_pages` | 2 / 1 | 2 / 1 | 0 | 0 |
| `release_held` | 11 | 0 | 0 | 1 |
| `crash(` | 3 | 0 | 0 | 2 |

### 4.2 How each world wires the store, and its own joins

- **`testing/jig-conformance` `Harness`** (`harness.rs`, 533 lines):
  - `store` (`:153`); the `Application` trait passes `&Store` / `&mut Store` to `start`, `cold`, `inbound`, `deliver`, `poll`, `submitted` and `observed` (`:43-127`);
  - commits go `submit(n,writes,0)` (`:299`) then `tick(false)` (`:317`);
  - cuts `AfterCommit` / `Random` (`:323-338`); `crash` (`:247-258`);
  - `cold_snapshot` clones the store (`:474-480`); `from_snapshot` (`:495-532`).
- **tests/core `effects::World`** (`effects.rs`, 1289 lines):
  - commits `submit(n, store_writes(rows), store_delay)` (`:499`), ticked per drain (`:609-613`);
  - restart `restart_step` / `load_restart` / `restore_pages` (`:1003-1067`, 2 rows per page, `hold_restore_pages` `:61`), and `restored` with the rank sort (`:1074-1120`);
  - point reads (§1.5), transcripts (`:507-526`), notes (`:796-820`), party reads (`:689-695`) via `reads::Parties` (`reads.rs:126-143`, `hold_loads`);
  - crash `restart_with` (`:1161-1189`).
- **tests/core `world::World`** (810 lines): it commits and ticks at once (`:486-495`) and uses restored markers.
- **tests/conformance `Testing`** (`lib.rs`, 665 lines): restart (`:193-233`), sort (`:146-191`), transcripts (`:461-475`), notes (`:554-576`). **It answers no party reads and no historical reads** (`:477`, `:586`).
- **tests/ops `Ops`** (`conformance/mod.rs`, 975 lines):
  - `Output::Load` answered synchronously, filtered and sorted (`:341-392`), with no pages and no faults;
  - notes answered empty (`:540-546`); transcripts (`:547-565`).
- **tests/ops `joint/engine.rs`** (455 lines): `core_store` re-keys ops rows into a testing-root store (`:45-61`); `read_page` (`:73-142`); SeenChat (`:234-241`).
- **Joins in world code** (they move into the component): `reads.rs:166-219` (owner, ancestry, links, related), `engine.rs:63-71,84-108` (owner, ancestry), and `effects.rs` / `world.rs` point reads.
- **Direct row mutation the real store will not allow:**
  - fixtures inserted without a commit: `tests/conformance/src/lib.rs:376`, `tests/ops/src/conformance/mod.rs:213`, `tests/core/src/effects.rs:336`, `tests/core/src/stories/reads.rs:153`;
  - record surgery: `tests/core/tests/restart.rs:115,159`, `tests/core/tests/store_peer.rs:126`.

### 4.3 What the move to the real store touches

- **Answer paths that move into the component:**
  - `effects.rs:499-626,689-695,796-820,1003-1120`;
  - `world.rs:486-595`;
  - `reads.rs:166-372`;
  - `historical.rs`;
  - `tests/conformance/src/lib.rs:146-233,461-475,554-576,600-610`;
  - `tests/ops/src/conformance/mod.rs:341-392,540-565`;
  - `joint/engine.rs:45-142,234-246`.

  The three rank sorts (`effects.rs:1083-1115`, `conformance/src/lib.rs:153-185`, `ops conformance/mod.rs:383-390`) disappear.
- **Faults move between the root and the component** (store.md 14):

  | Today | Becomes |
  |---|---|
  | `Hold` (applied, unacknowledged; a crash keeps it) | hold the component's `committed n` before the root |
  | `store_delay` / `Pending.delay` (a crash loses it) | hold commits before they reach the component |
  | `Fail` | the world answers `failed n` |
  | `SlowPages` / `hold_restore_pages` | delay the component's same-step answers |

- **Crash:** keep both `skein_kv::Store::memory` values and the file stand-in, drop what the world held, and rebuild the component over them.
  - That needs a constructor taking existing stores. `Store::memory` cannot be closed and reopened: `Close` sets `recovered = false` and everything after is `Unavailable` (skein `crates/skein-kv/src/store.rs:584-588`). It is not `Clone` (`:108`).
  - The harness's `cold_snapshot` / `from_snapshot` therefore needs either `Clone` from skein or a replay of the durable commits.
- **Typed views:** 135 test assertions, plus the observers (`tests/core/src/observations.rs`, `store_peer.rs:37-43`, `peers.rs:430`, ops `observations::snapshot`), read `store.rows` as typed values.
  - The cheapest path is a typed mirror updated on each `committed`.
  - `application_observations.rs` (84 lines) and ops `conformance/application.rs` (82) are near-copies reading `store.pending.back()`.
- **Application records as bytes:** the testing connector (12 kinds) and the ops connectors (6 + 8) need key encoders and codecs before either world can commit through the component (store.md 12, 15 "the application owes").

### 4.4 Suite cost

- **Gate:** focused 2173 tests in 9.0–10.6 s against a 15 s budget; fuzzy 86 tests in 12.0–14.0 s against 60 s (`.config/nextest.toml:7-21`; gate records in `/srv/data/git/runner/src/tmp/gates/jig/`).
- **Per binary,** from today's `scratchpad/times.tsv` (nextest per-binary totals at test-thread parallelism):

| World | Focused | Fuzzy |
|---|---|---|
| jig-core-world | 134 tests / 5.64 s | 8 / 5.78 s |
| jig-conformance-world | 51 / 4.78 s | 10 / 15.74 s |
| jig-ops-world | 46 / 2.81 s | 22 / 27.36 s |

  - The slowest fuzzy tests are ops `fuzzy_joint` (11 tests, 15.66 s, max 2.0 s), ops `fuzzy_client` (2 / 9.23 s), conformance `fuzzy_snapshot` (8 / 9.95 s) and conformance `fuzzy_scenarios` (1 / 4.49 s).
  - The serial `-j 1` measure of Oct 11 (`plans/jig-client-implementation/logs/45/referee-workspace-measure-*.log`): core 1.79 s / 1.65 s; conformance 1.56 s / 4.45 s; ops 0.80 s / 4.13 s.
- **Cost risk:** conformance and the ops alert stories replay whole stories once per cut, which is quadratic in commits (`docs/development/tech-debt.md:86-89`). Per-commit encode and decode cost multiplies there; the fuzzy budget (60 s) has about 46 s of headroom.

---

## 5. skein-kv and io as the component needs them

- **Dependencies:** jig's lock has `skein-io`, `skein-sim` and `skein-codec` transitively. It has no `skein-kv` and no `skein-fake-machine`, so jig's `Cargo.toml` needs new `[workspace.dependencies]` rows for both.
- **`Request`** has 5 variants (`crates/skein-kv/src/boundary.rs:60-66`): `Open{owner, root: Fd}`, `Commit{owner, ops: Box<[Op]>}`, `Get{owner, key}`, `Load{owner, range, max: Page}`, `Close{owner}`.
- **`Op`** is `Put{key, value} | Erase{key}`; within a commit only a key's last op counts (`map.rs:75-78,105-110`).
- **`Range{start, end: Option}`:** start inclusive, end exclusive (`map.rs:127-131`). `Range::prefix` (`boundary.rs:91-102`). `end < start` gives `Refused TooLarge`.
- **`Page{rows: u32, bytes: u32}`** and **`Row{key, value}`.**
- **`Event`** has 10 variants (`:118-159`): `Opened{owner, last}`, `Committed{owner, number}` (**no fill**), `Refused{owner, refusal}`, `Failed{owner, number}`, `Got{owner, value}`, `Loaded{owner, rows, next}`, `Closed`, `OpenFailed{owner, failure}`, `Recovered{last}`, `RecoveryFailed{failure}`.
  - `Refusal = TooLarge|Full|Busy|Unavailable`; `Failure = Io|Corrupt|Full`.
- **`Loaded.next`** is the **first key not returned** (inclusive), and `None` when the range is exhausted (`map.rs:123-146`).
  - The component can pass `next` straight back. Store.md 7.1's "cursor + 0x00" also works.
  - **A first row larger than `max.bytes` is `Refused TooLarge`, not a short page** (`tests.rs:118-152`). The 5.4 size check must require `page.bytes ≥ largest key + largest value`.
  - `0 < rows ≤ limits.page.rows` and `0 < bytes ≤ limits.page.bytes` (`store.rs:543-546`).
- **`Limits`** has 12 fields (`limits.rs:5-18`): `key, value, ops, commit: u32; queued: u32; queued_bytes: u64; budget: u64; segment: u64; snapshot_after: u64; chunk: u32; page: Page; deadline: Duration`.
  - There are no defaults. `is_usable()` (`:22-38`) requires `commit ≥ 26`, `0 < queued ≤ 32` and `chunk ≥ key+value+20`.
  - `worst_case(&Limits) -> Option<u64>` (`:42-60`) is about 3× budget plus the queue.
- **`KeyWriter`** (`key.rs:8-52`): `new(limit)`, `tag(u8)`, `u64` (big-endian), `rev_u64`, `bytes` (0x00 becomes `00 ff`, terminated by `00 00`), `finish() -> Option<Box<[u8]>>` (None past the limit). `KeyReader` (`:55-112`) mirrors it.
  - **There is no u8/u16/u32, no signed integer, no option or list method.** Store.md 4.1's rules are built from these: widen narrower numbers to `u64`; `tag(0|1)` for an option or a variant index; `tag(1)+bytes` per list item and `tag(0)` to end. jig's keys have no signed fields.
- **`Store::memory(&Limits)`** (`store.rs:162-168`) needs no `Open`. `Commit` is applied and `Committed` pushed in the same `down` (`:595-625`), and it never answers `Failed` or `Busy`.
  - The last commit is `Store::last()` (`:171`); the fill is `Store::used()` (`:176`), charged at `128 + key + value` per entry against `budget` (`map.rs:15,79-94`).
  - `Close` makes it unusable for good (`:584-588`). It is not `Clone`.
- **Numbers** are assigned by skein-kv (`last + active + queued + 1`, `store.rs:230-233`); the caller cannot supply them. A refusal consumes none. They are contiguous across restarts. Store.md 6's equality check is the component's.
- **Group commit:** a queue of ≤ `queued` (≤ 32) entries and `queued_bytes`; past it, `Refused Busy` (`:490-493,531-534`). In flight it can reach about 2×`queued` (active plus queue).
- **Failure** fails every active and queued commit, then the store **recovers by itself** and emits `Recovered{last}` (`:193-200,295-310`). The component must stop, not resume (store.md 6).
- **I/O:** skein-kv emits `skein_io::file::Request` itself, one outstanding, with the fixed owner `Token(0x4b56)` (`store.rs:25,634`). **Each store needs its own `FileIo`** (kv.md 12), so there are 3 drivers (store.md 13).
  - It issues Close, WriteAt, ReadAt, Remove, SyncDirectory, Create, OpenRead, Sync, List and Rename.
  - The owner must call `fire` when `is_ready` (`:872-881`).
- **io files** (`crates/skein-io/src/file.rs:31-147`, 19 variants):
  - `MakeDirectory{root: Token, path, mode}` → `Made`;
  - `OpenPrivate{root: Token, path}` → `Opened{file, len: 0}`, or `Refused{Unsafe}` (`file_layer/private.rs:194`);
  - `Create{root: Fd, name, mode}` is **`O_WRONLY|O_CREAT|O_EXCL`** (`kernel.rs:603-607`), not readable, and `Exists` if present;
  - `OpenRead{root: Fd, name}` is read-only;
  - `WriteAt{file, offset, bytes}` → `Written`; `ReadAt{file, offset, max: u32}` → `Read{bytes}`;
  - `Sync{file}`, `SyncDirectory{root: Fd}`;
  - `Remove{root: Fd, name}`: **a missing name is `Failed NotFound`** (`kernel.rs:792-795`), which store.md 8.5 must treat as success;
  - `Close`, `List`, `Rename`, `Stat`.
  - `FileIo::new(files, max_read, max_entries, timeout)` (`file_layer.rs:292`) runs one request at a time. `adopt_root(Fd) -> Token` (`:343`) and `descriptor(token)` (`:349`): the secrets store goes `OpenPrivate` → token → `descriptor()` → `skein_kv::Request::Open{root: Fd}`.
- **`OpenWrite` / `OpenWriteNoFollow{root: Fd, name}` and `Cut{file, length}` are designed, not built:**
  - vocabulary `io.md:553,557`; semantics `io.md:590-597`; listed as remaining at `io.md:993-994`;
  - absent from `file.rs` and from `OpenHow` (`kernel.rs:591-610`).
  - Building them touches the kernel records, the ring adapter, the simulator, the fake machine (`fs.rs` has no truncate) and the conformance suite.
  - **Until they land,** a transcript can be appended only while it stays open from `Create`. Reopening one to append after a restart, and the 8.4 cut, are impossible.
- **The crashing disk:** `skein-fake-machine::Machine` (`testing/skein-fake-machine/src/fs.rs`): `crash(seed)` (`:487-535`), `reopen_root` (`:538`), `serve` (`face.rs:109`).
  - Driven by `skein_sim::{Sim, Faults}`; skein-world has `Cut::{Kill, PowerLoss}` (`skein-world/src/world.rs:44-49`) and `files::Driver` (`files.rs:24-61`).
  - **The pattern to copy:** `tests/kv/tests/store.rs`: World `:33-46`, step `:103-130`, crash `:132-149`, and the referee `crash_at_each_commit_operation_keeps_whole_prefix` `:207-236`. The memory model is `tests/kv/tests/fuzzy_map.rs`; the memory budget is `tests/kv/tests/memory.rs:27-61`.
- **No in-memory file stand-in exists in skein:** none answers `file::Request` without `Sim` + `Machine`. jig writes it (store.md 14). `Machine`'s direct API (`fs.rs:359-700`) speaks `How`/`Opened`, and also lacks truncate and open-for-write.

---

## 6. Codecs: how skein's generator is used

- **The generator** is `skein/testing/skein-codegen`, with a lib and a bin: `parse(&str) -> Schema`, `generate(&Schema) -> Output{rust, goldens}` (`src/lib.rs:21,96,133`).
  - The bin `skein-codegen <schema> <out-dir>` writes `<out>/v<N>.rs` and `<crate>/golden/v<N>/*.bin` (`src/main.rs:10-27`).
  - The runtime is `skein-codec` (`crates/skein-codec/src/lib.rs`): `Reason{Short,Bound,Tag,Bool,Utf8,Version,Trailing}`, `read_len`, `read_count`, `read_text`, `version_of`.
- **Best example: smith's `smith-transcript`:**
  - schema `crates/smith-transcript/schema/transcript-v3.schema`, with header `family smith-transcript 3`, `record`/`enum`, and one `versioned record Turn`;
  - generated `src/generated/v3.rs`, checked in and included as `#[rustfmt::skip] #[path = "generated/v3.rs"] pub mod v3;` (`src/lib.rs:11-24`); its crate depends on `skein-lib` and `skein-codec` only;
  - the version is a leading big-endian `u16`, checked at `v3.rs:6102-6103`; smith peeks `[0,3]` to map it to its own `Refusal::Version` (`smith-protocol-channel/src/transcript.rs:95-97`);
  - the hand-written translations are `encode_turn` (`smith-protocol-channel/src/turn.rs:17-414`) and `decode_turn`/`translate_turn` (`transcript.rs:77,116`), with exhaustive matches. **They are fallible, not total:** encode fails on bounds.
  - jig's own example is `crates/jig-charter/src/encode.rs`, which builds `smith_charter::v2`.
- **Golden tests:**
  - generated inline in `vN.rs`: `record_<x>_{smallest,full}`, `enum_<e>_<v>_*` (`v3.rs:7410+`);
  - files `golden/vN/*.bin`;
  - a drift test regenerates and compares. Use skein's form, `skein/tests/codec/tests/drift.rs` (`SKEIN_CODEGEN_WRITE=1` rewrites); smith's `tests/codecs/tests/drift.rs:7-47` has no write mode.
- **Bounds:** generated `bound_<record>_<field>` tests, at the ceiling and one past it.
- **Fuzzing:** there is **no cargo-fuzz** in skein or smith. Deterministic `fuzzy_decode.rs` (smith `tests/codecs/tests/fuzzy_decode.rs:7-90`) runs 10k random inputs plus 100 mutations per golden, checking that decoding never panics and that decode then encode reproduces the bytes. jig's nextest `binary(/^fuzzy_/)` already gates such binaries.
- **Memory:** `tests/codecs/tests/memory.rs` uses `skein_heap::Counting` against `worst_case_heap`.
- **Layout:** the codec crate holds the generated code and its inline tests; a world crate (`smith/tests/codecs`) holds drift, fuzzy and memory tests.
- **Reaching it from jig:** jig depends on neither skein-codec nor skein-codegen directly, and has no `.schema` file. Add `skein-codec`, `skein-codegen` and `skein-heap` as smith does (`smith/Cargo.toml:16-19`). skein-kv uses no generated codec.
- **Generator limits that bite jig's records:**
  - **no `secret`** (`parse.rs:111-127`), so the secrets' digests are `fixed N`;
  - **no signed integer and no time type** (u8–u64, bool, duration), so `Wall` is carried as `u64`;
  - **no maps or sets**, only `list N T` with uniqueness checked by hand;
  - **an enum variant holds one record or nothing** (`check.rs:181`, ≤256 variants), so every jig variant with fields needs a record of its own;
  - no recursion;
  - no imports across families;
  - each type's worst case fits a `u32`.
- **What that means for the translations:** `people::Ask` has 24 variants and `Outcome` 29 (`jig-core-people/src/boundary.rs`). `CallPart` has 26 (`stored.rs:20-71`). The tasks child's durable graph is about 70 types (`jig-core-tasks/src/boundary.rs` 66 pub types, `value.rs` 12, `control.rs` 8, `proposals.rs` 9). The hand-written translations are the bulk of the codec work (§7).

---

## 7. Sizing: a proposed cut into increments (≤ about 600 production lines each)

- **Line counts:** generated `vN.rs` and goldens are not counted as production lines (they are checked by drift); the plan's README must say so. Test-only crates (stand-ins, worlds) are counted as test code.
- **"Needs"** names capabilities.

### 7.1 Domain changes (store.md 15), landed on the fake store first

| Id | Increment | Files (shared hotspots in bold) | Size | Needs |
|---|---|---|---|---|
| D1 | **Live before history.** Reorder `tasks::Key` (Live, Stub, Ledger, Writer, Pool, pending PersonProposal; then Ended, past Ledger, decided PersonProposal, History, Milestone, Activity). Split `PersonProposal` into pending and decided (Key and Stored). Split `Ledger` into live and past, evicting closed ledgers from `domain.funding` and refusing closed ones at restore. `CoreKey::Deployment` first | `jig-core-tasks/src/{boundary.rs:978-1040, funders.rs:594-683, stored.rs:703-760, proposals.rs}`, `jig-core/src/{stored.rs, reads/rows.rs:1512-1546, inbox.rs:397-414}`, tests/tasks (Ledger 34 hits) | ~300 | nothing |
| D2 | **Turns as the store keeps them** (15.4): a `window` flag from the host's turn through fleet into `AcceptedTurn`/`TurnRecord`; the core keeps the current window start per live task (in `RunProof` or a new `CoreKey::Conversation`); `StartPreparation` names the window's first turn `{task, attempt, turn}`; `append_transcript` reads from the window; a `Write` variant to drop a transcript whole | `jig-host` (`turns.rs`), `jig-core-fleet`, **`jig-core/src/routing/vocabulary.rs`**, `run.rs:136-210`, `routing/run_preparation.rs:75-100,322-339`, `stored.rs`; both roots' Transcript events | ~350 | smith's turn carries "opens a window" (smith `protocol/transcript.md` 3) |
| D3 | **Keys for single reads** (15.7): `QuestionCredit` gains the question line's position and the answer's; the escalation's hold milestone position on the task; a task proposal's history revision kept by number | `jig-core-tasks/src/{boundary.rs:198-201, escalation.rs:151-180, proposals.rs, control.rs}`, `jig-core/src/reads/rows.rs:601-760` | ~200 | D1 |
| D4 | **The inbox index plus a census from it** (15.2, 15.3 part): a tasks-child index family keyed by party and time (`rev_u64` for newest first), saved and erased with its records; `inbox_restore_key` / `restore_inbox_row` read the index; `Position::Inbox` becomes the index key | `jig-core-tasks` (new `index.rs`), **`jig-core/src/inbox.rs:397-490`**, `reads/vocabulary.rs:101-106`, `reads/rows.rs:145-178,1404-1426` | ~500 | D1 |
| D5 | **Other indexes** (15.3): a party's conversations; a project's goals (live, then ended newest first); ended delegates by requester; a subtree's delegates for history. Decide Goals' phase counts (§8 Q2) | `jig-core-tasks` index module, `jig-core/src/reads/rows.rs:828-1000` | ~500 | D4 (the index pattern) |
| D6 | **Fill** (15.5): a fill event from the root to the core with each `committed`; refusal above the high-water mark at the entrances (new tasks and chats, new sign-ins) | **`jig-core/src/routing/vocabulary.rs`**, people and tasks admissions, both roots' `Store::Committed` | ~250 | nothing |
| D7 | **Retention** (15.6): ended-task summaries (a new family; one commit erases a task's records and saves its summary); transcripts dropped past their horizon; answered keys expired (people has `request_retention`, `domain.rs:481`); timers | `jig-core-tasks`, `jig-core`, `jig-core-people` | 2 × ~400 | D1, D2 (drop write), D4 (index rows erased with summaries) |

### 7.2 The component (`crates/jig-protocol-store`, new)

| Id | Increment | Size | Needs |
|---|---|---|---|
| S1 | Crate and keys (4.1, 4.2): tags 0/1/2 and child tags; encoders and decoders for `CoreKey`, `tasks::Key`, `people::Key` and `notes::Key` (Scope and Pattern as lists of byte strings) and the component's rows; property tests against the derived `Ord`. Moves `core_key` (duplicated in `examples/ops/domain/src/store.rs:60-85` and `testing/jig-test-domain/src/lib.rs:52-79`) to jig-core | ~500 | D1; skein-kv dependency row |
| S2 | Ranges to intervals (4.3): the restart's fixed list (7.2: Deployment, People, Tasks-live, Core-live), notes' `Entry`/`Lines`, the 17 scopes' interval lists (§3); predicate tests | ~350 | S1; D4/D5 for the index scopes, or stubbed until then |
| S3 | Codec: core family (Deployment, ProjectionRecord, CallRecord with `CallPart` 26 variants and `SettledCall`, RunProof, Conversation, Terminal, both decisions); schema, generated v1, translations, goldens, bounds | ~600 | D1, D2; skein-codec/codegen rows |
| S4 | Codec: tasks live (`TaskRecord` and its graph: Spec, Contract, Authority, Numbers, Funder, Phase, Holding, Wake, Subscription, Proposal/ProposalAction, Escalation, RecurringState, PersonAnswer, Revisions, Tries, ReplyHint, QuestionCredit, Word; Stub, WriterSlot, PoolSlots, FundingRecord, pending PersonProposal) | ~600, likely 2 × 450 | D1, D3 |
| S5 | Codec: tasks history (History with AmendmentHistory/Settings, Milestone, ActivityLine/Line, decided PersonProposal, past Ledger) plus the index rows | ~450 | D1, D4, D5 |
| S6 | Codec: people (Person/Identity, SignIn, Roles/Holding, Policy/PolicyValue, Answer with `Ask` 24 and `Outcome` 29 variants, ReadResults, ReadChat) | ~600 | nothing beyond S1 |
| S7 | Codec: notes plus the component's own rows; the format row (5.3) and the sizes check against skein-kv's limits (5.4, including `page.bytes ≥ key+value`); the drift and fuzzy world `tests/store-codecs` | ~400 | S3–S6 |
| S8 | Commits (6): boundary types (commit, committed with fill, failed), writes to ops, the number check, the header against `last()` at open, the group-commit bound against `queued`/`queued_bytes`, stop on failure and ignore `Recovered`; over `Store::memory` | ~500 | S1, S3–S7, D6 |
| S9 | Restart loads (7.1, 7.2, 10): open (format, header, run proofs' lengths, with the cut deferred to S14), restore steps paged and typed, the application's intervals as bytes, the census from the index; both roots' restore paths aligned (ops `Store::Restore`, test-domain `restore_record`) | ~500 | S2, S8, D4 |
| S10 | Child loads and gets (7.1): pages under both bounds, cursor handling, `NotesLoad`, the four historical gets, `SeenChat`, delegate-input stubs; a row that does not decode stops | ~350 | S8 |
| S11 | Party reads: point scopes and joins (Task, PersonTask, Result, Proposal ×2, Escalation, Question, Reply, Turn; owner, ancestry ≤ depth, links, related); priced within bounds | ~600 | S10, D3 |
| S12 | Party reads: paged scopes (Activity backwards from `activity`, History, EndedDelegates, Inbox, Conversations, Goals, Notes, ScopedNotes, Spend) | ~600 | S11, D4, D5 |
| S13 | Transcripts (a): files and frames (8.1; checksum, lengths at both ends), the places family, write-before-commit with the queue behind syncs (8.2), and `committed` only after skein-kv | ~500 | S8, D2; **skein: `OpenWrite`** for appends after restart (with `Create` alone, only new files work) |
| S14 | Transcripts (b): read back (window, a turn by walking from the attempt's first frame, pages either way, the tail), the cut at open (8.4), drop (8.5; `NotFound` is success), the bounded open-file set | ~500 | S13, D7 (drop write); **skein: `Cut`** |
| S15 | Secrets' store (9): the second store over `OpenPrivate` → `descriptor` → `Open{root}`; keep, erase and get by owner and number; the sweep after people restore; read whole at start | ~350 | S8; the client component's sign-in path (another session) to consume it |

### 7.3 The worlds' move and the store's world

| Id | Increment | Size | Needs |
|---|---|---|---|
| W0 | The application's keys and codecs: the testing connector's 12 records (`testing/jig-test-connector`) and ops's 6 + 8 (`examples/ops/domain-{observability,infrastructure}`), so the worlds commit bytes (store.md 12) | ~600 (ops, production) plus testing | S1's rule; §8 Q1 |
| W1 | `testing/jig-store-stand-in` (test code): an in-memory `file::Request` answerer with the crash rule; a world adapter (component over `Store::memory` plus the stand-in, holds between root and component, a typed mirror on `committed`, crash keeping both stores); `testing/jig-conformance` `Harness` over it (`harness.rs:33,153,247-338,474-532`) | test ~600 | S8–S14; §8 Q4 (snapshots) |
| W2 | tests/core onto it: `effects.rs`, `world.rs`, `reads.rs` and `historical.rs` answer paths removed; fixture inserts become first commits (`effects.rs:336`, `stories/reads.rs:153`); surgery helpers (`tests/restart.rs:115,159`) | test, −1000 / +400 | W0, W1 |
| W3 | tests/conformance and tests/ops onto it; the rank sorts removed; `jig-fake-store` and its 4 Cargo rows deleted; check the suite budgets (fuzzy headroom about 46 s, §4.4) | test | W2 |
| W4 | `tests/store` on the crashing disk (14): the component over skein-kv files and transcripts on `skein-fake-machine`, a scripted root, a crash at every file operation and sync; the referee copies `skein/tests/kv/tests/store.rs:207-236` plus "no turn past its commit"; pages against a `BTreeMap` model (`skein/tests/kv/tests/fuzzy_map.rs`) | test ~700 | S13, S14; skein's `OpenWrite`/`Cut` in the fake machine |

**Critical path:** D1 → S1 → S3–S7 → S8 → S9/S10 → S11/S12 → W1 → W2 → W3. Running alongside it:
- D2 → S13 → S14 → W4, gated by skein's io;
- D4/D5 → S12;
- S15 alone.

There are about 30 increments in all. The codecs (S3–S7, about 2,700 hand-written lines) are the largest single block; plan them as their own session.

---

## 8. Questions and gaps for the designer (observable or long-lived)

1. **The application's records in the domain worlds.**
   - store.md 14 says "every story runs the real keys, codecs, intervals, paging and joins", but the worlds also commit connector records: 12 testing-connector kinds and 14 ops kinds, all typed today.
   - Do the testing application and ops ship real key encoders and codecs (store.md 12) before W2 and W3, or may a world encode them with a test codec?
   - Ops is the reference application, so its codecs are probably owed.
2. **Goals' phase counts.** The `Goals` page counts the ended goal's subtree phases from `Row.linked` (`reads/rows.rs:923-971`), but 7.3's joins (owner, ancestry, links, related) do not price a whole subtree. Should the ended goal (or its index row) keep its counts?
3. **Escalation reads "newest decision" without a revision** (`Scope::Escalation{task}`, backwards). With 7.3 "by the revision its task's record names", does the read always show the latest decision only?
4. **The memory store cannot be cloned or reopened** (skein `store.rs:108,584-588`), but the conformance harness snapshots the store (`harness.rs:474-532`). Is the answer `Clone` (or a snapshot) in skein-kv, or a replay of the durable commits in the world?
5. **Fill's route.** skein-kv's `Committed` has no fill, and skein-lib's `Journal::committed(number)` takes only a number. The component reads `used()`. The fill reaches the core as what, a separate event? (store.md 11.2 attributes "fill with each commit's answer" to kv.md 7, which does not say it.)
6. **Delegate historical inputs.** `DelegateValidated.stubs` is "loaded … by the root's store transport", but no route exists (ops passes `[]`, tasks refuses inputs). Is it in this plan's scope, as a "get" of `Ended` rows?
7. **`HistoricalEscalation` and `SeenChat` hit `unreachable!` in the testing root** (`jig-test-domain/src/routes.rs:513-517`), and no world answers `HistoricalEscalation`. The real store's get should be exercised by a story.
8. **A decided proposal's read when its `ProposalDecision` row is missing** answers `Gone` (`rows.rs:632-635`). Once decided proposals move to a history family, retention (D7) decides when `Gone` appears. That is horizons only, already in the design.
