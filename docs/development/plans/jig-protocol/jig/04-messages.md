# Session 04: a run's messages, from its start to its end

A run's start carries the messages waiting in its task's inbox, as many
as the run's inbox takes, and the brief no longer has an inbox section;
later messages are relayed as the run reads, never more relayed and
unread than its inbox takes; a turn's commit takes the messages its fence
covers but those the run refused; and each message a run refused or
left unread stays in the inbox and wakes the task again when the run
ends, as if it arrived then. It builds on jig 05's assignment, turns
and answer (the start's messages in the hub's assignment, refusals in
turns, the answer's fence), and changes only the core, the tasks child,
the brief child and the roots' assembly of the start: never the hub's or
the fleet's boundaries (decisions.md, 6).

Paths carry jig 01's names (`crates/jig-domain-core/` was
`crates/jig-core/`, `crates/jig-domain-tasks/` was
`crates/jig-core-tasks/`, `crates/jig-domain-brief/` was
`crates/jig-core-brief/`, `crates/jig-domain-host/` was
`crates/jig-host/`). Line numbers are as built at jig `1cdfeb62`; jig 01
and jig 05 may move them by a few lines.

## Read first

- jig's `domain/tasks.md`, 8.2 (the inbox: "Carried whole and read
  oldest first", "Refused or unread, a message stays") and 8.3 (wake
  policies: "A message a run refused or left unread (8.2) is judged
  again when the run ends").
- jig's `domain/engine.md`, 7.1 (steps 3 and 4: the claim commits "the
  messages its start carries"), 7.2 ("Committed turn by turn"), 7.4
  ("Messages to a live run", "Refused or unread, a message stays"),
  section 9 ("A brief has no inbox section") and section 15 (the
  stories).
- jig's `domain/hosts.md`, section 9 (messages and delivery).
- jig's `protocol/channel.md`, 5.1 ("Refusals ride with turns") and 5.2
  (the assignment's start's messages).
- findings/domain.md, item 6, 6.3 to 6.5.
- This plan's `jig/05-hub-link.md`, 05.2 and 05.3 (the fields this
  session fills and reads).

## What the design asks

- **The start carries the inbox** (tasks.md, 8.2): "A run's start
  carries the messages waiting in its task's inbox, each whole, oldest
  first, as many as the run's inbox takes ... The rest wait in the
  inbox, in order, and are relayed as the run reads, ... never more
  relayed and unread than the run's inbox takes".
- **No inbox section** (engine.md, 9): "The messages waiting in the
  task's inbox go with the assignment as its start's messages".
- **A turn takes what it read but what it refused** (engine.md, 7.2):
  "The core commits the turn with its spend, with the messages up to
  that one taken but those it says the run refused".
- **Refused or unread, a message stays** (tasks.md, 8.2 and 8.3; hosts.md,
  9): "stays in the inbox, untaken, and wakes the task again when the run
  ends, as it would had it arrived then"; "it wakes the task if its kind
  does, and otherwise waits for the next activation".

## Choices this plan makes

- `tasks::Limits::run_inbox` is the deployment's count of messages a run's inbox takes: the start's count and the relay window.
- The core takes the start's messages from the head of `RunContext.inbox`, which keeps its full snapshot for procedures.
- `RunProof.offered` is the last message the start carries, committed with the claim; the claim's tasks event is unchanged.
- The relay window is the core's, judged against `RunProof.offered`; one relay of a run is in flight at a time, the next sent when the previous one's offer commits or a turn's fence frees room.
- A run's refusals are kept on its task (`TaskRecord.refused`) until the run ends, so a restart between turns keeps them.
- An answer whose fence passes its last committed turn's read, or whose refusals name a message it was not offered, is admitted as failed invalid, as an answer with invalid saved state is.
- At a run's end the tasks child judges each message still in the inbox again, as `wake::after_step` does for procedures.

## Rules that bind it

- From README, "Rules of this plan": "A durable record's codec follows
  its record. Once jig 14 or jig 19 has landed a family's codec, an increment of
  any session that changes a durable record of that family changes its
  schema, its generated codec and its translation in the same increment,
  and the record's family's version moves at the first such change in
  this plan".
- From code-organization.md, section 6: "No increment of this plan:
  adds a type beside an existing one for the same thing: it changes the
  type in place, and its users in the same increment; ... computes an
  expected value with the code under test;".
- From code-organization.md, section 4: "the core's vocabulary toward
  the root (`routing/vocabulary.rs`) | `jig-domain-core` | each session
  adds its variants; hotspot".
- For this session: a file already over 800 lines (named in each
  increment's Files) must not grow by more than 20 lines in an
  increment; new code goes in a module of its own (code-organization.md,
  section 7). `git diff --stat` in the commit body shows it.

## Increments

### 04.1 The start carries the inbox's waiting messages

Scope: a run's start carries the oldest messages waiting in its task's
inbox, each uncut, at most `run_inbox`; the claim commits the last of
them as the run's offered fence; the roots put them in the hub's
assignment, and `ops`'s root stops replaying its inbox after the
assignment; the brief loses its inbox section (tasks.md, 8.2;
engine.md, 7.1 and 9; findings/domain.md, 6.4).
Needs: jig 05's assignment (`jig_domain_host::Assignment.messages`).
Files:
- `crates/jig-domain-tasks/src/limits.rs` (`Limits`, :61-65;
  `worst_case`, :113-116): `run_inbox`; its literals (4):
  `crates/jig-domain-tasks/src/tests/revisions.rs:44`,
  `crates/jig-domain-core/src/tests.rs:54`, `tests/core/src/world.rs:41`,
  `tests/tasks/src/world.rs:40`.
- `crates/jig-domain-core/src/routing/run_preparation.rs`
  (`workspace_prepared`, :93-96 and :131-141; shared with jig 06 and
  jig 07).
- `crates/jig-domain-core/src/routing/vocabulary.rs` (`Now::RunPrepared`,
  :254-264; hotspot, the assignment).
- `Now::RunPrepared`'s consumers: `examples/ops/domain/src/route.rs:1245-1258`
  (1,585 lines), `testing/jig-test-domain/src/routes.rs:421-443` (1,005
  lines); the `..` matches at
  `crates/jig-domain-core/src/tests/completion_retry.rs:52`,
  `tests/core/src/client_peer.rs:159`, `tests/ops/src/joint/front.rs:751`,
  `tests/ops/src/joint/engine.rs:342`, `tests/client/src/joint/session.rs:796`
  stay.
- `ops`'s root: `examples/ops/domain/src/assemble.rs` (`assignment`,
  :48-112; the brief's title at :37), `examples/ops/domain/src/domain.rs`
  (`Assignment.inbox`, :17-20), `examples/ops/domain/src/route.rs`
  (`Held::Assign`, :249-259; `offered_inbox`, :341-356).
- The test root: `Assignment.inbox` (`testing/jig-test-domain/src/lib.rs:141`)
  and its readers `tests/core/src/peers.rs:255-262`,
  `tests/core/src/effects.rs:719`, `tests/core/src/world.rs:634`,
  `tests/core/src/observations.rs:835-846`, `tests/conformance/src/lib.rs:507`.
- `testing/jig-fake-workers/src/lib.rs` (`Assignment`, :17-22) and
  `testing/jig-fake-workers/src/tests.rs:4`.
- The brief: `crates/jig-domain-brief/src/planned.rs` (:19-20, :149-154,
  the test at :257-269), `crates/jig-domain-brief/src/orchestrated.rs:532`.
- The ops world's scripted agent (`tests/ops/src/conformance/scripts.rs`,
  `answer` :90-125): the start's messages become its unread messages.
Shape:
- `jig_domain_tasks::Limits::run_inbox: u32`, doc "Messages a run's
  inbox takes: the most its start carries, and the most relayed and
  unread (tasks.md, 8.2)"; `worst_case` refuses `run_inbox == 0` and
  `run_inbox > inbox_messages`.
- `Now::RunPrepared`'s `inbox` becomes `messages: Box<[tasks::Word]>`:
  the first `run_inbox` words of `context.inbox` (already oldest first,
  `crates/jig-domain-tasks/src/domain.rs:748-808`).
- `RunProof.offered` is the last of `messages`' numbers, or `None`
  (`run_preparation.rs:93-96`), replacing `context.last_message`.
- `ops`'s `assemble::assignment` builds the hub's
  `Assignment.messages` from them (`StartMessage { name: Token::new(word.number),
  sender: assemble::sender(word.from), words: assemble::inbox_words(&word) }`),
  replacing jig 05's empty start; `domain::Assignment.inbox` and
  `offered_inbox` go.
- The test root's `Assignment.inbox` becomes `messages`;
  `jig_fake_workers::Assignment` gains `messages: Box<[u64]>`, their
  names, which `tests/core/src/peers.rs` fills.
- `brief::Core::Inbox` goes, with its rule that keeps a required inbox
  section uncut.
Test shape:
- `tests/core/tests/runs.rs` (245 lines; scenario functions called from
  `runs_scenarios`, :236), with `World::new` (`tests/core/src/effects.rs:161`),
  `World::say` (:1193) and `World::configured` (:175):
  `a_woken_chats_start_carries_its_waiting_words_oldest_first`,
  `a_start_carries_at_most_the_runs_inbox` (with
  `limits.core.tasks.run_inbox = 1`, set on `effects::fixture`'s limits,
  :112). Each under 0.5 s.
- `tests/tasks/tests/inbox.rs`: `a_parked_chat_wakes_on_its_persons_words`
  (:63-71) keeps reading `contexts[&1].inbox`. No new test there.
- `crates/jig-domain-brief/src/planned.rs`: `a_required_inbox_is_kept_whole`
  (:257-269) goes with the section.
Proof: world, `heavy cargo nextest run -p jig-core-world --test runs`,
once: a chat woken by a person's words starts with those words in its
start, oldest first, and with no more than the run's inbox; under 0.5 s
per scenario.
Don't: put messages in the brief; cut a message to fit (a message too
large for the start stays for a relay);
change the hub's or the fleet's boundaries.
Size: at most 120 lines of production code.
Removes: `brief::Core::Inbox`; `ops`'s `offered_inbox` and
`domain::Assignment.inbox`; jig 05's empty start in `ops`.

### 04.2 Relays stay within the run's inbox

Scope: while a run is claimed or running, a message is relayed only
while the run has fewer than `run_inbox` messages relayed and unread;
the rest wait in the inbox, in order, and are relayed as turns read
(tasks.md, 8.2; engine.md, 7.4; findings/domain.md, 6.4, gaps 2 and 3).
Needs: nothing beyond 04.1.
Files:
- `crates/jig-domain-core/src/routing/tasks.rs` (the
  `tasks::Request::Relay` arm, :41-43; 1,414 lines).
- `crates/jig-domain-core/src/routing/children.rs` (`fleet::Request::Inbound`,
  :243-267; `Undelivered`, :279-282).
- `crates/jig-domain-core/src/routing/runs.rs` (`route_accepted_turn`,
  :681-719; 784 lines).
- `crates/jig-domain-core/src/sibling_routes.rs` (`relay_previous`,
  :439-459), `crates/jig-domain-core/src/lib.rs` (`relaying`, :265).
- `crates/jig-domain-tasks/src/domain.rs` (1,251 lines): the query
  `next_waiting`, beside `escalation` (:413), in a new module
  `crates/jig-domain-tasks/src/waiting.rs` that reuses the ordering of
  `activate` (:748-808).
- The test root's relay (`testing/jig-test-domain/src/lib.rs:913-916`,
  `relay_message` :1061).
- New module `crates/jig-domain-core/src/relays.rs` holds the window.
Shape:
- `jig_domain_tasks::Domain::next_waiting(&self, task: u64, after: Option<u64>) -> Option<Word>`:
  the oldest message waiting for the task (inbox words, and proposals
  and escalations waiting for its decision) numbered past `after`.
- `relays::room(core, task) -> bool`: fewer than `run_inbox` words of
  the task's inbox numbered at most the run's `RunProof.offered`, the
  attempt's refused ones (04.3) aside.
- The `tasks::Request::Relay` arm emits `Held::Relay` only with room and
  no relay of the run in flight (`core.relaying`); otherwise the word
  waits.
- After a relay's offer commits (`children.rs:259-266`) and after an
  accepted turn (`route_accepted_turn`), the core relays
  `next_waiting(task, proof.offered)` when there is room.
Test shape:
- `tests/core/tests/runs.rs`, with `World::configured`, `World::say`,
  `World::turn` (:1215) and the field `World.inbound`
  (`tests/core/src/effects.rs:33`, filled at :718). Reference scenario:

  ```rust
  fn relays_wait_while_the_runs_inbox_is_full() {
      let (configuration, mut limits) = jig_core_world::effects::fixture(109, false);
      limits.core.tasks.run_inbox = 1;
      let mut world = World::configured(109, false, configuration, limits);
      let first = world.say(82);
      let second = world.say(83);
      let relayed: Vec<u64> = world.inbound.iter().map(|word| word.number).collect();
      assert_eq!(relayed, vec![first]);
      world.turn(1, 1, Some(first), b"reads the first words");
      assert_eq!(world.inbound.last().map(|word| word.number), Some(second));
  }
  ```

  called from `runs_scenarios`. Other scenario:
  `a_run_reading_nothing_is_relayed_nothing_more`. Each under 0.5 s.
- New step module `crates/jig-domain-tasks/src/tests/waiting.rs`,
  declared beside `mod revisions;` (`crates/jig-domain-tasks/src/tests.rs:22`):
  `next_waiting_merges_words_and_decisions_in_number_order`. Under
  20 ms.
Proof: world, `heavy cargo nextest run -p jig-core-world --test runs`,
once: a person's second words reach a run whose inbox holds one only
after the run reads the first; under 0.5 s per scenario.
Don't: drop a waiting message; relay a message twice to one attempt;
change `wake::after_message`'s judgement.
Size: at most 120 lines of production code.
Removes: nothing.

### 04.3 Refusals are committed with the turn that carries them

Scope: a turn's commit takes the messages its fence covers but those it
says the run refused; the refused stay in the inbox, recorded for the
attempt, so no later fence of the same attempt takes them, across a
restart too; an answer's fence and refusals are checked against what
the run was offered and read (engine.md, 7.2; hosts.md, 6.4;
findings/domain.md, 6.3, gaps 4 and 5).
Needs: jig 05's turns carrying refusals and the answer's fence
(`core::Event::TurnPayload.refused`, `Event::AnswerPayload.read` and
`refused`).
Files:
- `crates/jig-domain-core/src/routing/runs.rs` (`route_turn_payload`,
  :760-784, and `route_answer_payload`, :721-758, which jig 05 left
  dropping the fields).
- `crates/jig-domain-tasks/src/boundary.rs` (1,776 lines): `Event::Turn`
  (:1372-1389), `TaskRecord` (:844).
- `crates/jig-domain-tasks/src/admission.rs` (`turn`, :42-116),
  `crates/jig-domain-tasks/src/inbox.rs` (`take`, :456-466; its callers
  `admission.rs:87` and `procedure.rs:60`), `crates/jig-domain-tasks/src/domain.rs`
  (the destructure at :614), `crates/jig-domain-tasks/src/stored.rs`
  (1,049 lines; the task record's restore checks).
- `tasks::Event::Turn` literals (13): `crates/jig-domain-core/src/routing/runs.rs:772`;
  `crates/jig-domain-core/src/tests/live_delegates.rs:35`;
  `crates/jig-domain-core/src/tests/read_filters.rs:188`;
  `crates/jig-domain-tasks/src/tests/revisions.rs:325,455`;
  `tests/tasks/src/world.rs:1105` (1,165 lines); `tests/tasks/tests/activity.rs:47`;
  `tests/tasks/tests/funding.rs:15`; `tests/tasks/tests/inbox.rs:36,48`;
  `tests/tasks/tests/memory.rs:168,179,344`; matched at
  `tests/tasks/src/world.rs:385,513` and `tests/tasks/src/coverage.rs:50`.
- `TaskRecord` literals (5): `crates/jig-domain-tasks/src/domain.rs`,
  `tests/core/src/stories/closing_chat.rs`, `tests/core/src/stories/instructions.rs`,
  `tests/core/tests/stories.rs`, `tests/tasks/src/world.rs`.
- The core world: `tests/core/src/effects.rs` (`World::turn`, :1215;
  1,289 lines) and `testing/jig-fake-workers/src/lib.rs` (`Script::Turn`,
  :58), for a run that refuses.
- When jig 14 has landed the tasks family's codec: its schema, its
  generated codec and its translation in `jig-protocol-store`.
Shape:
- `tasks::Event::Turn` gains `refused: Box<[u64]>`, admitted only if each
  is a message the attempt was offered (`offered`) and still in the
  inbox, otherwise `Refusal::Read`.
- `inbox::take(task: &mut TaskRecord, read: Option<u64>, refused: &[u64])`
  keeps the refused words; the procedure's caller passes `&[]`.
- `TaskRecord.refused: Box<[u64]>`, doc "Messages the current attempt's
  run refused; untaken, and never taken by a later fence of that
  attempt"; extended at each turn, cleared when the attempt is claimed
  and when its run ends (04.4); restored with the task.
- `route_turn_payload` passes `refused`; `route_answer_payload` admits
  the answer as `tasks::End::Failed(tasks::Class::Invalid)` when its
  `read` passes the last committed turn's read (`proof.turn`) or a name
  in `refused` was not offered.
- `World::turn_refusing(number, spent, read, refused: &[u64], body)` in
  the core world; `jig_fake_workers::Script::Refuse { name: u64 }`, which
  the fake's next turn carries.
Test shape:
- `tests/tasks/tests/inbox.rs` (538 lines), with `say` (:8) and
  `World::claim` (`tests/tasks/src/world.rs:852`). Reference test:

  ```rust
  #[test]
  fn a_refused_word_survives_a_later_fence() {
      let mut world = World::new(91, LIMITS);
      world.make(Party::Person(9), vec![task(1, &[])]);
      world.claim(1, 1);
      say(&mut world, 1);
      say(&mut world, 2);
      let call = 1003;
      world.send(Event::Turn {
          reply_to: ReplyTo::new(Token::new(call)),
          task: 1,
          attempt: 1,
          turn: 1,
          read: Some(2),
          refused: Box::new([1]),
          offered: Some(2),
          cumulative: 1,
      });
      assert_eq!(world.replies[&call], Reply::Turn(Accepted::New));
      assert_eq!(world.record(1).inbox.len(), 1);
      assert_eq!(world.record(1).inbox[0].number, 1);
  }
  ```

  Others: `a_refusal_naming_an_unoffered_word_is_refused_as_read`,
  `a_refused_word_survives_a_restart_and_a_later_fence`. Each under
  0.5 s.
- `tests/core/tests/runs.rs`: `an_answer_whose_fence_passes_its_last_turn_fails_invalid`.
  Under 0.5 s.
- `tests/tasks/src/referee.rs`: the rule "no committed message is lost"
  gains "none taken while refused".
Proof: world, `heavy cargo nextest run -p jig-tasks-world --test inbox`,
once: a message a run refused stays in its task's inbox after a later
turn whose fence passes it, and after a restart; under 0.5 s per test.
Don't: take a message at an answer; keep refusals past their attempt.
Size: at most 120 lines of production code.
Removes: the dropping of `refused`, `read` in the core's routes (jig
05's transitional).

### 04.4 A refused or unread message wakes its task again as the run ends

Scope: when an agent run ends and its task stays live, each message
still in the task's inbox (refused, left unread past the answer's fence,
or never relayed) is judged again as if it arrived then: it wakes the
task if its kind does, and otherwise waits; the attempt's refusals are
cleared (tasks.md, 8.2 and 8.3; hosts.md, 9; engine.md, 7.4;
findings/domain.md, 6.5).
Needs: nothing beyond 04.3.
Files:
- `crates/jig-domain-tasks/src/run.rs` (`activation`, :300-441; the
  Parked, Failed and Refused arms at :400-418).
- `crates/jig-domain-tasks/src/wake.rs` (`after_message`, :114-183;
  `after_step`, :184-196).
- `tests/core/tests/runs.rs`, `tests/tasks/tests/inbox.rs`,
  `tests/tasks/src/referee.rs`; the ops world's scripted agent comment
  at `tests/ops/src/conformance/scripts.rs:27-28,115-118`.
Shape:
- `wake::after_run(domain: &mut Domain, env: &Env<Limits>, number: u64, out: &mut Queue<Request>)`:
  clears `TaskRecord.refused`, then calls
  `after_message(domain, env, number, None, word, out)` for each word of
  the inbox while the task is `Active::Idle`, stopping once it is due,
  as `after_step` does.
- `run::activation` calls it after an agent run's Parked, Failed and
  Refused arms leave the task live.
Test shape:
- `tests/core/tests/runs.rs`, with `World::turn_refusing` (04.3) and
  `World::end` (`tests/core/src/effects.rs:1228`). Reference scenario,
  the replacement of jig 05's removed
  `a_bounced_message_stays_unread_for_a_later_read`:

  ```rust
  fn a_refused_message_survives_a_later_fence_and_wakes_its_task_as_the_run_parks() {
      let mut world = World::new(108, false);
      let first = world.say(80);
      let second = world.say(81);
      let attempt = world.assigned.as_ref().expect("assigned chat").attempt;
      world.turn_refusing(1, 1, Some(second), &[first], b"reads the second words only");
      world.end(tasks::End::Parked, 1);
      let woken = world.assigned.as_ref().expect("woken again");
      assert!(woken.attempt > attempt, "{:?}", world.trace);
      assert_eq!(woken.messages.first().map(|word| word.number), Some(first));
  }
  ```

  called from `runs_scenarios`. Other scenarios:
  `a_message_left_unread_wakes_its_task_as_the_run_parks` (engine.md,
  15), `quiet_news_left_unread_waits_for_the_next_activation`. Each
  under 0.5 s.
- `tests/tasks/tests/inbox.rs`:
  `a_persons_words_left_unread_wake_the_task_as_its_run_ends`
  (tasks.md, 12). Under 0.5 s.
Proof: world, `heavy cargo nextest run -p jig-core-world --test runs -E 'test(runs_scenarios)'`,
once: a person's words a run refused, or left unread as it parked, start
the chat's next run; under 0.5 s.
Don't: wake a task on a kind its policy does not wake on; re-judge
anything at a run that closes its task.
Size: at most 40 lines of production code.
Removes: nothing.

## Interface sketches

```rust
// crates/jig-domain-tasks/src/limits.rs (jig_domain_tasks)
/// The tasks child's limits (as built, plus:).
pub struct Limits {
    /// Messages a run's inbox takes: the most its start carries, and the
    /// most relayed and unread (tasks.md, 8.2). At most `inbox_messages`.
    pub run_inbox: u32,
    /// Maximum whole unread word messages kept by one live task (as built).
    pub inbox_messages: u32,
    // … the other fields as built.
}
```

```rust
// crates/jig-domain-tasks/src/boundary.rs (jig_domain_tasks)
use skein_lib::ReplyTo;

/// Parent to tasks (as built, but:).
pub enum Event {
    /// Admit the next contiguous turn, its read fence, its refusals and its
    /// spend atomically; answered `TurnAcknowledged` or `Refused`.
    Turn {
        reply_to: ReplyTo,
        task: u64,
        attempt: u64,
        turn: u32,
        /// Highest message read; taken with the turn, but `refused`.
        read: Option<u64>,
        /// Messages the run refused since its previous turn, by number;
        /// each offered to this attempt and still in the inbox.
        refused: Box<[u64]>,
        offered: Option<u64>,
        cumulative: u64,
    },
    // … the other variants as built.
}

/// A live task (as built, plus:).
pub struct TaskRecord {
    /// Messages the current attempt's run refused: untaken, and never
    /// taken by a later fence of that attempt; cleared when its run ends.
    pub refused: Box<[u64]>,
    // … the other fields as built.
}
```

```rust
// crates/jig-domain-core/src/routing/vocabulary.rs (jig_domain_core)
use jig_domain_tasks as tasks;

/// Core output that follows no mutation (as built, but:).
pub enum Now {
    /// Core-owned run parts, for the root's assembly of the assignment.
    RunPrepared {
        task: u64,
        attempt: u64,
        charter: u32,
        run: Box<crate::RunCharter>,
        /// The start's messages: the oldest waiting, at most `run_inbox`.
        messages: Box<[tasks::Word]>,
        saved: Box<[tasks::SavedResource]>,
        transcript: Box<[Box<[u8]>]>,
        answered: Box<[crate::CallKey]>,
        grant: jig_domain_accounts::Grant,
    },
    // … the other variants as built.
}
```

The tasks child owns the inbox, what a turn takes and the wake
judgement; the core owns the offered fence and the relay window; the
roots only put the start's messages into the hub's assignment. A
message ends read (taken by a turn's commit), or stays: refused,
unread, or waiting.

## Questions to raise when reached

- **A message larger than the hub's bound** cannot be in a start or a
  relay; the design says every message fits (tasks.md, 8.2), and jig
  20's configuration and jig 05's fleet check hold it. If a world
  reaches one, raise it rather than cutting the message.
- **Proposals and escalations in the relay window:** they wake the task
  always (tasks.md, 9); whether one waits behind a full window or is
  relayed past it is not said. The default here: it waits, like any
  message. Raise it if a story needs otherwise.

## Done when

- The brief has no inbox section: `git -C ~/src/rust/jig grep -n 'Core::Inbox' -- crates examples`
  prints nothing.
- `ops` replays no inbox after an assignment: `git -C ~/src/rust/jig grep -n 'offered_inbox' -- examples`
  prints nothing.
- A turn's refusals reach the tasks child: `git -C ~/src/rust/jig grep -n 'refused' -- crates/jig-domain-tasks/src/inbox.rs crates/jig-domain-tasks/src/admission.rs`
  prints at least two lines.
- The scenarios this session names are in `runs_scenarios`:
  `git -C ~/src/rust/jig grep -n -E 'relays_wait_while_the_runs_inbox_is_full|a_refused_message_survives_a_later_fence' -- tests/core/tests/runs.rs`
  prints four lines.
- The gate passes on the last landed commit:
  `git -C ~/src/rust/jig log -1 --format='%(trailers:key=Gate-Result,valueonly)' origin/main`
  names `scripts/check.sh passed`.
