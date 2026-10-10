use super::spine::{LIMITS, delivered, drain, drive, env, frame, open_token, start};
use crate::{
    Domain, Event, Request,
    contract::*,
    intents::{Progress, send_card},
    objects::{self, Card, Left, Source},
};
use skein_lib::{List, Queue, Time, Token, Wall};
pub(super) fn text(bytes: &[u8]) -> Text {
    Text::new(Box::from(bytes), 32).unwrap()
}
pub(super) fn chip(task: u64) -> Chip {
    Chip {
        task,
        title: text(b"title"),
        executor: Executor::Party { party: 7 },
        phase: Phase::Held,
        details: Box::new(phase_details(Phase::Held)),
    }
}
pub(super) fn phase_details(phase: Phase) -> ChipPhase {
    match phase {
        Phase::Waiting => ChipPhase::Waiting { dependencies: List::with_capacity(0), hold: None },
        Phase::Idle => ChipPhase::Idle {
            wake: WakePolicy {
                words: true,
                results: true,
                topics: List::with_capacity(0),
                subscriptions: List::with_capacity(0),
            },
        },
        Phase::Due | Phase::Preparing | Phase::Claimed => ChipPhase::Starting,
        Phase::Running => ChipPhase::Running { attempt: 1, since: Wall::EPOCH, tool: None },
        Phase::BackingOff => ChipPhase::BackingOff {
            at: Wall::EPOCH,
            attempt: 1,
            tries: 3,
            class: FailureClass::Temporary,
            reason: text(b"temporary"),
        },
        Phase::Active => ChipPhase::Person { addressed: Recipient::Party(7), taken: None, since: Some(Wall::EPOCH) },
        Phase::Closing => ChipPhase::Closing(Closing {
            delegates: 0,
            connectors: Some(List::with_capacity(0)),
            stage: None,
            ending: None,
        }),
        Phase::Held => ChipPhase::Held {
            reason: HoldReason::Stopped { by: frame(7).party, at: Wall::EPOCH },
            since: Wall::EPOCH,
            revision: Revision { value: 3 },
            was: None,
        },
        Phase::Done => ChipPhase::Ended { how: Ending::Done, at: Wall::EPOCH },
        Phase::Failed => ChipPhase::Ended { how: Ending::Failed { reason: Some(text(b"failure")) }, at: Wall::EPOCH },
        Phase::Cancelled => ChipPhase::Ended {
            how: Ending::Cancelled { by: Actor::Party(7), reason: Some(text(b"cancelled")), at: None },
            at: Wall::EPOCH,
        },
    }
}
pub(super) fn authority() -> Authority {
    Authority {
        budget: 10,
        grants: List::with_capacity(0),
        task_kinds: List::with_capacity(0),
        depth: 1,
        tasks: 1,
        deadline: None,
        tools: List::with_capacity(0),
        note_scopes: List::with_capacity(0),
    }
}
pub(super) fn proposal() -> Proposal {
    Proposal {
        key: ProposalKey { proposer: Proposer::Task(8), number: 1 },
        project: 1,
        proposer: Proposer::Task(8),
        why: text(b"why"),
        action: ProposalAction::Widen { before: authority(), after: authority() },
        needs: authority(),
        funding: Budget { total: 10, own: 0, ended: 0, reserved: 0, left: 10, funder: Actor::Party(7) },
        waiting: Recipient::Party(7),
        trail: List::with_capacity(0),
        since: Wall::EPOCH,
        acts: List::with_capacity(0),
        ending: None,
    }
}
pub(super) fn question() -> Question {
    Question {
        task: chip(8),
        message: 1,
        words: text(b"whole question"),
        since: Wall::EPOCH,
        acts: List::with_capacity(0),
        ending: None,
    }
}
pub(super) fn escalation() -> Escalation {
    Escalation {
        task: chip(8),
        hold: Hold {
            revision: Revision { value: 3 },
            reason: HoldReason::Stopped { by: frame(7).party, at: Wall::EPOCH },
            before: Phase::Running,
            since: Wall::EPOCH,
            result: None,
            spend: List::with_capacity(0),
        },
        steering: Box::new(Steering {
            person_revision: None,
            taken_by: None,
            addressed: None,
            revision: Revision { value: 1 },
            spec: specification(text(b"held work")),
            instructions: text(b""),
            wake: WakePolicy {
                words: true,
                results: true,
                topics: List::with_capacity(0),
                subscriptions: List::with_capacity(0),
            },
            dependencies: List::with_capacity(0),
            authority: authority(),
            budget: Budget { total: 100, own: 100, ended: 0, reserved: 0, left: 0, funder: Actor::Party(1) },
        }),
        impact: Impact {
            tasks: PhaseCounts { waiting: 0, running: 0, held: 1, closing: 0, done: 0, failed: 0, cancelled: 0 },
            live_runs: 0,
            proposals: 0,
            effects_pending: 0,
            effects_settling: 0,
            takeover_budget: 0,
        },
        waiting: Recipient::Party(7),
        trail: List::with_capacity(0),
        acts: List::with_capacity(0),
        ending: None,
    }
}
pub(super) fn person() -> PersonTask {
    PersonTask {
        task: chip(8),
        revision: Revision { value: 4 },
        requester: Actor::Task(1),
        spec: specification(text(b"do it")),
        contract: PersonContract::Words,
        inputs: List::with_capacity(0),
        addressed: Recipient::Role { project: 1, role: 2 },
        taken: None,
        since: None,
        stall: None,
        deadline: None,
        acts: List::with_capacity(0),
        ending: None,
    }
}
pub(super) fn result() -> Result {
    Result {
        task: chip(8),
        ending: Ending::Done,
        value: Some(ResultValue::Report(text(b"report"))),
        at: Wall::EPOCH,
        acts: List::with_capacity(0),
    }
}
fn inbox(keys: &[ObjectKey]) -> Delivery {
    let mut entries = List::with_capacity(2);
    for key in keys {
        entries
            .push(InboxEntry {
                reply: None,
                lineage: List::with_capacity(0),
                goal: None,
                recipient: None,
                stall: None,
                object: *key,
                project: 1,
                group: InboxGroup::Proposals,
                position: 1,
                at: Wall::EPOCH,
            })
            .unwrap();
    }
    Delivery::Inbox(Box::new(Inbox {
        entries,
        needs: Needs { proposals: 0, questions: 0, escalations: 0, person_tasks: 0 },
        updates: Count { value: 0, limited: false },
        results_seen: 0,
        replies_seen: List::with_capacity(0),
        more: false,
    }))
}
pub(super) fn live() -> (Domain, Token) {
    let mut domain = Domain::new(&LIMITS, 10);
    let f = start(&mut domain, List::with_capacity(0));
    let out = delivered(&mut domain, Time::ZERO, f, frame(7));
    let page = open_token(&out);
    super::pages::finish_initial_inbox_reads(&mut domain, &out);
    (domain, page)
}
fn watched(domain: &mut Domain, page: Token, keys: &[ObjectKey]) -> Box<[Request]> {
    drive(domain, Time::ZERO, Event::Watch { watch: page, event: WatchEvent::Delivered(inbox(keys)) })
}
fn read_token(rows: &[Request]) -> Token {
    for row in rows {
        if let Request::Read { read, .. } = row {
            return *read;
        }
    }
    unreachable!("an object read was emitted")
}

#[test]
fn documents_keep_whole_object_facts_revisions_and_reply_positions_within_the_working_set() {
    let (mut domain, page) = live();
    let cases = [
        (ObjectKey::Proposal(proposal().key), ReadValue::Proposal(Box::new(proposal()))),
        (ObjectKey::Escalation(8), ReadValue::Escalation(Box::new(escalation()))),
        (ObjectKey::Question { task: 8, message: 1 }, ReadValue::Question(Box::new(question()))),
        (ObjectKey::PersonTask(8), ReadValue::PersonTask(Box::new(person()))),
        (ObjectKey::Result(8), ReadValue::Result(Box::new(result()))),
    ];
    for (key, value) in cases {
        domain.objects.clear();
        domain.objects.reclaim();
        domain.reads.reclaim();
        let requests = watched(&mut domain, page, &[key]);
        drive(
            &mut domain,
            Time::ZERO,
            Event::Read { read: read_token(&requests), result: ReadResult::Value(value.clone()) },
        );
        let id = domain.objects.find(key).unwrap();
        let object = domain.objects.get(id).unwrap();
        match value {
            ReadValue::Proposal(document) => assert_eq!(objects::proposal::facts(object), Some(document.as_ref())),
            ReadValue::Escalation(document) => {
                assert_eq!(objects::escalation::facts(object), Some(document.as_ref()));
                assert_eq!(objects::escalation::revision(object), Some(Revision { value: 3 }));
            }
            ReadValue::Question(document) => assert_eq!(objects::question::facts(object), Some(document.as_ref())),
            ReadValue::PersonTask(document) => {
                assert_eq!(objects::person_task::facts(object), Some(document.as_ref()));
                assert_eq!(objects::person_task::revision(object), Some(Revision { value: 4 }));
            }
            ReadValue::Result(document) => assert_eq!(objects::result::facts(object), Some(document.as_ref())),
            ReadValue::Frame(_)
            | ReadValue::Task(_)
            | ReadValue::Ended(_)
            | ReadValue::History(_)
            | ReadValue::Activity(_)
            | ReadValue::Inbox(_)
            | ReadValue::Conversations(_)
            | ReadValue::Spend(_)
            | ReadValue::Goals(_)
            | ReadValue::Project(_)
            | ReadValue::Notes(_)
            | ReadValue::Deployment(_) => unreachable!(),
        }
    }
    let line = ConversationLine {
        runtime: Runtime::Idle,
        delegates: 0,
        budget: Budget { total: 0, own: 0, ended: 0, reserved: 0, left: 0, funder: Actor::Party(7) },
        chip: chip(9),
        project: 1,
        first_words: text(b"first words"),
        unread: Count { value: 2, limited: false },
        seen: 5,
        last_reply: 7,
        changed: Wall::EPOCH,
    };
    let id = domain.objects.reply(Source::Window { generation: 0, number: 1 }, line).unwrap();
    let reply = domain.objects.get(id).unwrap().reply.as_ref().unwrap();
    assert_eq!(reply.first_words.bytes(), b"first words");
    assert_eq!(reply.first_unread, 6);
    assert!(domain.objects.name(ObjectKey::Task(10), Source::Window { generation: 0, number: 1 }).is_none());
}

#[test]
fn a_deciding_card_survives_a_drop_until_its_durable_answer_then_lingers() {
    let (mut domain, page) = live();
    let key = ObjectKey::Proposal(proposal().key);
    let reads = watched(&mut domain, page, &[key]);
    drive(
        &mut domain,
        Time::ZERO,
        Event::Read { read: read_token(&reads), result: ReadResult::Value(ReadValue::Proposal(Box::new(proposal()))) },
    );
    let mut out = Queue::with_capacity(crate::max_out(&LIMITS));
    let request = send_card(
        &mut domain,
        key,
        ActKind::Accept,
        Box::new(Ask::DecideProposal { project: 1, proposal: proposal().key, decision: Decision::Accepted }),
        &mut out,
    )
    .unwrap();
    assert_eq!(drain(out).len(), 1);
    let id = domain.objects.find(key).unwrap();
    assert!(watched(&mut domain, page, &[]).is_empty());
    assert_eq!(
        domain.objects.get(id).unwrap().card,
        Card::Deciding { request: Some(request), progress: Progress::InFlight }
    );
    assert!(domain.notices().ids().is_empty());
    let outcome = Outcome::DecidedFirst { by: frame(9).party, how: Decision::Accepted, at: Wall::EPOCH };
    drive(&mut domain, Time::from_nanos(20), Event::Answered { request, answer: Answer::Done(outcome.clone()) });
    assert_eq!(
        domain.objects.get(id).unwrap().card,
        Card::Leaving { how: Left::Answered { outcome }, until: Time::from_nanos(5_000_000_020) }
    );
    assert!(domain.notices().ids().is_empty(), "a card still on this page owns its outcome");
    // Fire the unrelated heartbeat before the linger.
    let mut out = Queue::with_capacity(crate::max_out(&LIMITS));
    for _ in 0_u8..4 {
        crate::fire(&mut domain, &env(Time::from_nanos(5_000_000_020)), &mut out);
    }
    assert!(domain.objects.find(key).is_none());
}

#[test]
fn a_drop_reads_how_the_card_ended_and_a_preexisting_read_can_supply_the_terminal() {
    for read_before_drop in [true, false] {
        let (mut domain, page) = live();
        let key = ObjectKey::Question { task: 8, message: 1 };
        let first = watched(&mut domain, page, &[key]);
        let initial = read_token(&first);
        if !read_before_drop {
            drive(
                &mut domain,
                Time::ZERO,
                Event::Read { read: initial, result: ReadResult::Value(ReadValue::Question(Box::new(question()))) },
            );
            domain.reads.reclaim();
        }
        let dropped = watched(&mut domain, page, &[]);
        let terminal = if read_before_drop {
            assert!(dropped.is_empty());
            initial
        } else {
            read_token(&dropped)
        };
        let id = domain.objects.find(key).unwrap();
        let Card::Leaving { how, .. } = &domain.objects.get(id).unwrap().card else {
            panic!("terminal read keeps card leaving");
        };
        assert_eq!(*how, Left::Looking);
        let ending = Ending::Decided {
            by: frame(9).party.number,
            party: Some(frame(9).party),
            how: Some(Decision::Accepted),
            at: Wall::EPOCH,
        };
        let mut ended = question();
        ended.ending = Some(ending.clone());
        drive(
            &mut domain,
            Time::from_nanos(30),
            Event::Read { read: terminal, result: ReadResult::Value(ReadValue::Question(Box::new(ended))) },
        );
        assert_eq!(
            domain.objects.get(id).unwrap().card,
            Card::Leaving { how: Left::Ended { how: ending }, until: Time::from_nanos(5_000_000_030) }
        );
        assert_ne!(domain.page(), &crate::pages::Page::Missing);
        assert_eq!(domain.objects.get(id).unwrap().sources.len(), 0);
    }
}

fn specification(words: Text) -> SpecificationDocument {
    SpecificationDocument::complete(Specification {
        words,
        parameters: List::with_capacity(0),
        inputs: List::with_capacity(0),
    })
}
