# Session 11: the store's domain debts: live before history, keys for single reads, fill

jig's domain pays three of the debts `protocol/store.md` section 15 names, so
that the store (jig 13 to 18) can serve it by intervals and point reads: each
child's keys put what a restart loads before history, with decided person
proposals and closed funding periods in families of their own and the core's
deployment header first, and the core names its restart's live ranges as
predicates; a task's record names the activity positions, the hold's
milestone and the history revisions that single reads need; and the store's
fill reaches the core with each commit's answer, which refuses new work above
a high-water mark and shows the fill in the deployment's document. Everything
lands on `jig-fake-store` and is proven in the domain worlds; the store's
component reads these keys from jig 13 on.

## Read first

- jig's `protocol/store.md`, sections 4.2 to 4.4 (tags, ranges to intervals,
  jig's families), 7.2 (the restart's ranges), 7.3 (parties' reads: the
  single reads), 11.2 (fill) and 15, items 1, 5 and 7.
- jig's `domain/engine.md`, sections 5.4 to 5.6 and 6 (what the store keeps,
  what the core asks of it, the restart script).
- `decisions.md`, 9 (fill reaches the core as an event of its own; an
  escalation's read shows its latest kept decision).
- `findings/domain.md`, items 11.1, 11.5 and 11.7; `findings/store.md`,
  sections 2.1 to 2.6, 3 (rows 4 to 6 of the table) and 7.1 (D1, D3, D6).
- `code-organization.md`, sections 1 (the new names) and 6 (the "Don't"
  list).

## What the design asks

- **Live before history** (store.md 15.1, 4.3): "within each child, the
  families a restart loads come first, and history after them …, so each
  child's live part is one interval"; "the tasks child's history and ended
  tasks after its live families (stubs among these), decided proposals
  apart from pending ones, and past funding periods apart from live ones".
- **The restart's ranges** (store.md 7.2): "the deployment's header first,
  then people's families, then the tasks child's live families, then the
  core's own live rows … It is a fixed list of intervals, in that order";
  "what the core never restores (turns, terminals, history, ended tasks,
  notes' entries) is never read at start".
- **Keys for single reads** (store.md 15.7): "a question's line and its
  answer's, an escalation's hold, and a task's proposal's row in its
  proposer's history, each named by its position in its task's record, as a
  reply's is, so a read finds it by key, never by a scan". 7.3: "an
  escalation | its decision, by the revision its task's record names, with
  the hold's milestone".
- **Fill** (store.md 11.2, 15.5): "The store reports its fill with each
  commit's answer: the map's bytes against its budget"; "Above a high-water
  mark of that budget …, the domain refuses new work at its entrances: new
  tasks and chats, new sign-ins. Work under way goes on, and a refusal is
  never a failed commit. The deployment's document shows the fill."

## Choices this plan makes

- **Family names.** Pending person proposals keep `tasks::Key::PersonProposal`;
  decided ones move to `tasks::Key::DecidedProposal`. Live ledgers keep
  `tasks::Key::Ledger`; closed ones move to `tasks::Key::ClosedLedger`. Both
  new families hold the same record types as before (`PersonProposal`,
  `FundingRecord`): no new record type.
- **Each child names its live keys** with `Key::live(&self) -> bool`; the core
  names its restart's ranges as `jig_domain_core::RestoreRange`, whose
  `admits(&Key)` is the predicate jig 13 maps to intervals (store.md 4.3).
- **A closed ledger leaves memory** in the decision that closes it: erased
  from `Ledger`, saved under `ClosedLedger`. A project's newest period is
  never closed (`funders.rs` closes a period only once a newer one exists),
  so it stays live and `open`'s monotonic check (`funders.rs:407-416`) still
  reads it.
- **Single reads' positions** are kept newest first in the task record,
  bounded by `tasks::Limits::inbox_messages`, as `TaskRecord.replies` is.
- **Fill** reaches the core as `core::Event::Fill { bytes }`, sent by the
  root after the journal takes the commit's number (decision 9); the mark
  is `core::Limits::fill_mark`, in the map's bytes, which jig 20's profiles
  fill.
- **The refusal above the mark** is the existing unsaved `Again::Busy` for a
  party's goal, person proposal or chat, and the existing
  `Now::SignInRefused` for a sign-in: no new word reaches the client (see
  "Questions").
- **The entrances** are a party's new goal, person goal proposal and chat,
  and a new sign-in. A run's delegation and a recurring task's next run are
  work under way and go on (see "Questions").

## Rules that bind it

- **A durable record's codec follows its record** (README, "Rules"): "Once
  jig 14 or jig 19 has landed a family's codec, an increment of any session that
  changes a durable record of that family changes its schema, its generated
  codec and its translation in the same increment, and the record's
  family's version moves at the first such change in this plan." The same
  holds for jig 13's key encoders: an increment that adds or moves a key
  variant extends `crates/jig-protocol-store/src/keys/` when it exists.
- **No increment "adds a type beside an existing one for the same thing: it
  changes the type in place, and its users in the same increment"**
  (`code-organization.md`, section 6).
- **"A file: at most 800 lines, generated code excepted. A file of at
  most 800 lines that the increment grows past 800 is split in the same
  increment"**
  (`code-organization.md`, section 7). `crates/jig-domain-tasks/src/funders.rs`
  is 784 lines today.

## Increments

### 11.1 Live first: keys reordered, the core's restart ranges named

Scope: `tasks::Key`'s variants reordered live first (Live, Stub, Ledger,
Writer, Pool, PersonProposal, then Ended, History, Milestone, Activity);
`CoreKey::Deployment` moved to variant 0; `Key::live` on the core's, the
tasks child's and the people child's keys; `RestoreRange` with its
`admits` predicate and the fixed order `RestoreRange::LIVE` (store.md 4.3,
7.2, 15.1). The worlds' restarts load by these ranges in order, and their
rank sorts and variant filters go.
Needs: nothing.
Files:
- `crates/jig-domain-tasks/src/boundary.rs` (`Key` at :978-999 reordered, `Key::live` added; `Stored::key` :1040-1055 unchanged);
- `crates/jig-domain-core/src/stored.rs` (`CoreKey` :183-200: `Deployment` first; `CoreKey::live`);
- `crates/jig-domain-people/src/boundary.rs` (`Key` :532-547: `Key::live`, true for each of its seven variants);
- `crates/jig-domain-core/src/restart.rs` (`RestoreRange`), `crates/jig-domain-core/src/lib.rs` (re-export, beside `inbox_restore_key` at :30);
- the order-dependent users of the reordered variants: `tests/client/src/joint/wire/cursor.rs` (`CoreKey` tags :597-645, `tasks::Key` tags :793-850), `tests/tasks/src/world.rs` (:834-846, `restart` :976-1006);
- the worlds' restart loads: `tests/core/src/effects.rs` (`load_restart` :1028-1048, `restored` :1066-1120, shared hotspot), `tests/conformance/src/lib.rs` (`Neighbours::restored` :146-215), `tests/ops/src/conformance/mod.rs` (:340-393), `examples/ops/domain/src/restart.rs` (:121, `Milestone` accepted and dropped).
Shape:
- `jig_domain_tasks::Key`: the same ten variants and fields, declared in the
  order above; `pub const fn live(&self) -> bool` (true for Live, Stub,
  Ledger, Writer, Pool, PersonProposal).
- `jig_domain_core::CoreKey`: `Deployment` first, then Projection, RunProof,
  Call, Turn, Terminal, ProposalDecision, EscalationDecision;
  `pub const fn live(&self) -> bool` (Projection, RunProof, Call; Deployment
  is its own range). A live core family jig 06 adds (each live task's
  conversation, store.md 4.4) answers true.
- `jig_domain_people::Key::live(&self) -> bool`: true for each variant.
- `jig_domain_core::RestoreRange` and `RestoreRange::LIVE` as sketched; a
  world's `LoadCore` loads `LIVE[0]` to `LIVE[3]` in turn, each in key order,
  feeding each row to `restore_record` as today. The root still sends
  `people::Event::Restored` after the step.
- Notes rows leave `LoadCore`: no range admits a `Key::Notes` (store.md 7.2).
Test shape:
- New `crates/jig-domain-tasks/src/tests/keys.rs`, declared in
  `crates/jig-domain-tasks/src/tests.rs` beside `mod revisions;` (:22).
  Reference test, written out in full:

  ```rust
  //! Key order against store.md 4.3: a child's live families come first.
  use crate::{Funder, Key};

  #[test]
  fn every_live_tasks_key_sorts_before_every_history_key() {
      let live = [
          Key::Live(u64::MAX),
          Key::Stub(u64::MAX),
          Key::Ledger(Funder::Period { project: u32::MAX, period: u64::MAX }),
          Key::Writer(u64::MAX),
          Key::Pool(u64::MAX),
          Key::PersonProposal(u64::MAX),
      ];
      let history = [
          Key::Ended(0),
          Key::History { task: 0, revision: 0 },
          Key::Milestone { task: 0, position: 0 },
          Key::Activity { task: 0, position: 0 },
      ];
      for early in live {
          assert!(early.live(), "{early:?} is a live family");
          for late in history {
              assert!(!late.live(), "{late:?} is history");
              assert!(early < late, "{early:?} sorts before {late:?}");
          }
      }
  }
  ```
- Also there: `the_deployment_key_sorts_before_every_other_core_key` (in
  `crates/jig-domain-core/src/tests/restore_admission.rs`, which already
  holds restore tests; fixtures `limits()` and `config(seed)` at
  `crates/jig-domain-core/src/tests.rs:29` and `:193`),
  `restore_ranges_admit_no_turn_terminal_history_or_note`,
  `restore_ranges_admit_each_live_family_once`. Unit ceilings: under
  20 ms each.
- World: `a_restart_restores_people_before_live_tasks_and_core_rows` in
  `tests/core/tests/restart.rs` (reuse `World::new(seed, false)`,
  `World::restart` at `tests/core/src/effects.rs:1150`; the world records
  the ranges it loaded, in order, in a field of `World` it reads back).
  Focused ceiling 0.5 s. The existing restart, store and conformance
  stories run unchanged.
Proof: world. `cargo nextest run -p jig-core-world --test restart` passes
once, through `heavy`, with the rank sort gone: it shows a restarted engine
answering as before when the worlds read only `RestoreRange::LIVE`, in its
order. No joint layer exists for the store yet (jig 18 builds it). Bound:
1 minute.
Don't: change any variant's fields; let a world filter rows by variant
after this increment.
Size: at most 120 lines of production code (domain and roots; world code
is test code).
Removes: the rank sorts at `tests/core/src/effects.rs:1083-1115`,
`tests/conformance/src/lib.rs:153-185` and
`tests/ops/src/conformance/mod.rs:383-390`; the variant filter at
`tests/tasks/src/world.rs:991-1000`; the `Milestone` drop at
`examples/ops/domain/src/restart.rs:121`.

### 11.2 Decided proposals and closed ledgers apart

Scope: a decided person proposal moves from `PersonProposal` to
`DecidedProposal` in the decision that decides it; a closed ledger moves
from `Ledger` to `ClosedLedger` in the decision that closes it, and leaves
`domain.funding`; neither is restored (store.md 15.1).
Needs: nothing beyond 11.1.
Files:
- `crates/jig-domain-tasks/src/boundary.rs` (`Key`, `Stored`, `Stored::key`);
- `crates/jig-domain-tasks/src/proposals.rs` (`decide_person` :262-334: erase `PersonProposal(n)`, save `DecidedProposal`);
- `crates/jig-domain-tasks/src/stored.rs` (`restore` :703-780: a decided state under `PersonProposal` refused `Restore`; `DecidedProposal` and `ClosedLedger` refused as history);
- `crates/jig-domain-tasks/src/funders.rs` (`retire` :633-686, `restore_funding` :594-619, `open` :391-416; split into `funders.rs` and a new `crates/jig-domain-tasks/src/retire.rs` so neither passes 800 lines);
- the readers of `FundingRecord.closed`, each checked against eviction: `crates/jig-domain-core/src/documents/objects.rs:432`, `documents/project_note.rs:284,345`, `crates/jig-domain-tasks/src/{moving.rs:203,226, standing.rs:56,62, periods.rs:31}`;
- the exhaustive `Stored` matches (the users of `Stored::Pool` as the marker, `findings/domain.md` 11.1): tasks `boundary.rs`, `holds.rs`, `lib.rs`, `owned.rs`, `stored.rs`; core `inbox.rs`, `proposals.rs`, `reads/rows.rs`, `reads/vocabulary.rs`, `routing/tasks.rs`, `stored.rs`; `tests/client/src/joint/truth.rs`, `tests/core/src/observations.rs`, `tests/core/src/watches.rs`, `tests/core/tests/effects.rs`, `tests/ops/src/conformance/observations.rs`, `tests/tasks/src/accounting_referee.rs`, `tests/tasks/src/world.rs`, `tests/tasks/tests/{activity,funding,person,recurring,referee,restart}.rs`;
- the `Key::Ledger` and `Key::PersonProposal` users: `crates/jig-domain-core/src/inbox.rs:397-414`, `reads/rows.rs:1512-1546` (`key_kind`: `ClosedLedger(Period{project,..})` is `KeyKind::Ledger(project)`, `DecidedProposal(n)` is `KeyKind::PersonProposal(n)`), `tests/client/src/joint/wire/cursor.rs`, `tests/client/src/joint/wire/ledger_pages.rs`, `tests/core/src/effects.rs`, `tests/core/tests/stories.rs`, `tests/core/tests/stories/procedure_goals.rs`;
- if jig 13 has landed `crates/jig-protocol-store/src/keys/tasks.rs`, its encoder and decoder for the two new variants.
Shape:
- `tasks::Key::DecidedProposal(u64)` and `tasks::Key::ClosedLedger(Funder)`,
  in the history part of the order: Ended, ClosedLedger, DecidedProposal,
  History, Milestone, Activity.
- `tasks::Stored::DecidedProposal(Box<PersonProposal>)` and
  `tasks::Stored::ClosedLedger(FundingRecord)`; `Stored::key` maps them.
- `FundingRecord.closed` stays (it is the saved row's fact); a ledger in memory
  is open. Each `!row.closed` test on `domain.funding` becomes a
  presence test; a pool or recurring allotment of a project period older
  than the project's newest live period is refused as closed, where
  `moving.rs` would otherwise make it afresh (`moving.rs:203-230`).
Test shape:
- In `crates/jig-domain-tasks/src/tests/periods.rs` (reuse `Harness`,
  `LIMITS` and `to()` at `crates/jig-domain-tasks/src/tests/revisions.rs:93`,
  `:21`, `:99`, or `crate::tests::fixtures` if a session before this one
  has moved them there): `a_closed_period_leaves_memory_and_saves_under_closed_ledger`,
  `a_restored_closed_ledger_refuses_the_start`,
  `the_newest_period_stays_live_after_its_pools_close`.
- In `tests/tasks/tests/person.rs` (reuse `jig_tasks_world::{World, LIMITS, task}`):
  `a_decided_person_proposal_is_erased_and_saved_as_decided`,
  `a_restart_restores_no_decided_person_proposal`.
- Extend `every_live_tasks_key_sorts_before_every_history_key` with
  `Key::ClosedLedger(Funder::Task(0))` and `Key::DecidedProposal(0)`.
- Ceilings: unit 20 ms, world 0.5 s.
Proof: world. `cargo nextest run -p jig-tasks-world --test person --test
funding --test restart` once, through `heavy`: a restarted tasks child
holds no closed period and no decided proposal, and its funders' table has
room for a new period after earlier ones closed. Bound: 1 minute.
Don't: keep a closed ledger in `domain.funding`; read a `ClosedLedger` or
`DecidedProposal` row at restore.
Size: at most 250 lines of production code.
Removes: the decided-proposal branch of `restore` that reads and discards
(`crates/jig-domain-tasks/src/stored.rs:724`).

### 11.3 Keys for single reads

Scope: a task's record names the activity positions of each question it
heard and of its answer, with the task whose activity holds the answer
(store.md 7.3, "An answer is read where it was heard"), the milestone
position of its latest hold, and the history revision that keeps each of
its decided proposals; the core world's
read transport serves `Scope::Question`, `Scope::Escalation` and
`Scope::Proposal { proposer: Task(_), .. }` by key from them (store.md
15.7, 7.3; decision 9).
Needs: nothing beyond 11.1.
Files:
- `crates/jig-domain-tasks/src/boundary.rs` (`TaskRecord` :844-972: three fields; `QuestionLine`, `AnswerLine`, `ProposalRow`);
- `crates/jig-domain-tasks/src/activity.rs` (`message` :90-107: a Question line records its position; a party's Answer line fills `answer` with the task's own);
- `crates/jig-domain-tasks/src/inbox.rs` (`message`, where a task's answer clears its asker's credit: the answering task's `QuestionLine` names the line in the asker's activity, and its record is published in the same decision);
- `crates/jig-domain-core/src/reads/rows.rs` (the `Scope::Question` arm, :668-720: the answer's line is accepted from the task the record names, not only the question's);
- `crates/jig-domain-tasks/src/domain.rs` (`milestone` :827-880: `hold` set when the phase is Held);
- `crates/jig-domain-tasks/src/control.rs` and `proposals.rs` (where a decided proposal's `History` row is saved for its proposer);
- `crates/jig-domain-tasks/src/stored.rs` (`valid_record`: the new fields' bounds), `owned.rs` (:403, the record's bytes);
- the `TaskRecord` literal constructions: `crates/jig-domain-tasks/src/domain.rs:1143`, `tests/core/src/stories/closing_chat.rs`, `tests/core/src/stories/instructions.rs`, `tests/core/tests/stories.rs`, `tests/tasks/src/world.rs`;
- the core world's transport: `tests/core/src/reads.rs` (`candidate` :228-327, the three scopes now served);
- if jig 14 has landed the tasks family's codec: its schema, generated code and translation (README, "Rules").
Shape:
- `TaskRecord.question_lines: Box<[QuestionLine]>`: "the activity positions
  of the questions this task heard and of their answers, each answer with
  the task that heard it, newest first, at most `inbox_messages`".
- Where an answer is: a party's answer is a line in the task's own
  activity; the task's answer to the delegate that asked is the line
  `inbox::message` appends to the delegate's activity, as today, and the
  same decision names it in the answering task's `QuestionLine`
  (`AnswerLine { task: delegate, position }`). No line is added to the
  answering task's activity. The first answer is kept; a later one
  changes nothing.
- `TaskRecord.hold: u64`: "the milestone position of this task's latest
  hold; zero before its first".
- `TaskRecord.proposal_rows: Box<[ProposalRow]>`: "the history revision
  that keeps each proposal this task made and that was decided, newest
  first, at most `inbox_messages`".
- `QuestionLine`, `AnswerLine` and `ProposalRow` as sketched.
- The scopes' reads stay as they are (`crates/jig-domain-core/src/reads/vocabulary.rs:61-96`):
  the transport finds the owner's record by key (`Live(task)`, then
  `Ended(task)`), and from it the row's key: `Activity { task, position:
  line }` with `related` `Activity { task: answer.task, position:
  answer.position }`;
  `EscalationDecision { task, revision: escalation.revision() }` with
  `related` `Milestone { task, position: hold }`; `History { task:
  proposer, revision }`. A read whose position the record no longer keeps
  answers `Why::Gone`.
Test shape:
- In `tests/core/tests/reads.rs` (reuse `stories::reads::{world, request}`,
  `reads::{ask, latest}`, `World::say`, `World::end`):
  `a_question_read_after_its_task_ended_finds_both_lines_by_key`,
  `an_escalation_read_shows_its_latest_decision_and_its_hold`,
  `a_task_proposal_read_finds_its_history_row_by_key`,
  `a_question_older_than_the_kept_lines_reads_as_gone`,
  `a_question_answered_by_its_task_reads_the_line_in_the_delegates_activity`.
  Each is its own `#[test]`, focused ceiling 0.5 s.
- In `crates/jig-domain-tasks/src/tests/revisions.rs` (its `Harness`):
  `a_held_task_names_the_milestone_of_its_hold`,
  `question_lines_keep_the_newest_within_inbox_messages`,
  `a_tasks_answer_names_the_line_in_the_delegates_activity`. Unit ceiling
  20 ms.
- The transport's three new arms read only the keys named above: no
  iteration over `store.rows` for these scopes (its `page` keeps the scan
  for the other scopes until jig 12 and jig 16).
Proof: world. `cargo nextest run -p jig-core-world --test reads` once,
through `heavy`: a person reads a question, an escalation and a task's
proposal after the task ended, each found by key. Bound: 1 minute.
Don't: change `Scope`, `Read` or `Position`; scan a family in the
transport's new arms; add a line to the answering task's activity.
Size: at most 240 lines of production code (the designer's answer on an
answer's task, 2026-10-12, adds about 40).
Removes: nothing.

### 11.4 Fill: the event, the mark, refusals at the entrances

Scope: the root hands the core the store's fill with each commit's answer;
at or above the mark, a party's new goal, person goal proposal and chat
are answered busy, and a new sign-in is refused; work under way goes on
(store.md 11.2, 15.5; decision 9).
Needs: nothing beyond 11.1.
Files:
- `crates/jig-domain-core/src/routing/vocabulary.rs` (`Event::Fill`, `Limits::fill_mark` at :275; shared hotspot);
- `crates/jig-domain-core/src/lib.rs` (`Core.fill`), `routing/mod.rs` (the event's route);
- the entrances: `crates/jig-domain-core/src/routing/people.rs` (`sign_in` :62-90), `routing/parties.rs` (`begin_goal_creation` :134, `begin_chat_creation` :191, and the person goal proposal's admission);
- the roots: `testing/jig-test-domain/src/lib.rs` (`Event::Committed` :268, handled :562), `examples/ops/domain/src/boundary.rs` (`Store::Committed` :103), `examples/ops/domain/src/domain.rs:203`, `examples/ops/domain/src/restart.rs:78`;
- the `Committed` users (`findings/domain.md` 11.5): `examples/ops/domain/src/tests.rs` (6), `tests/ops/src/conformance/mod.rs:236`, `tests/core/src/effects.rs` (:610, :1132), `tests/core/src/world.rs:494`, `tests/core/src/{observations.rs:403, reads.rs:83, watches.rs:195, stories/reads.rs:292}`, `tests/core/tests/{client_peer_routes.rs:190,199, fuzzy_reads.rs:32, application_documents.rs:119, client_peer.rs:71, reads.rs:29, stories.rs:1465,1642,1736}`, `tests/conformance/src/lib.rs:404`, `tests/conformance/src/negative.rs:155`;
- the `core::Limits` literals: `crates/jig-domain-core/src/routing/vocabulary.rs`, `crates/jig-domain-core/src/tests.rs:29`, `tests/core/src/world.rs`, and the roots' limits constructors.
Shape:
- `core::Event::Fill { bytes: u64 }`: "the store's map bytes, from the root
  with each commit's answer; it decides nothing and needs no commit".
- `core::Limits::fill_mark: u64`: "the high-water mark, in the store map's
  bytes; at or above it a party's new goal, person goal proposal and chat
  are answered busy, and a sign-in is refused".
- The roots: `Event::Committed { number, fill: u64 }` (testing root) and
  `Store::Committed { number, fill: u64 }` (`ops`): the journal takes
  `number`, then the root sends `core::Event::Fill { bytes: fill }`.
- The fake store has no fill: each world passes `fill: 0` unless a test
  sets it (`World::fill` in `tests/core/src/effects.rs`, used for the
  next commits' answers).
Test shape:
- New `crates/jig-domain-core/src/tests/fill.rs`, declared in
  `crates/jig-domain-core/src/tests.rs` beside `mod restore_admission;`
  (:2934); it reuses `core()` (`crates/jig-domain-core/src/tests.rs:301`: a
  core whose people child is restored, with person 1 owning project 1).
  Reference test, written out in full:

  ```rust
  //! The store's fill and the high-water mark at the entrances (store.md 11.2).
  use super::*;

  #[test]
  fn a_sign_in_at_the_fill_mark_is_refused() {
      let (mut core, limits) = core();
      let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
      let _filled = step(&mut core, &env, Event::Fill { bytes: limits.fill_mark });
      let identity = people::Identity {
          key: people::IdentityKey { provider: 0, subject: Box::new([2]) },
          login: b"second".as_slice().into(),
          name: b"B Person".as_slice().into(),
      };
      let Requests::Out(mut out) =
          step(&mut core, &env, Event::SignIn { reply_to: ReplyTo::new(Token::new(7201)), identity });
      let Some(Request::Now(now)) = out.pop() else { panic!("one refusal, first") };
      let Now::SignInRefused { to } = *now else { panic!("the sign-in's refusal") };
      assert_eq!(to.into_token(), Token::new(7201));
      let Some(Request::Decided) = out.pop() else { panic!("the decision closes") };
      assert!(out.is_empty(), "nothing is written");
  }
  ```
- Also there: `a_chat_at_the_fill_mark_is_answered_busy`,
  `a_goal_below_the_fill_mark_is_made`.
- In `tests/core/tests/store.rs` (reuse `World`, `stories::reads::request`):
  `a_running_chat_goes_on_above_the_fill_mark`,
  `a_fill_below_the_mark_admits_a_chat_again`.
- Ceilings: unit 20 ms, world 0.5 s.
Proof: world. `cargo nextest run -p jig-core-world --test store` once,
through `heavy`: a person's new chat is answered busy while the store
reports itself full, and the chat already running answers its next turn.
Bound: 1 minute.
Don't: refuse a turn, an answer, a call or a connector's event for fill;
let a fill refusal make a commit.
Size: at most 150 lines of production code.
Removes: nothing.

### 11.5 The deployment's document shows the fill

Scope: the deployment's document carries the last fill and the mark, and
the client's contract, view and wire tests show them (store.md 11.2: "The
deployment's document shows the fill").
Needs: 11.4's `Core.fill`.
Files:
- `crates/jig-domain-core/src/documents/inbox_deployment.rs` (`Deployment` :142-166, its constructor :742; 830 lines today: move the deployment document to `documents/deployment.rs` in this increment);
- the client's mirrors: `crates/jig-domain-client/src/contract/owner_deployment.rs`, `crates/jig-view/src/owner_deployment.rs` and the view's phrasebook (one line), `testing/jig-scripted-engine/src/tests/owner_deployment.rs`, `tests/client/src/joint/wire/deployment.rs`, `tests/client/src/joint/wire/deployment/tests.rs`;
- `jig-protocol-client-wire`'s deployment record, if jig 30 has landed it (shared with jig 30).
Shape:
- `documents::Deployment.store: StoreFill`, with
  `StoreFill { bytes: u64, mark: u64 }`: "the store map's bytes at the last
  commit's answer, and the high-water mark". The client's
  `owner_deployment` contract type gains the same field, named the same.
Test shape:
- `crates/jig-domain-core/src/tests/deployment_connectors.rs`:
  `the_deployment_document_shows_the_last_fill_and_the_mark`.
- `crates/jig-view/src/tests/owner_deployment.rs`:
  `the_deployment_page_shows_the_store_fill_against_its_mark`.
- Ceiling: unit 20 ms.
Proof: unit. The two tests above, run once through `heavy`: the owner's
deployment page shows the fill and the mark. The client's joint world
already carries the deployment document; its existing deployment tests
pass unchanged. Bound: 1 minute.
Don't: add a field to the core's document that the client's contract
lacks.
Size: at most 100 lines of production code.
Removes: nothing.

## Interface sketches

```rust
// jig_domain_tasks::boundary (crates/jig-domain-tasks/src/boundary.rs)
use crate::Funder;

/// Logical store key for task state or history (store.md 4.4, 15.1): live
/// families first, then history; jig 12 appends its indexes after history.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Key {
    /// A live task's row. Live.
    Live(u64),
    /// A stub naming an ended result a live task names. Live.
    Stub(u64),
    /// An open period, pool or recurring allotment. Live.
    Ledger(Funder),
    /// An occupied writer slot. Live.
    Writer(u64),
    /// A connector pool's last capacity. Live.
    Pool(u64),
    /// A pending person goal proposal. Live.
    PersonProposal(u64),
    /// An ended task's row. History.
    Ended(u64),
    /// A closed period, pool or allotment, kept for spend pages. History.
    ClosedLedger(Funder),
    /// A decided person goal proposal. History.
    DecidedProposal(u64),
    /// One semantic change. History.
    History { task: u64, revision: u64 },
    /// One lifecycle milestone. History.
    Milestone { task: u64, position: u64 },
    /// One activity line. History.
    Activity { task: u64, position: u64 },
}

impl Key {
    /// Whether a restart loads this family (store.md 7.2).
    #[must_use]
    pub const fn live(&self) -> bool { todo!() }
}

/// A question this task heard, and where its answer is (store.md 7.3, 15.7).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct QuestionLine {
    /// The question's message number, as `Scope::Question` names it.
    pub message: u64,
    /// The question's activity position in this task.
    pub line: u64,
    /// Where its first answer was heard, once answered.
    pub answer: Option<AnswerLine>,
}

/// An answer's activity line: this task's own for a party's answer, the
/// asking delegate's for this task's answer to it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AnswerLine {
    pub task: u64,
    pub position: u64,
}

/// The history row that keeps one of this task's decided proposals.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ProposalRow {
    /// The proposal's number, as `Scope::Proposal` names it.
    pub proposal: u64,
    /// The revision of this task's `History` row that keeps it.
    pub revision: u64,
}

// TaskRecord gains, beside `replies`:
//     pub question_lines: Box<[QuestionLine]>,
//     pub hold: u64,
//     pub proposal_rows: Box<[ProposalRow]>,
// Stored gains `DecidedProposal(Box<PersonProposal>)` and `ClosedLedger(FundingRecord)`.
```

```rust
// jig_domain_core::restart (crates/jig-domain-core/src/restart.rs)
use crate::Key;

/// One range of the core's restart, in the order it restores them (store.md
/// 7.2). The store maps each to a fixed list of intervals (jig 13); a world
/// feeds its rows to the root's restore in key order. Not `Copy`: jig 12
/// adds the census's range, which holds a list.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RestoreRange {
    /// The deployment's header: one row, first.
    Deployment,
    /// Every people family.
    People,
    /// The tasks child's live families (`tasks::Key::live`).
    Tasks,
    /// The core's own live rows (`CoreKey::live`).
    Core,
}

impl RestoreRange {
    /// `LoadCore`'s ranges, in order.
    pub const LIVE: [RestoreRange; 4] =
        [RestoreRange::Deployment, RestoreRange::People, RestoreRange::Tasks, RestoreRange::Core];

    /// Whether this range holds `key`: the predicate the store's intervals
    /// are checked against (store.md 4.3).
    #[must_use]
    pub fn admits(&self, key: &Key) -> bool { todo!() }
}
```

```rust
// jig_domain_core::routing::vocabulary (crates/jig-domain-core/src/routing/vocabulary.rs)
pub enum Event {
    // … every variant as today, and:
    /// The store's map bytes, from the root with each commit's answer
    /// (store.md 11.2; decisions.md 9). It decides nothing.
    Fill { bytes: u64 },
}
pub struct Limits {
    // … every field as today, and:
    /// The high-water mark, in the store map's bytes (store.md 11.2).
    pub fill_mark: u64,
}

// jig_domain_core::documents
/// The store's fill as the deployment's document shows it (store.md 11.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct StoreFill {
    /// The map's bytes at the last commit's answer.
    pub bytes: u64,
    /// The high-water mark.
    pub mark: u64,
}
```

- **Owners.** The tasks child owns its keys and their order; the core owns
  `RestoreRange` and the fill's mark. jig 12 adds the census's ranges to
  `RestoreRange` and the index families after `Activity`; jig 13 maps
  `RestoreRange` to intervals and encodes the keys.
- **What ends what.** A restart's `LoadCore` ends when the root has fed the
  four ranges and the core answers `restart_done`; a fill ends nothing: it
  replaces the last one.

## Questions to raise when reached

- **The word above the high-water mark** (11.4). store.md 11.2 says "the
  domain refuses new work at its entrances" and names no refusal; the
  client's contract names none either. This plan answers busy (unsaved,
  `people::Again::Busy`) and refuses a sign-in as `SignInRefused`. A
  designer may want a word that says the store is full.
- **Which entrances** (11.4). "new tasks and chats, new sign-ins": this
  plan gates a party's goals, person goal proposals and chats, and
  sign-ins, and lets a run's delegation and a recurring task's next run go
  on, as "work under way". A delegation is a new task too.
- **A single read past the kept positions** (11.3). Question lines and
  proposal rows are kept newest first, at most `inbox_messages`, as
  replies are; an older question's or proposal's read answers gone. store.md
  15.7 says "as a reply's is", which bounds them; whether an older one
  should stay readable is the designer's.

## Done when

- `git -C ~/src/rust/jig grep -n 'let rank = match row' -- tests` prints
  nothing.
- `git -C ~/src/rust/jig grep -n -E 'ClosedLedger|DecidedProposal' --
  crates/jig-domain-tasks/src/boundary.rs` prints the two keys and the two
  rows.
- `git -C ~/src/rust/jig grep -n 'Fill { bytes' --
  crates/jig-domain-core/src/routing/vocabulary.rs` prints one line.
- `heavy cargo nextest run -p jig-domain-tasks -p jig-domain-core -p
  jig-tasks-world -p jig-core-world` passes once on `main` after 11.5.
