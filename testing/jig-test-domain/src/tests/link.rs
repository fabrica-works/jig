use super::fixtures;
use crate::{Delivery, Payload, Request, link};
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, JournalRoom, Queue, Token};

#[test]
fn a_worker_s_hello_reaches_the_fleet_with_its_bounds() {
    let core::Event::Fleet(fleet::Event::Hello { channel, hello }) = link::hello(fixtures::CHANNEL, fixtures::hello())
    else {
        panic!("a hello reaches the fleet");
    };
    assert_eq!(channel, fixtures::CHANNEL);
    assert_eq!(hello.slots, 2);
    assert_eq!(hello.stop_bound, Duration::from_secs(10));
    assert_eq!(hello.workstreams.as_ref(), [4, 5]);
    assert_eq!(
        hello.hosting.as_ref(),
        [fleet::Hosted { run: fixtures::RUN, attempt: fixtures::ATTEMPT, phase: fleet::Phase::Waiting }]
    );
    assert_eq!(
        hello.bounds,
        fleet::Bounds {
            models: Box::new([fleet::ModelBound { name: b"small".as_slice().into(), window: 1024, output: 128 }]),
            turns: 8,
            spend: 100,
            time: Duration::from_secs(60),
            waiting: Duration::from_secs(30),
            checks: Duration::from_secs(1),
            message_bytes: 128,
            inbox: 4,
            unacknowledged: 2,
        }
    );
}
#[test]
fn a_turn_from_a_link_reaches_the_fleet_with_its_refusals() {
    let mut domain = fixtures::domain();
    let core::Event::Fleet(fleet::Event::Turn { channel, run, attempt, turn, body }) =
        link::up(&mut domain, fixtures::CHANNEL, fixtures::turn())
    else {
        panic!("a turn reaches the fleet");
    };
    assert_eq!((channel, run, attempt, turn), (fixtures::CHANNEL, fixtures::RUN, fixtures::ATTEMPT, 1));
    let Some(Payload::Turn { cumulative, read, refused, opens_window, transcript, .. }) = domain.payloads.get(&body)
    else {
        panic!("turn bytes stay in the root");
    };
    assert_eq!((*cumulative, *read, *opens_window), (9, Some(7), true));
    assert_eq!(refused.as_ref(), [6]);
    assert_eq!(transcript.as_ref(), b"opaque turn");
}
#[test]
fn an_assignment_for_a_channel_leaves_as_the_hub_s_record() {
    let mut domain = fixtures::domain();
    let env = fixtures::env();
    domain.assignments.insert(fixtures::RUN.raw(), fixtures::assignment()).expect("one prepared run");
    let room = JournalRoom { writes: 0, held: 1 };
    let mut decision = domain.journal.decision(&room).expect("assignment room");
    let mut out = Queue::with_capacity(1);
    out.push(core::Request::Held(Box::new(core::Held::Assign {
        channel: fixtures::CHANNEL,
        run: fixtures::RUN,
        attempt: fixtures::ATTEMPT,
        activation: 3,
        assignment: fleet::Assignment { turns: Token::new(10), answered: Token::new(11) },
    })));
    crate::routes::route_core(&mut domain, &env, &mut decision, core::Requests::Out(out), &mut fixtures::work());
    domain.journal.accept(decision);
    let mut released = Queue::with_capacity(8);
    crate::release(&mut domain, &env, &mut released);
    let Request::Deliver(Delivery::Assigned { channel, assignment }) = released.pop().expect("assignment leaves")
    else {
        panic!("the typed assignment reaches the routing");
    };
    assert_eq!(channel, fixtures::CHANNEL);
    assert_eq!((assignment.task, assignment.attempt, assignment.conversation, assignment.charter), (2, 3, 1, 1));
    assert_eq!(assignment.run, fixtures::run());
    assert_eq!(assignment.transcript.as_ref(), [Box::<[u8]>::from(b"prior turn".as_slice())]);
    assert_eq!(assignment.grants[0].table, jig_domain_accounts::GrantTable::Accounts);
}
#[test]
fn a_lost_link_reaches_the_fleet_as_its_channel_lost() {
    let event = link::lost(fixtures::CHANNEL);
    let core::Event::Fleet(fleet::Event::Lost { channel }) = event else {
        panic!("a lost link reaches the fleet");
    };
    assert_eq!(channel, fixtures::CHANNEL);
}
#[test]
fn a_fenced_result_is_handed_out_before_the_answer_is_read() {
    let mut domain = fixtures::domain();
    let (run, attempt, payload, answer) = received_answer(&mut domain, fixtures::answer());
    assert_eq!(answer, fleet::Answer::Ended);
    let mut work = fixtures::work();
    link::answer(&mut domain, run, attempt, payload, &mut work);
    assert!(work.is_empty());
    let Request::ReadResult { payload: handed, outcome } = domain.now.pop().expect("fenced result handout") else {
        panic!("opaque result leaves for reading");
    };
    assert_eq!(handed, payload);
    assert_eq!(outcome.as_ref(), b"opaque result");
    let Some(Payload::Answer { cumulative, read, refused, .. }) = domain.payloads.get(&payload) else {
        panic!("the answer stays retained");
    };
    assert_eq!((*cumulative, *read), (12, Some(7)));
    assert_eq!(refused.as_ref(), [6]);
    let event = link::result_event(&domain, payload, tasks::End::Parked).expect("result metadata");
    let core::Event::AnswerPayload { cumulative, end, read, refused, .. } = event else {
        panic!("core result input");
    };
    assert_eq!((cumulative, end, read), (12, tasks::End::Parked, Some(7)));
    assert_eq!(refused.as_ref(), [6]);
}
#[test]
fn channel_records_leave_as_the_hub_s_vocabulary() {
    let mut domain = fixtures::domain();
    domain.links.insert(fixtures::CHANNEL).expect("one connected link");
    for (held, expected) in fixtures::channel_records() {
        let Delivery::Link { channel, down } = link::down(&mut domain, held) else {
            panic!("the hub receives the record");
        };
        assert_eq!(channel, fixtures::CHANNEL);
        assert_eq!(down, expected);
    }
}

fn received_answer(domain: &mut crate::Domain, up: jig_domain_host::LinkUp) -> (Token, Token, Token, fleet::Answer) {
    let core::Event::Fleet(fleet::Event::Answer { run, attempt, answer, payload, .. }) =
        link::up(domain, fixtures::CHANNEL, up)
    else {
        panic!("an answer reaches the fleet");
    };
    (run, attempt, payload, answer)
}
#[test]
fn a_start_refusal_reaches_the_core_as_invalid_without_reading_a_result() {
    let mut domain = fixtures::domain();
    let (run, attempt, payload, answer) = received_answer(&mut domain, fixtures::start_refusal());
    assert_eq!(answer, fleet::Answer::Invalid);
    let mut work = fixtures::work();
    link::answer(&mut domain, run, attempt, payload, &mut work);
    assert!(domain.now.is_empty());
    let Some(crate::Work::Core(core::Event::AnswerPayload { end, .. })) = work.pop() else {
        panic!("invalid start reaches the core without a result codec");
    };
    assert_eq!(end, tasks::End::Failed(tasks::Class::Invalid));
}

fn core_request(request: core::Request) -> core::Requests {
    let mut out = Queue::with_capacity(1);
    out.push(request);
    core::Requests::Out(out)
}
fn gather_brief(
    domain: &mut crate::Domain,
    env: &skein_lib::Env<crate::Limits>,
    decision: &mut skein_lib::Decision<crate::Write, Delivery>,
    work: &mut Queue<crate::Work>,
) {
    use jig_domain_brief::{Core, Planned};
    let mut sections = skein_lib::List::with_capacity(2);
    for (kind, text) in [(Core::Task, b"request".as_slice()), (Core::Results, b"previous".as_slice())] {
        sections
            .push(Planned::Core { kind, text: text.into(), limit: 32, priority: 0, required: true })
            .expect("sections");
    }
    crate::routes::route_core(
        domain,
        env,
        decision,
        core_request(core::Request::Now(Box::new(core::Now::BriefCorePlanned {
            task: fixtures::RUN.raw(),
            parent: None,
            sections,
        }))),
        work,
    );
    let crate::Work::Core(core::Event::Brief(event)) = work.pop().expect("brief planned") else {
        panic!("brief child");
    };
    let mut completed = Queue::with_capacity(jig_domain_brief::gather_max_out(&env.limits.core.brief));
    jig_domain_brief::gather_step(
        &mut domain.core.brief,
        &skein_lib::Env { now: env.now, wall: env.wall, limits: env.limits.core.brief },
        event,
        &mut completed,
    );
    let jig_domain_brief::GatherRequest::Complete { brief, order } = completed.pop().expect("completed brief") else {
        panic!("complete typed sections");
    };
    crate::routes::route_core(
        domain,
        env,
        decision,
        core_request(core::Request::Now(Box::new(core::Now::CompleteBrief { brief, order }))),
        work,
    );
}
fn prepare_brief_assignment(
    domain: &mut crate::Domain,
    env: &skein_lib::Env<crate::Limits>,
    decision: &mut skein_lib::Decision<crate::Write, Delivery>,
    work: &mut Queue<crate::Work>,
) {
    let fixture = fixtures::assignment();
    domain.workspace_writes.insert(fixture.task, Box::default()).expect("workspace");
    crate::routes::route_core(
        domain,
        env,
        decision,
        core_request(core::Request::Now(Box::new(core::Now::RunPrepared {
            task: fixture.task,
            attempt: fixture.attempt,
            conversation: fixture.conversation,
            charter: fixture.charter,
            run: fixture.run,
            messages: fixture.messages,
            transcript: fixture.transcript,
            saved: Box::default(),
            answered: Box::default(),
            grants: fixture.grants,
        }))),
        work,
    );
    crate::routes::route_core(
        domain,
        env,
        decision,
        core_request(core::Request::Held(Box::new(core::Held::Assign {
            channel: fixtures::CHANNEL,
            run: fixtures::RUN,
            attempt: fixtures::ATTEMPT,
            activation: 3,
            assignment: fleet::Assignment { turns: fixtures::RUN, answered: fixtures::RUN },
        }))),
        work,
    );
}
#[test]
fn an_assignment_carries_its_brief_s_sections_in_order() {
    let mut domain = fixtures::domain();
    let env = fixtures::env();
    let mut decision = domain.journal.decision(&JournalRoom { writes: 0, held: 1 }).expect("assignment room");
    let mut work = fixtures::work();
    gather_brief(&mut domain, &env, &mut decision, &mut work);
    prepare_brief_assignment(&mut domain, &env, &mut decision, &mut work);
    domain.journal.accept(decision);
    let mut released = Queue::with_capacity(8);
    crate::release(&mut domain, &env, &mut released);
    let Request::Deliver(Delivery::Assigned { assignment, .. }) = released.pop().expect("assignment") else {
        panic!("typed assignment");
    };
    assert_eq!(
        assignment.brief.as_ref(),
        [
            jig_domain_brief::GatherPlaced::Core {
                kind: jig_domain_brief::Core::Task,
                text: b"request".as_slice().into()
            },
            jig_domain_brief::GatherPlaced::Core {
                kind: jig_domain_brief::Core::Results,
                text: b"previous".as_slice().into()
            },
        ]
    );
}
