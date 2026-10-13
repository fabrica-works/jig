# Session 03: people's and tasks' refusals, timers and signing out

The people's and tasks' items the design asks of the domain and the code
lacks (findings/domain.md, item 10): a party's reply names only its own
sign-in, and the core stops keeping the last one; the people child's
timers fire in the conformance and `ops` worlds too; a decision on an
escalation no longer waiting is refused as not waiting, not as unknown; a
task's proposal that nobody may accept is refused to its proposer, and a
party's words stay within what a run takes as one message; a chat ended
and read is forgotten by the core's own decision; a read on an ended
sign-in is answered signed out; and a person signs out from the frame.
jig 30, 31 and 32 carry the sign-in's number, the signed-out read and the
sign-out on the client's wire, so they need this session first.

Paths carry jig 01's names (`crates/jig-domain-core/` was
`crates/jig-core/`, `crates/jig-domain-people/` was
`crates/jig-core-people/`, `crates/jig-domain-client/` was
`crates/jig-client/`, `crates/jig-view/` was `crates/jig-client-view/`,
`testing/jig-test-domain-client/` was `testing/jig-test-client-domain/`,
`examples/ops/domain-client/` was `examples/ops/client-domain/`). Line
numbers are as built at jig `1cdfeb62`; jig 01's renames may move them
by a few lines.

## Read first

- jig's `domain/people.md`, section 3 (sign-ins), 5.1 (requests: "Words
  are never cut", "A proposal needs someone who may accept it", "A hold's
  revision moves with the hold", "Two parties deciding one thing") and
  section 6 (a chat forgotten).
- jig's `domain/tasks.md`, 8.2 ("Within what a run reads") and section 9
  ("A proposal nobody may accept is refused").
- jig's `domain/engine.md`, 7.3 (`propose` answers "refused, when nobody
  may accept it").
- jig's `client/domain/client.md`, section 12 (signing in and out);
  `client/domain/contract.md`, section 4 (answers) and section 6 (reads
  refused signed out).
- jig's `protocol/configuration.md`, section 6 ("The deployment's message
  bound").
- findings/domain.md, item 10 (10.1 to 10.6 and 10.8) and its "Note on
  10.8 and 11.6".
- findings/protocol.md, 1.3 (the client's boundary) and 6.1 (the old
  draft's sessions 03 and 08).

## What the design asks

- **The domain names a sign-in only by its number** (people.md, 3): the
  reply to a sign-in names the one it made; no other reply names one.
- **Sign-ins expire and answered keys are forgotten** on the people
  child's timers (people.md, 3; contract.md, 3.1), in each world that
  runs the core.
- **Deciding an escalation needs it to wait for the party** (people.md,
  5.1): "deciding the escalation needs it to wait for the party, and is
  refused as not waiting for them otherwise"; "the first commit decides;
  the second is told it was decided, by whom, and how".
- **A proposal nobody may accept is refused** (tasks.md, 9): "When the
  first holder it would wait for is the parties the project's policy
  names ... and the policy names no role that may accept its kind, it is
  refused to its proposer as beyond authority, saying that no proposal
  could be accepted, and nothing of it is committed ... a run's `propose`
  call is answered so; a procedure's step's proposal is refused in its
  decision. A proposal already pending waits as this section says."
- **Words are never cut** (people.md, 5.1): "A party's words ... are
  within the people child's bound, which is at most what a run admits as
  one message".
- **A chat is forgotten** (people.md, 6): "once it has ended and its
  person has read its replies and its result; its record and its read
  position are erased in one decision".
- **Reads refused signed out** (contract.md, 6): "when the sign-in has
  ended, as signed out".
- **Signing out, from the frame** (client.md, 12): "it ends the sign-in
  on the engine ..., and the client then shows the sign-in page, as when
  signed out." A service has no frame.

## Choices this plan makes

- The sign-in's number travels on `people::Reply::SignedIn` only; `Held::PeopleReply` carries none.
- A task's refused proposal reuses the party's refusal, `Authority { lacking: Approver, proposable: Some(false) }`, through a new `tasks::Refusal::Unproposable`.
- The people child's word bound must be at most the tasks child's message bound: `Core::new` asserts it.
- Forgetting is the core's decision, made at the mark that completes the last condition; the root only loads the ended row.
- At a results mark the core looks only at the chats the person has a read position for (the people child's `chats_seen`).
- A read on an unknown, ended or expired sign-in, once the people child is ready, is `Why::SignedOut`; before it is ready it stays `Unknown`.
- The client's sign-out is `Act::SignOut` and `Request::SignOut`, with no field; the client changes nothing until the shell answers `Event::SignedOut`, which it already takes.

## Rules that bind it

- From code-organization.md, section 6: "No increment of this plan: ...
  adds a test that checks several behaviours under one name; ... computes
  an expected value with the code under test;".
- From code-organization.md, section 4: "the core's vocabulary toward the
  root (`routing/vocabulary.rs`) | `jig-domain-core` | each session adds
  its variants; hotspot".
- From code-organization.md, section 5: "Ceilings per test, unoptimized
  on the idle machine: a unit test under 20 ms, a focused world test
  under 0.5 s, a fuzzy test about 5 s." and "the focused suite has about
  6 s left for the whole plan."
- For this session: a file already over 800 lines (named in each
  increment's Files) must not grow by more than 20 lines in an
  increment; new code goes in a module of its own (code-organization.md,
  section 7). `git diff --stat` in the commit body shows it.

## Increments

### 03.1 A reply names its own sign-in; the people child's timers fire everywhere

Scope: the reply to a sign-in, and to its replay, names the sign-in it
made; no other reply names one; `Core::signing_in` goes, so
`Core::quiescent` is true again once a sign-in's reply has left
(people.md, 3; findings/domain.md, 10.1). The conformance world, the
`ops` world and the core world's `World::advance` fire `Timer::People`
(findings/domain.md, 10.8 and its note).
Needs: nothing.
Files:
- `crates/jig-domain-people/src/boundary.rs` (`Reply::SignedIn`, :504;
  781 lines, +3), `crates/jig-domain-people/src/domain.rs` (:815, :879;
  1,873 lines).
- `crates/jig-domain-core/src/lib.rs` (`signing_in` :267, :537, :636),
  `crates/jig-domain-core/src/routing/people.rs` (:70-73, :80, :131,
  :169), `crates/jig-domain-core/src/routing/vocabulary.rs`
  (`Held::PeopleReply`, :140; hotspot).
- The users of `Held::PeopleReply`'s `sign_in` (20 sites in 17 files):
  `crates/jig-domain-core/src/tests/restore_admission.rs:22-24`;
  `examples/ops/domain/src/tests.rs:338`;
  `tests/core/src/client_peer.rs:57,60`;
  `tests/core/tests/client_peer_routes.rs:52-61,148,150`;
  `tests/core/src/observations.rs:864-867`;
  `tests/core/src/world.rs:593-594`;
  `tests/core/src/effects.rs:672-673,778-779` (1,289 lines);
  `tests/core/tests/client_peer.rs:31-32`;
  `tests/core/src/referee/offers.rs:112-113`;
  `tests/core/tests/restore_pages.rs:38`;
  `tests/conformance/src/lib.rs:511-513`;
  `tests/ops/src/conformance/mod.rs:519-523` (975 lines);
  `tests/ops/src/joint/front.rs:533` (1,035 lines);
  `tests/client/src/joint/requests.rs:245`;
  `tests/client/src/joint/front.rs:428-429`;
  `tests/client/src/joint/engine.rs:185,273`;
  `tests/client/src/joint/session.rs:627-632` (1,553 lines).
- Helpers threading the `Option<u64>`: `tests/core/src/peers.rs:307`
  (`Peers::reply`) and :367 (`party_reply`);
  `tests/client/src/joint/packets.rs:292` (`people_reply`, 1,104 lines).
- `Reply::SignedIn` literals without `..` (7): 
  `crates/jig-domain-people/src/tests.rs:210`;
  `tests/core/tests/joint_clock.rs:32,65`;
  `tests/client/src/joint/wire/answers.rs:28,95`;
  `tests/client/src/joint/engine.rs:189`;
  `tests/client/src/joint/packets.rs:869`.
- Timers: `tests/conformance/src/lib.rs:389-396` (`timers`),
  `tests/ops/src/conformance/mod.rs:225-233` (`timers`),
  `tests/core/src/effects.rs:411-417` (`World::advance`).
Shape:
- `jig_domain_people::Reply::SignedIn { person: u64, sign_in: u64, expires: Wall }`:
  `domain.rs:815` (the replay) passes `number`, `domain.rs:879` passes
  `number`.
- `jig_domain_core::Held::PeopleReply { to: ReplyTo, reply: people::Reply }`.
- `Core::signing_in` removed; `routing/people.rs::sign_in` stops setting
  it; `tag_people` builds `Held::PeopleReply { to, reply }`.
- The worlds and the joint world read a sign-in's number from
  `Reply::SignedIn { sign_in, .. }`; `Peers::reply`, `party_reply` and
  `people_reply` lose their `Option<u64>` parameter.
- The three `timers` lists and `World::advance` gain
  `Timer::People` (`root::Event::Timer(core::Timer::People)`).
Test shape:
- New step module `crates/jig-domain-core/src/tests/sign_ins.rs`,
  declared beside `mod engine_timers;` in
  `crates/jig-domain-core/src/tests.rs:2930`; it reuses `timer_core`
  (`crates/jig-domain-core/src/tests/engine_timers.rs:13`), made
  `pub(super)`. Reference test:

  ```rust
  //! A sign-in is named only by its own reply (`domain/people.md`, section 3).
  use super::engine_timers::timer_core;
  use super::*;

  fn newcomer() -> people::Identity {
      people::Identity {
          key: people::IdentityKey { provider: 0, subject: Box::new([2]) },
          login: Box::from(&b"newcomer"[..]),
          name: Box::from(&b"A Newcomer"[..]),
      }
  }

  #[test]
  fn a_sign_in_reply_names_the_sign_in_its_decision_saved() {
      let (mut core, env) = timer_core(false);
      let Requests::Out(mut out) =
          step(&mut core, &env, Event::SignIn { reply_to: ReplyTo::new(Token::new(60)), identity: newcomer() });
      let mut saved = None;
      let mut named = None;
      while let Some(request) = out.pop() {
          if let Request::Write(Write::Save(Record::People(people::Stored::SignIn { number, .. }))) = &request {
              saved = Some(*number);
          }
          if let Request::Held(held) = request
              && let Held::PeopleReply { reply: people::Reply::SignedIn { sign_in, .. }, .. } = *held
          {
              named = Some(sign_in);
          }
      }
      assert!(saved.is_some(), "the decision saved one sign-in");
      assert_eq!(named, saved);
  }
  ```

  Other tests there: `the_core_is_quiescent_once_a_sign_in_reply_has_left`,
  `a_replayed_live_sign_in_names_the_same_sign_in`. Each under 20 ms.
- `tests/core/tests/client_peer_routes.rs`: `replies` (:52) returns
  `(u64, Token, people::Reply)`; `signed_in` (:64) takes the number from
  `Reply::SignedIn { sign_in, .. }`;
  `interleaved_destinations_route_by_registration_even_for_the_same_person`
  (:131) asserts the opposite of the design at :148 and :150 (each
  `MarkedSeen` reply names the second sign-in): those two assertions go,
  and the test keeps its other assertions. Under 0.5 s.
- `tests/conformance/tests/scenarios.rs`: new test
  `a_sign_in_past_its_lifetime_is_answered_signed_out`, which advances
  the harness's clock (`advance`,
  `testing/jig-conformance/src/harness.rs:232`) past the configured
  `people.sign_in_lifetime` and asks once more. Under 0.5 s.
- `tests/core/tests/joint_clock.rs`: `World::advance` firing
  `Timer::People` changes no test there; the gate shows it.
Proof: world, `heavy cargo nextest run -p jig-conformance-world --test scenarios -E 'test(a_sign_in_past_its_lifetime_is_answered_signed_out)'`,
once: a party whose sign-in expired in the conformance world is answered
signed out; under 0.5 s. (The step test shows the reply's number; the
conformance world is the highest layer that runs the people child's
timer under an application's root.)
Don't: make any reply other than `SignedIn` carry a sign-in; add a
`Now` or `Held` that names an ended sign-in (findings/domain.md, 10.8:
the erase in the decision is what the protocol layer reads).
Size: at most 40 lines of production code.
Removes: `Core::signing_in`; `Held::PeopleReply`'s `sign_in`; the
`Option<u64>` parameters of the three helpers; the assertions at
`client_peer_routes.rs:148,150`.

### 03.2 A decision on an escalation no longer waiting is refused as not waiting

Scope: a party deciding an escalation that waits for nobody, on a task
they may see, is refused `NotWaiting`; one decided by another party is
still told it was decided, by whom and how; `Unknown` stays for what the
party may not see (people.md, 5.1; findings/domain.md, 10.2).
Needs: nothing.
Files:
- `crates/jig-domain-core/src/acts/objects/decisions.rs` (`escalation`,
  :132; the arm at :144-149).
- `crates/jig-domain-core/src/routing/party_decisions.rs`
  (`person_escalation_decide`, :313; the arm at :339-348).
- `crates/jig-domain-core/src/routing/historical.rs`
  (`route_historical_escalation`, :11; the `row: None` arm at :67-68).
- `crates/jig-domain-core/src/routing/tasks.rs` (the
  `tasks::EscalationOutcome::Stale` arm at :88-99; 1,414 lines, +10 at
  most).
- Tests: `crates/jig-domain-core/src/tests/acts.rs` (973 lines; the
  escalation offers at :611), new `tests/core/tests/refusals.rs`.
Shape:
- `acts::escalation`: `tasks::Escalation::Unheld | Routing | Rejected`
  on a held task the party may see answers
  `people::Refusal::NotWaiting`, so the escalation document offers it
  for `Release`, `LeaveHeld` and `PassUp`.
- `person_escalation_decide`: `pending == false` answers `NotWaiting`.
- `tasks::EscalationOutcome::Stale` ("No exact live pending revision;
  root may read its own historical outcome") answers as
  `escalation_request` (`routing/party_decisions.rs:438`) does past its
  current check: through `Now::HistoricalEscalation`, so a decision
  recorded since is told by whom, and none is `NotWaiting`.
- `route_historical_escalation`, when no decision record is found: if
  `core.tasks.task(task)` is live in `project` and the party holds a role
  there (the check `documents::small::visible` makes,
  `crates/jig-domain-core/src/documents/small.rs:14`), the outcome is
  `Refused(NotWaiting)`; otherwise `Refused(Unknown)`. A record found
  answers `EscalationDecided` as built.
- No vocabulary change: `people::Refusal::NotWaiting` exists
  (`crates/jig-domain-people/src/refusals.rs:29`) and is mapped by
  `tests/core/src/peers.rs:595`, `tests/client/src/joint/wire/refusals.rs:26`,
  `crates/jig-domain-client/src/contract/refusals.rs:159` and
  `crates/jig-view/src/words/refusals.rs:25`.
Test shape:
- New file `tests/core/tests/refusals.rs`, its helpers private to it
  (`tests/core/tests/stories.rs`, 1,886 lines, keeps its own). Fixtures:
  `World::new` (`tests/core/src/effects.rs:161`), `World::end` (:1228),
  `World::call_key` (:858). Reference test:

  ```rust
  use jig_core_world::effects::World;
  use jig_domain_core as core;
  use jig_domain_people as people;
  use jig_domain_tasks as tasks;
  use jig_test_domain as root;
  use skein_lib::{ReplyTo, Token};

  fn ask(world: &mut World, request: u8, ask: people::Ask) {
      world.send(root::Event::Core(core::Event::People(people::Event::Ask {
          reply_to: ReplyTo::new(Token::new(8000 + u64::from(request))),
          sign_in: world.sign_in.expect("signed-in person"),
          key: [request; 16],
          ask,
      })));
  }

  fn hold_revision(world: &World, task: u64) -> u64 {
      match world.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Live(task)))) {
          Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))) => row.revisions.hold,
          row => panic!("live task row missing: {row:?}"),
      }
  }

  #[test]
  fn deciding_an_escalation_after_its_release_is_refused_as_not_waiting() {
      let mut world = World::new(152, false);
      let task = world.call_key(1).task;
      ask(&mut world, 60, people::Ask::Stop { project: 1, task });
      world.end(tasks::End::Parked, 0);
      let revision = hold_revision(&world, task);
      ask(&mut world, 61, people::Ask::Release { project: 1, task, revision, again: None });
      ask(
          &mut world,
          62,
          people::Ask::DecideEscalation { project: 1, task, revision, decision: people::EscalationDecision::Pass },
      );
      assert_eq!(
          world.people_answers.last(),
          Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotWaiting)))
      );
  }
  ```

  Other tests there: `deciding_an_escalation_another_party_decided_says_by_whom`
  (two sign-ins, as `tests/core/tests/stories.rs:18` `ask_as` does);
  `deciding_an_escalation_of_an_unseen_project_is_unknown`. Each under
  0.5 s.
- `crates/jig-domain-core/src/tests/acts.rs`: new test
  `a_rejected_escalation_still_held_offers_not_waiting`, beside
  `leaving_an_escalation_held_closing_a_chat_and_marking_it_seen_are_admitted`
  (:611), with `sign_in` (:520) and `ask_now` (:538). Under 20 ms.
Proof: world, `heavy cargo nextest run -p jig-core-world --test refusals`,
once: a person deciding an escalation that a release ended is told it no
longer waits for them, and one decided by another is told by whom; three
tests, each under 0.5 s.
Don't: change `tasks::Escalation` or `people::Refusal`; answer
`NotWaiting` for a task the party may not see.
Size: at most 40 lines of production code.
Removes: the `Unknown` answers for escalations the party may see but
that wait for nobody.

### 03.3 A proposal nobody may accept is refused; words within a run's message

Scope: a run's proposal (the `propose` call, a batch proposal, an
effect's proposal) whose first holder would be the policy's parties, in a
project whose policy names no role that may accept its kind, is answered
refused, saying no proposal could be accepted, and nothing of it is
committed; a procedure's step's proposal in that case is refused in its
decision; a proposal already pending and routed again still waits
(tasks.md, 9; engine.md, 7.3; findings/domain.md, 10.4). `Core::new`
refuses limits whose party word bound exceeds the tasks child's message
bound (people.md, 5.1; tasks.md, 8.2), and the core's own news is never
cut.
Needs: nothing.
Files:
- `crates/jig-domain-core/src/acts/bounds.rs` (`recipient`, :11, made
  `pub(crate)`; its callers :91, :176, :278, :372).
- `crates/jig-domain-core/src/routing/runs.rs` (`named_propose`, :246,
  the check after `holder` at :270; 784 lines, +10 at most), reached from
  `routing/runs.rs:558`, `routing/run_delegation.rs:233` and
  `effects.rs:961`.
- `crates/jig-domain-core/src/effects.rs` (`procedure_propose`, :1167,
  the check after `holder` at :1200; 1,441 lines);
  `projection_refused` (:1425-1441).
- `crates/jig-domain-tasks/src/boundary.rs` (`Refusal`, :1061; 1,776
  lines, +2): the new variant.
- `crates/jig-domain-core/src/routing/refusals.rs` (`task_answer`,
  :25-70): its one exhaustive map.
- `crates/jig-domain-core/src/lib.rs` (`Core::new`, :467).
- Tests: `tests/core/tests/refusals.rs` (03.2's file),
  `crates/jig-domain-core/src/tests/acts.rs` (beside :87).
Shape:
- `tasks::Refusal::Unproposable`, doc "No role of the project's policy
  may accept the proposal's kind; nothing of it is committed", mapped in
  `routing/refusals.rs` to
  `people::Refusal::Authority { lacking: people::Lack::Approver, proposable: Some(false) }`.
- `named_propose`: after `core.proposal_holder(..)` returns
  `tasks::ProposalHolder::Policy { project, kind }` and
  `acts::bounds::recipient(core, project, kind)` is `None`, it returns
  `proposal_refused(core, to, key, tasks::Refusal::Unproposable)`; the
  run hears `CallPart::ProposalRefused`.
- `procedure_propose`: the same case drops the effect with
  `authority::Answer::Refuse` and calls `procedure_wait`, as a missing
  holder does at :1200-1203.
- `proposal_holder` (`crates/jig-domain-core/src/proposals.rs:311`) is
  unchanged: the re-routing at `routing/tasks.rs:318,337`,
  `proposals.rs:243` and `routing/runs.rs:215` keeps a pending proposal
  waiting for the policy's parties.
- `Core::new` asserts
  `limits.people.words <= limits.tasks.message_bytes`;
  `projection_refused` loses its cut, and `Core::new` asserts that its
  news (`b"Projection write refused"`) fits `limits.tasks.message_bytes`.
Test shape:
- `tests/core/tests/refusals.rs`, with `effects::fixture`
  (`tests/core/src/effects.rs:112`) and `World::configured` (:175):
  `a_run_proposing_an_effect_nobody_may_accept_is_refused` (the policy's
  roles set to `decides: authority::Proposals(16)` and no grants, as
  `policy` at `crates/jig-domain-core/src/tests/acts.rs:24` sets
  `decides`; the run's `root::Event::EffectCall` with `proposal: Some(..)`,
  as `World::propose` at `tests/core/src/effects.rs:910` sends it; the
  answer is `CallPart::ProposalRefused` whose `why` is
  `tasks::Refusal::Unproposable`, and the store's rows are unchanged);
  `a_proposal_pending_when_the_policy_narrows_still_waits`. Each under
  0.5 s.
- `crates/jig-domain-core/src/tests/acts.rs`:
  `a_procedure_step_proposing_what_nobody_may_accept_is_refused` and
  `core_new_refuses_party_words_beyond_a_runs_message`
  (`#[should_panic]`). Each under 20 ms.
Proof: world, `heavy cargo nextest run -p jig-core-world --test refusals -E 'test(a_run_proposing_an_effect_nobody_may_accept_is_refused)'`,
once: a run asking for what nobody may accept is told no proposal could
be accepted, and the store is unchanged; under 0.5 s.
Don't: refuse a proposal already pending; add a refusal for the party's
path, which `acts/bounds.rs:384` has.
Size: at most 60 lines of production code.
Removes: proposals committed to wait for nobody; the cut in
`projection_refused`.

### 03.4 A chat ended and read is forgotten by the core

Scope: when a mark completes the last condition (the chat ended, its
replies read, its result read), the core forgets the chat in that
decision: its ended row and its read position erased together
(people.md, 6; findings/domain.md, 10.3). A mark over the chat's
replies forgets it when its result was read; a mark over results forgets
each ended chat the person has a read position for whose result it
passes. The half-forget at a chat's end goes.
Needs: nothing.
Files:
- `crates/jig-domain-core/src/inbox.rs` (`seen_chat_loaded`, :356-387;
  `forget_chat`, :490-517).
- `crates/jig-domain-core/src/routing/people.rs` (`Seen::Results` at
  :256-259; the live `Seen::Chat` at :260-270).
- `crates/jig-domain-core/src/routing/tasks.rs` (the half-forget at
  :686-697; 1,414 lines, shrinks).
- `crates/jig-domain-core/src/routing/vocabulary.rs` (hotspot): a new
  `Now::LoadEndedChat`.
- `crates/jig-domain-people/src/inbox.rs`: the accessor
  `Domain::marked_chats`.
- Exhaustive `Now` matches gaining the arm:
  `testing/jig-test-domain/src/routes.rs:513` (1,005 lines),
  `examples/ops/domain/src/route.rs:1325` (1,585 lines),
  `tests/core/src/client_peer.rs:152`, `tests/ops/src/joint/engine.rs:335`,
  `tests/ops/src/joint/front.rs:744`, `tests/client/src/joint/session.rs:789`,
  `crates/jig-domain-core/src/tests/completion_retry.rs:45`; the core
  world's loader beside `Now::SeenChat` (`tests/core/src/effects.rs:547`).
- Tests: `tests/core/tests/activity.rs` (219 lines).
Shape:
- `jig_domain_people::Domain::marked_chats(&self, person: u64) -> List<(u64, u64)>`:
  the person's chats and positions in `chats_seen`, at most
  `limits.inbox_tasks`.
- `Now::LoadEndedChat { task: u64 }`: the root loads
  `tasks::Key::Ended(task)` and answers with the existing
  `Event::ForgetChat { row }`, or with nothing when the row is gone.
- After `people::Event::ChatSeen` for an ended chat
  (`seen_chat_loaded`), the core pushes `Event::ForgetChat { row }` with
  the loaded row; `forget_chat` (:490) is the one judge, unchanged.
- `Seen::Results { through }`: for each `(task, _)` in
  `marked_chats(person)` whose task is not live, the core pushes
  `Now::LoadEndedChat { task }`, after the people child's
  `MarkedSeen` decision.
- `routing/tasks.rs:686-697`: no `people::Event::ForgetChat` at a chat's
  end.
Test shape:
- `tests/core/tests/activity.rs`:
  `all_replies_read_then_ended_survive_restart_and_forgetting` (:178)
  drops its own `world.send(root::Event::Core(core::Event::ForgetChat { row: owner }))`
  (:207): the results mark (request 62, :202-206) forgets the chat, and
  the test's assertions after it stay. New tests there, with
  `jig_core_world::reads::{ask, latest}` (`tests/core/src/reads.rs:375,402`)
  and `jig_core_world::stories::reads::request`
  (`tests/core/src/stories/reads.rs:43`):
  `a_chat_ended_with_its_result_unread_keeps_its_read_position`,
  `marking_an_ended_chats_replies_after_its_result_forgets_it`. Each
  under 0.5 s.
- `crates/jig-domain-people/src/tests.rs` (1,592 lines; new test in a
  new module `crates/jig-domain-people/src/tests/marked_chats.rs`):
  `marked_chats_lists_only_that_persons_positions`. Under 20 ms.
Proof: world, `heavy cargo nextest run -p jig-core-world --test activity`,
once: a person who has read an ended chat's replies and then its result
no longer sees the chat, and a restart finds neither its row nor its
read position; under 0.5 s per test.
Don't: forget a chat whose result or replies are unread; load more than
`limits.people.inbox_tasks` rows per mark; change `forget_chat`'s
conditions.
Size: at most 80 lines of production code.
Removes: the half-forget at `routing/tasks.rs:686-697`; the test's own
`ForgetChat` at `tests/core/tests/activity.rs:207`, and jig 12.1's at
`tests/core/tests/reads.rs:217`, each replaced by the person's marks
(ruling 118).

### 03.5 Reads answered signed out

Scope: a read naming a sign-in the ready people child does not know, or
one ended or expired, is answered `SignedOut`; an application's read
likewise; a read whose sign-in ends while it loads is answered
`SignedOut` (contract.md, 6; findings/domain.md, 10.6).
Needs: nothing.
Files:
- `crates/jig-domain-core/src/reads/vocabulary.rs` (`Why`, :304-311).
- `crates/jig-domain-core/src/reads.rs` (`refused_application`,
  :150-179; `start`, :182-185; `loaded`, :386-410).
- `tests/client/src/joint/read_packets.rs:38-42` (the one exhaustive
  match of `Why`).
- Tests: `tests/core/tests/reads.rs` (163 lines),
  `crates/jig-domain-core/src/tests/read_filters.rs`; the equality
  asserts at `crates/jig-domain-core/src/tests.rs:2445-2632` and
  `tests/core/tests/reads.rs:144` stay unless their case is now signed
  out.
Shape:
- `reads::Why::SignedOut`, doc "The read's sign-in is unknown to a ready
  engine, ended or expired."; `Why::Unknown`'s doc becomes "Missing, or
  invisible to the party".
- `refused_application(reply, read, why: Why)`; `start` passes
  `Why::SignedOut` when `core.people.ready()` and
  `core.people.person(sign_in, ..)` is `None`, and `Why::Unknown` when
  the people child is not ready.
- `loaded`: a sign-in that no longer names `pending.party` answers
  `SignedOut`; a scope the party may not see answers `Unknown`.
- The joint world maps `reads::Why::SignedOut` to
  `client::ReadNone::SignedOut`, which the client already takes
  (`crates/jig-domain-client/src/contract/reads.rs:222`).
Test shape:
- `tests/core/tests/reads.rs`, with `stories::reads::world`
  (`tests/core/src/stories/reads.rs:13`), `ask` and `latest`: new
  scenario functions called from
  `five_independent_read_stories_fit_the_shared_process_budget` (:151):
  `a_read_after_the_sign_in_ended_is_answered_signed_out` (the person's
  `people::Event::SignOut`, then a `reads::Read::Frame`);
  `a_read_whose_sign_in_ends_while_it_loads_is_signed_out`. Each under
  0.5 s.
- `crates/jig-domain-core/src/tests/read_filters.rs`:
  `a_read_before_the_people_child_is_ready_stays_unknown`. Under 20 ms.
Proof: world, `heavy cargo nextest run -p jig-core-world --test reads`,
once: a person whose sign-in ended is told so on their next read; under
0.5 s per test.
Don't: answer `SignedOut` for a party who is signed in but may not see
the subject.
Size: at most 30 lines of production code.
Removes: `Why::Unknown`'s standing for a signed-out party.

### 03.6 Signing out from the frame

Scope: a signed-in person's frame offers signing out; choosing it asks
the client's root to have its shell end the sign-in on the engine; the
client changes nothing until the shell answers `SignedOut`, then shows
the sign-in page as when signed out; a sign-out the shell reports lost
leaves the person signed in, the frame saying so and offering it again;
a service is never offered it (client.md, 12, "A sign-out that does not
reach the engine"; findings/domain.md, 10.5).
Needs: nothing.
Files:
- `crates/jig-domain-client/src/action.rs` (`Act`, :12),
  `crates/jig-domain-client/src/boundary.rs` (`Request`, :61),
  `crates/jig-domain-client/src/domain.rs` (dispatch :644-695; 1,238
  lines), `crates/jig-domain-client/src/signin.rs` (`choose`, :33).
- `crates/jig-view/src/frame.rs` (:12-48), `crates/jig-view/src/binding.rs`
  (:167-206), `crates/jig-view/src/phrases/jig.rs` (2,994 lines: the
  enum ~:132, number ~:1189, class ~:1920, list ~:2470),
  `crates/jig-view/src/phrases/english.rs` (1,495 lines, ~:752).
- The roots: `testing/jig-test-domain-client/src/boundary.rs:61`,
  `testing/jig-test-domain-client/src/route.rs:107-140`,
  `examples/ops/domain-client/src/boundary.rs:72`,
  `examples/ops/domain-client/src/route.rs:153-185`.
- Users of the roots' `Request` gaining an arm (12 sites in 9 files):
  `tests/client/src/test.rs:170,203`; `tests/client/src/scripted.rs:252,354`;
  `tests/client/src/joint/requests.rs:123`;
  `tests/client/src/joint/session.rs:425`;
  `tests/client/src/joint/front.rs:182`;
  `tests/ops/src/client/app.rs:196,249`;
  `tests/ops/src/client/transport.rs:350`;
  `tests/ops/src/joint/front.rs:268`;
  `examples/ops/view/src/tests.rs:347` (as built
  `examples/ops/client-view/src/tests.rs`).
- The client world's harness: `Duty` and `Front`
  (`tests/client/src/harness/mod.rs:58-76`), the `Front` impls
  (`tests/client/src/scripted.rs:260`, `tests/client/src/joint/front.rs:158`,
  `tests/client/src/harness/navigation_tests.rs:18`,
  `tests/client/tests/referee.rs:170`, `tests/ops/src/client/transport.rs:184`,
  `tests/ops/src/joint/front.rs:829`), duty handling
  (`tests/client/src/harness/world.rs:521-580`).
Shape:
- `jig_domain_client::action::Act::SignOut`, offered while a person is
  signed in and `signin.service` is false (`signin.rs:19`).
- `jig_domain_client::Request::SignOut`, doc "Ask the shell to end the
  sign-in on the engine. Its end is the shell's `Event::SignedOut`, after
  which the sign-in page shows."
- Each client root's `Request::SignOut`, routed to its shell as
  `SignIn` is.
- The frame shows a button named by `JigPhrase::SignOut` ("Sign out").
- `jig_domain_client::Event::SignOutLost`, the shell's, doc "The sign-out
  exchange ended lost (protocol/client.md, 4): the person is still signed
  in." It changes nothing but a notice of the frame's (client.md, 13),
  `JigPhrase::SignOutLost` ("Not signed out: the engine was not
  reached"); `Act::SignOut` stays offered, and asking again sends the
  sign-out again.
- The harness's `Duty::SignOut` and `Front::sign_out(&mut self, client:
  ClientId, now: Time, wall: Wall)`: the scripted engine ends the
  sign-in (`Change::SignedOut`,
  `testing/jig-scripted-engine/src/truth.rs:38,172`) before the shell
  says `SignedOut`; the joint world's front calls `Session::sign_out`
  (`tests/client/src/joint/session.rs:214`).
Test shape:
- `tests/client/tests/shell.rs` (45 lines), with `stories::chat_ready`
  and the fake person's `Find::named(Role::Button, ..)`
  (`testing/jig-fake-person/src/lib.rs:40`). Reference test:

  ```rust
  #[test]
  fn a_person_signs_out_from_the_frame_and_sees_the_sign_in_page() {
      let mut w = stories::chat_ready(5);
      let c = w.client(7, stories::inbox());
      w.person(
          c,
          vec![
              Step::See { find: Find::named(Role::Button, b"Sign out"), within: Duration::from_secs(1) },
              Step::Press { find: Find::named(Role::Button, b"Sign out") },
              Step::See { find: Find::named(Role::Button, b"Sign in with Test"), within: Duration::from_secs(1) },
          ],
      );
      let result = w.run(Until::Done);
      assert_eq!(result.verdict, skein_world::domain::Verdict::Passed);
  }
  ```

  with `use jig_fake_person::{Find, Step}; use jig_view::Role;
  use skein_lib::Duration;` added to the file. Under 0.5 s.
- Also there: `a_lost_sign_out_keeps_the_person_signed_in_and_says_so`:
  the shell answers the first sign-out `SignOutLost`; the person sees the
  notice and "Sign out", not the sign-in page; a second press, answered
  `SignedOut`, shows the sign-in page. Under 0.5 s.
- The joint world (`tests/client/src/joint/people_stories.rs`):
  `a_person_signs_out_and_a_later_request_is_answered_signed_out`.
  Under 0.5 s.
- `crates/jig-view/src/tests/`: `a_service_frame_offers_no_sign_out`.
  Under 20 ms.
Proof: world, `heavy cargo nextest run -p jig-client-world --test shell -E 'test(a_person_signs_out_from_the_frame_and_sees_the_sign_in_page)'`,
once: a person presses "Sign out" and sees the sign-in page; under
0.5 s. (No process layer exists before jig 33.)
Don't: let the client domain hold or clear a secret or a cookie; show
the sign-in page before the shell's `SignedOut`, or on `SignOutLost`;
add a browser or web shell path.
Size: at most 110 lines of production code (the lost sign-out, from the
designer's answer of 2026-10-12, adds about 30).
Removes: nothing.

### 03.s1 A turn that does not decode, in the client's contract

A step not in the plan (decisions.md, ruling 202), from the designer's
answer on jig 35.2 (jig `13153661`: `client/domain/client.md`, 10, "A
turn that does not decode reaches the domain as such, with why ...
Wherever a turn appears ... it is one of the two, read or not, so the
view handles both in one place"); its text is the designer's. Before jig
35.2 carries turns.
Shape: the contract (`crates/jig-domain-client/src/contract/`) gains
  `TurnRead { Decoded(Turn), Undecodable { why: TurnProblem } }` and
  `TurnProblem { Version { found }, Malformed, TooLarge }`;
  `Activity::Turn` and `ArchivedTurn` carry a `TurnRead` in place of a
  `Turn`. The joint world's translation and the domain's users follow,
  with the view's card for an undecodable turn, its words saying why.
Test shape: a view test of the card; a client world story in which one
  turn does not decode and the turns around it still show.
Proof: the gate.
Don't: drop an undecodable turn, or show another in its place.
Size: at most 120 lines of production code.

## Interface sketches

```rust
// crates/jig-domain-people/src/boundary.rs (jig_domain_people::boundary)
use skein_lib::Wall;

/// Child-to-client answer routed by the parent (people.md, 3 and 5.1).
pub enum Reply {
    /// Sign-in succeeded, or a live sign-in of the same person was replayed:
    /// the only reply naming a sign-in. Sent by the people child; its
    /// terminal is the client's sign-in exchange.
    SignedIn { person: u64, sign_in: u64, expires: Wall },
    /// Sign-out completed.
    SignedOut,
    // … Outcome, Again and Refused as built.
}
```

```rust
// crates/jig-domain-core/src/routing/vocabulary.rs (jig_domain_core)
use jig_domain_people as people;
use skein_lib::ReplyTo;

/// Core output that must follow the decision's commit (as built, but:).
pub enum Held {
    /// A party's keyed terminal, released after its decision is durable;
    /// a sign-in's number is inside `Reply::SignedIn`, nowhere else.
    PeopleReply { to: ReplyTo, reply: people::Reply },
    // … the other variants as built.
}

/// Core output that follows no mutation (as built, plus:).
pub enum Now {
    /// Load the ended chat `task`'s row for the core to judge forgetting
    /// it. The root answers `Event::ForgetChat { row }` if the row is
    /// there, and nothing otherwise; no state waits for it.
    LoadEndedChat { task: u64 },
    // … the other variants as built.
}
```

```rust
// crates/jig-domain-tasks/src/boundary.rs (jig_domain_tasks)
/// Terminal bounded admission reason (as built, plus:).
pub enum Refusal {
    /// No role of the project's policy may accept the proposal's kind;
    /// nothing of it is committed. Made by the core at a run's proposal;
    /// the run hears it in its call's answer.
    Unproposable,
    // … the other variants as built.
}
```

```rust
// crates/jig-domain-core/src/reads/vocabulary.rs (jig_domain_core::reads)
/// Refusal facts which reveal no invisible object identity.
pub enum Why {
    /// Missing, or invisible to the party.
    Unknown,
    /// A construction or load bound prevents admission.
    Busy,
    /// Required historical rows are no longer retained.
    Gone,
    /// The read's sign-in is unknown to a ready engine, ended or expired.
    SignedOut,
}
```

```rust
// crates/jig-domain-client/src/boundary.rs and action.rs (jig_domain_client)
/// One effect or hand-off routed by the application root (as built, plus:).
pub enum Request {
    /// Ask the shell to end the sign-in on the engine. Its terminal is the
    /// shell's `Event::SignedOut`, after which the sign-in page shows.
    SignOut,
    // … the other variants as built.
}

/// A person's act (as built, plus:).
pub enum Act {
    /// End this sign-in, from the frame; never offered to a service.
    SignOut,
    // … the other variants as built.
}
```

The people child owns what a reply says and its alarms; the core fires
them and erases what the child names in the same decision. The core owns
forgetting: the root only loads a row. The shell owns the sign-out's
exchange; the client domain owns only the act and what it shows.

## Questions to raise when reached

- **A chat that ended with no reply and was never marked** has no read
  position, so a results mark cannot find it until jig 12's index of each
  party's conversations (`protocol/store.md`, 7.3) exists; this plan's
  `later.md` lists "chats forgotten" under retention too. Do not add a
  read position for such a chat: forget what `marked_chats` names, and
  raise it.
- **A sign-out the shell cannot finish** (the engine out of reach):
  settled by the designer (jig `e8d9aa52`; client.md, 12, "A sign-out
  that does not reach the engine"). The person stays signed in; the frame
  says the sign-out did not reach the engine and offers it again, which
  is safe (`protocol/client.md`, 8). 03.6's `SignOutLost` builds it.
  Whether a native shell forgets its token on a shared machine is open
  (client.md, 18; `decisions.md`, "For the user").

## Done when

- No reply but a sign-in's names one: `git -C ~/src/rust/jig grep -n -E 'signing_in|PeopleReply \{[^}]*sign_in' -- crates testing tests examples`
  prints nothing.
- The conformance and `ops` worlds fire the people child's timer:
  `git -C ~/src/rust/jig grep -n 'Timer::People' -- tests/conformance/src/lib.rs tests/ops/src/conformance/mod.rs`
  prints two lines.
- No chat is forgotten by a test's own event: `git -C ~/src/rust/jig grep -n 'core::Event::ForgetChat' -- 'tests/*/tests'`
  prints nothing. The worlds' loaders answering `Now::LoadEndedChat` with
  it (`tests/core/src/effects.rs`, `tests/ops/src/joint/engine.rs`) and the
  people child's own worlds' `Event::ForgetChat` stay (rulings 118, 119).
- The tasks' refusal for a proposal nobody may accept exists and is
  mapped: `git -C ~/src/rust/jig grep -n 'Refusal::Unproposable' -- crates/jig-domain-core/src/routing`
  prints at least two lines.
- Reads say signed out: `git -C ~/src/rust/jig grep -n 'Why::SignedOut' -- crates/jig-domain-core/src tests/client/src/joint/read_packets.rs`
  prints at least three lines.
- The tests this session names pass in the gate: the last landed
  commit's `Gate-Result` trailer names `scripts/check.sh passed`
  (`git -C ~/src/rust/jig log -1 --format='%(trailers:key=Gate-Result,valueonly)' origin/main`).
