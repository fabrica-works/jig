use super::fixtures;
use crate::{End, Stop, StoreAnswer, StoreRequest};
use jig_domain_core as core;

#[test]
fn the_restart_asks_one_load_at_a_time() {
    let mut service = fixtures::started(&fixtures::engine_document());
    fixtures::turn(&mut service);
    fixtures::turn(&mut service);
    let engine = fixtures::engine(&mut service);
    assert_eq!(engine.store_requests().len(), 1);
    let Some(StoreRequest::Restart(step)) = engine.store_requests().pop() else {
        panic!("restart load requested");
    };
    assert_eq!(step, core::RestartStep::LoadCore);
    engine.store_answers().push(StoreAnswer::Restored { step: core::RestartStep::LoadCore, rows: Box::new([]) });
    fixtures::turn(&mut service);
    let engine = fixtures::engine(&mut service);
    let Some(StoreRequest::Commit { .. }) = engine.store_requests().pop() else {
        panic!("restart writes commit first");
    };
    assert_eq!(engine.store_requests().len(), 0);
}

fn period(driven: &mut fixtures::Driven, number: u64) {
    driven.party(core::Event::Period { project: 1, period: number, budget: 100, next: None });
}

#[test]
fn a_store_answer_is_taken_with_the_journal_full() {
    let mut driven = fixtures::Driven::started(&fixtures::engine_document());
    for number in 2..=4 {
        period(&mut driven, number);
    }
    let first = driven.commit_asked();
    driven.commit(first);
    assert_eq!(fixtures::engine(&mut driven.service).store_answers().len(), 0);
    period(&mut driven, 5);
    assert_eq!(driven.commits.len(), 3);
}

#[test]
fn a_party_request_is_returned_busy_with_the_journal_full() {
    let mut driven = fixtures::Driven::started(&fixtures::engine_document());
    for number in 2..=4 {
        period(&mut driven, number);
    }
    driven.chat(101, b"retained");
    let event = driven.busy.pop().expect("unadmitted party input");
    let jig_test_domain::Event::Core(core::Event::People(jig_domain_people::Event::Ask { ask, .. })) = *event else {
        panic!("busy returns the same party request");
    };
    let jig_domain_people::Ask::StartChat { words, .. } = ask else {
        panic!("same chat request");
    };
    assert_eq!(words.as_ref(), b"retained");
}

#[test]
fn a_commit_leaves_in_the_iteration_that_made_it() {
    let mut driven = fixtures::Driven::started(&fixtures::engine_document());
    period(&mut driven, 2);
    assert_eq!(driven.commits.len(), 1);
}

#[test]
fn a_failed_commit_releases_nothing_after_it() {
    let mut driven = fixtures::Driven::started(&fixtures::engine_document());
    driven.chat(101, b"one");
    let first = driven.commit_asked();
    driven.chat(102, b"two");
    let second = driven.commit_asked();
    driven.fail(first);
    driven.commit(second);
    assert_eq!(driven.released_to_parties(), 0);
    assert_eq!(fixtures::engine(&mut driven.service).store_answers().len(), 0);
    assert_eq!(driven.end(), Some(&End::Stopped(Stop::CommitFailed { number: first })));
}

#[test]
fn the_service_waits_while_its_load_is_outstanding() {
    let mut service = fixtures::started(&fixtures::engine_document());
    fixtures::turn(&mut service);
    assert!(!service.work_pending(skein_lib::Time::from_nanos(1)), "waiting for the store is not work");
}

#[test]
fn a_store_answer_is_work_when_party_outputs_are_full() {
    let mut driven = fixtures::Driven::started(&fixtures::engine_document());
    period(&mut driven, 2);
    let number = driven.commit_asked();
    let engine = fixtures::engine(&mut driven.service);
    for _ in 0..engine.parties_out().room() {
        engine.parties_out().push(crate::PartyOutput::Busy(Box::new(jig_test_domain::Event::Core(
            core::Event::Period { project: 1, period: 3, budget: 100, next: None },
        ))));
    }
    engine.store_answers().push(StoreAnswer::Committed { number, fill: 0 });
    assert!(driven.service.work_pending(skein_lib::Time::from_nanos(1)));
    fixtures::turn(&mut driven.service);
    assert_eq!(fixtures::engine(&mut driven.service).store_answers().len(), 0);
}

fn path() -> jig_test_connector::Path {
    let mut segments = skein_lib::List::with_capacity(1);
    segments.push(b"one".as_slice().into()).expect("one segment");
    jig_test_connector::Path::new(segments, 8).expect("fixture path")
}
fn effect(target: u64) -> jig_test_connector::Effect {
    jig_test_connector::Effect {
        kind: 1,
        resources: Box::new([path()]),
        purpose: 1,
        condition: None,
        target,
        state: target,
    }
}
fn key() -> jig_test_connector::Key {
    jig_test_connector::Key { deployment: [1; 16], task: 1, purpose: 1, attempt: 1, completion: 1, position: 1 }
}
#[test]
fn the_system_applies_an_effect_and_reads_it_back() {
    use jig_test_connector::{ApplyResult, Fact, Form, Origin, Recovery, SystemEvent, SystemRequest};
    let mut system = crate::system::System::new(2, 2);
    assert_eq!(
        system.answer(SystemRequest::Apply {
            entry: 1,
            attempt: 1,
            key: key(),
            effect: effect(7),
            form: Form::Set,
            recovery: Recovery::Keyed
        }),
        SystemEvent::Applied { entry: 1, attempt: 1, result: ApplyResult::Made { state: 7 } }
    );
    assert_eq!(
        system.answer(SystemRequest::ReadFact { resource: path(), observed: skein_lib::Wall::EPOCH }),
        SystemEvent::Fact {
            resource: path(),
            fact: Fact { state: Some(7), observed: skein_lib::Wall::EPOCH, pending: false },
            origin: Origin::Own
        }
    );
}

#[test]
fn a_keyed_retry_keeps_the_first_result() {
    use jig_test_connector::{ApplyResult, Form, Recovery, SystemEvent, SystemRequest};
    let mut system = crate::system::System::new(2, 2);
    let _first = system.answer(SystemRequest::Apply {
        entry: 1,
        attempt: 1,
        key: key(),
        effect: effect(7),
        form: Form::Set,
        recovery: Recovery::Keyed,
    });
    assert_eq!(
        system.answer(SystemRequest::Apply {
            entry: 1,
            attempt: 2,
            key: key(),
            effect: effect(9),
            form: Form::Set,
            recovery: Recovery::Keyed
        }),
        SystemEvent::Applied { entry: 1, attempt: 2, result: ApplyResult::Made { state: 7 } }
    );
}

#[test]
fn a_handed_in_account_is_lent_and_its_value_stays_out_of_the_root() {
    use jig_domain_accounts::{FromComponent, Request, ToComponent};
    let config = fixtures::config();
    let mut accounts = crate::accounts::Accounts::new(&config, &fixtures::files(&[]), &fixtures::seeds(), None, 64)
        .expect("handed-in account");
    let mut values = fixtures::values();
    let mut events = skein_lib::Queue::with_capacity(8);
    let wall = skein_lib::Wall::from_nanos(1_000_000_000);
    accounts.iterate(skein_lib::Time::ZERO, wall, &mut values, &mut events);
    accounts.request(Request::Component(ToComponent::Want { account: 1 }));
    accounts.iterate(skein_lib::Time::ZERO, wall, &mut values, &mut events);
    let event = events.pop().expect("root account event");
    let jig_test_domain::Event::Core(core::Event::Account(jig_domain_accounts::Event::Component(
        FromComponent::Granted { account, generation, valid },
    ))) = event
    else {
        panic!("typed account event contains metadata only");
    };
    assert_eq!((account, generation, valid), (1, 1, skein_lib::Duration::from_secs(119)));
    let value = values.accounts.value(1, 1, skein_lib::Time::ZERO).expect("lent value");
    assert_eq!(value.token.expose(), fixtures::TOKEN);
    assert_eq!(value.account_id.expose(), b"acc");
}

#[test]
fn a_call_is_decoded_and_one_that_does_not_is_answered_an_error() {
    let config = fixtures::config();
    let mut tools = tools(&config);
    let rows: [(&[u8], Option<&[u8]>); 3] = [
        (br#"{"target":3,"reason":"no longer needed"}"#, None),
        (br#"{"target":3}"#, Some(b"cancel: the field reason is missing")),
        (b"broken", Some(b"cancel: the input is malformed")),
    ];
    for (input, error) in rows {
        let event = decode_call(&mut tools, input);
        match error {
            None => {
                let jig_test_domain::Event::Core(core::Event::NamedAction { key, action, .. }) = event else {
                    panic!("named action");
                };
                assert_eq!(key, core::CallKey { task: 2, attempt: 3, completion: 7, position: 2 });
                let core::NamedAction::Control { target, control: jig_domain_tasks::Control::Cancel { .. } } = action
                else {
                    panic!("cancel action");
                };
                assert_eq!(target, 3);
            }
            Some(expected) => {
                let jig_test_domain::Event::CallAnswered { call, .. } = event else {
                    panic!("call error");
                };
                assert_eq!(call.answer, core::SettledAnswer::Host { error: true, body: expected.into() });
            }
        }
    }
}

fn encoded_assignment(
    workers: &crate::workers::Workers,
    config: &jig_protocol_config::Engine,
    values: &crate::values::Values,
    attempt: u64,
) -> (jig_domain_host::Assignment, Box<[Option<jig_protocol_engine_workers::GrantValue>]>) {
    let mut assignment = fixtures::assignment(config);
    assignment.attempt = attempt;
    let event = workers
        .encoded_assignment(skein_lib::Time::ZERO, skein_lib::Token::new(17), assignment, values)
        .expect("assignment encoded");
    let jig_protocol_engine_workers::Event::Down {
        channel,
        down: jig_domain_host::LinkDown::Assign { assignment },
        values,
        ..
    } = event
    else {
        panic!("worker assignment");
    };
    assert_eq!(channel, skein_lib::Token::new(17));
    (assignment, values)
}
#[test]
fn an_assignment_leaves_with_its_charter_encoded_and_grants_filled() {
    let config = fixtures::config();
    let workers = fixtures::workers(&config);
    let values = fixtures::values();
    let (assignment, grants) = encoded_assignment(&workers, &config, &values, 3);
    let decoded = smith_protocol_channel::decode_charter(
        &assignment.assignment.charter,
        &jig_protocol_charter::limits(),
        &fixtures::endpoints(),
    )
    .expect("charter decodes");
    assert_eq!(decoded.instructions.as_ref(), b"fixture contents");
    assert_eq!(decoded.brief.sections.len(), 2);
    assert_eq!(decoded.brief.sections[0].title.as_ref(), b"Task");
    assert_eq!(decoded.brief.sections[0].text.as_ref(), b"Make a report");
    assert_eq!(decoded.brief.sections[1].title.as_ref(), b"Results");
    assert_eq!(decoded.brief.sections[1].text.as_ref(), b"Prior result");
    assert_eq!(grants.len(), 2);
    let account = grants[0].as_ref().expect("account grant value");
    assert_eq!(account.value.expose(), fixtures::TOKEN);
    assert_eq!(account.account_id.expose(), b"acc");
    let application = grants[1].as_ref().expect("application grant value");
    assert_eq!(application.value.expose(), b"application-token");
    assert!(application.account_id.expose().is_empty());
}
#[test]
fn a_grant_whose_generation_the_table_lacks_is_left_out() {
    use jig_domain_host::{Grant, GrantTable};
    use skein_lib::{Duration, Time};
    let values = fixtures::values();
    for (table, generation, time) in [
        (GrantTable::Accounts, 2, Time::ZERO),
        (GrantTable::Accounts, 1, Time::from_nanos(120_000_000_000)),
        (GrantTable::Application, 2, Time::ZERO),
    ] {
        let grants = values.grants(&[Grant { table, account: 1, generation, valid: Duration::from_secs(1) }], time);
        assert_eq!(grants.len(), 1);
        assert!(grants[0].is_none());
    }
}
#[test]
fn the_run_s_identity_is_shared_by_a_conversation_s_attempts() {
    let config = fixtures::config();
    let workers = fixtures::workers(&config);
    let values = fixtures::values();
    for attempt in [3, 4, 9] {
        let (assignment, _) = encoded_assignment(&workers, &config, &values, attempt);
        assert_eq!(assignment.assignment.attempt, skein_lib::Token::new(attempt));
        assert_eq!(assignment.assignment.identity, IDENTITY);
    }
}
#[test]
fn the_workers_listener_opens_before_the_restart() {
    let mut service = fixtures::started(&fixtures::workers_document());
    fixtures::turn(&mut service);
    assert!(fixtures::engine(&mut service).store_requests().is_empty());
    assert!(!fixtures::engine(&mut service).workers.as_ref().expect("workers").opened());
    fixtures::turn(&mut service);
    assert!(fixtures::engine(&mut service).workers.as_ref().expect("workers").opened());
    let Some(StoreRequest::Restart(core::RestartStep::LoadCore)) =
        fixtures::engine(&mut service).store_requests().pop()
    else {
        panic!("restart follows listener");
    };
}
#[test]
fn a_result_that_does_not_decode_fails_its_attempt_as_invalid() {
    for bytes in [b"".as_slice(), b"broken", &[0, 2]] {
        let event = crate::tools::result(skein_lib::Token::new(7), bytes);
        let jig_test_domain::Event::ResultRead { payload, end } = event else {
            panic!("result read");
        };
        assert_eq!(payload, skein_lib::Token::new(7));
        assert_eq!(end, jig_domain_tasks::End::Failed(jig_domain_tasks::Class::Invalid));
    }
}
#[test]
fn a_signed_in_source_without_runtime_is_refused_at_its_path() {
    let document = fixtures::with(
        &fixtures::accounts_document(),
        br#"{"handed_in": {"file": "accounts/chat.json"}}"#,
        fixtures::SIGNED_SOURCE,
    );
    let found = crate::startup(&document, &fixtures::files(&document), None, &fixtures::seeds(), None, None)
        .err()
        .expect("runtime absent");
    assert_eq!(
        found.problems.get(0),
        Some(&jig_protocol_config::Problem {
            path: jig_protocol_config::Path::root()
                .member(b"engine")
                .member(b"llm")
                .member(b"accounts")
                .child(jig_protocol_config::Segment::Index(0))
                .member(b"source"),
            what: jig_protocol_config::What::Refers,
        })
    );
}

const IDENTITY: [u8; 16] = [227, 220, 92, 103, 122, 9, 131, 161, 162, 47, 205, 195, 6, 132, 110, 28];

#[test]
fn kernel_operations_are_routed_back_to_their_owner() {
    use skein_io::kernel::{Complete, Done, Error, Fd, Op, Submit};
    use skein_lib::{Queue, Token};
    for (owner, external) in [(false, 1), (true, 2)] {
        let mut kernel = crate::accounts::Kernel::new(4);
        let mut sockets = Queue::with_capacity(2);
        let mut files = Queue::with_capacity(2);
        sockets.push(Submit { op: Token::new(1), kind: Op::Close { fd: Fd::new(6) } });
        files.push(Submit { op: Token::new(1), kind: Op::Close { fd: Fd::new(7) } });
        let mut output = Queue::with_capacity(4);
        kernel.publish(false, &mut sockets, &mut output);
        kernel.publish(true, &mut files, &mut output);
        let socket = output.pop().expect("socket operation");
        let file = output.pop().expect("file operation");
        let request = if owner { file } else { socket };
        let pending = if owner { &mut files } else { &mut sockets };
        pending.push(Submit { op: Token::new(2), kind: Op::Cancel { target: Token::new(1) } });
        let (routed, complete) = kernel.complete(
            Complete { op: request.op, kind: request.kind, result: Ok(Done::Nothing) },
            &sockets,
            Some(&files),
        );
        assert_eq!(routed, owner);
        assert_eq!(complete.op, Token::new(1));
        let pending = if owner { &mut files } else { &mut sockets };
        kernel.publish(owner, pending, &mut output);
        let cancel = output.pop().expect("cancel after target completion");
        assert_eq!(cancel.kind, Op::Cancel { target: Token::new(external) });
        let (routed, complete) = kernel.complete(
            Complete { op: cancel.op, kind: cancel.kind, result: Err(Error::TooLate) },
            &sockets,
            Some(&files),
        );
        assert_eq!(routed, owner);
        assert_eq!(complete.op, Token::new(2));
        assert_eq!(complete.kind, Op::Cancel { target: Token::new(1) });
    }
}

fn tools(config: &jig_protocol_config::Engine) -> crate::tools::Tools {
    crate::tools::Tools::new(&config.limits.core)
}
fn decode_call(tools: &mut crate::tools::Tools, input: &[u8]) -> jig_test_domain::Event {
    tools.decode(
        skein_lib::ReplyTo::new(skein_lib::Token::new(11)),
        skein_lib::Token::new(2),
        skein_lib::Token::new(3),
        &fixtures::call(input),
    )
}
#[test]
fn a_call_s_terminal_is_rendered_back_with_its_name() {
    let config = fixtures::config();
    let mut tools = tools(&config);
    let key = core::CallKey { task: 2, attempt: 3, completion: 7, position: 2 };
    for durable in [false, true] {
        let _decoded = decode_call(&mut tools, br#"{"target":3,"reason":"no longer needed"}"#);
        let event = tools
            .render(skein_lib::ReplyTo::new(skein_lib::Token::new(11)), key, &core::CallPart::Controlled, durable)
            .expect("rendered terminal");
        let (call, seen) = if let jig_test_domain::Event::CallAnswered { to, call } = event {
            assert_eq!(to.into_token(), skein_lib::Token::new(11));
            (call, false)
        } else {
            let jig_test_domain::Event::Core(core::Event::SettledCall { to, key: named, call }) = event else {
                panic!("rendered call route");
            };
            assert_eq!(to.into_token(), skein_lib::Token::new(11));
            assert_eq!(named, key);
            (call, true)
        };
        assert_eq!(seen, durable);
        assert_eq!(call.name, fixtures::call(b"").name);
        assert_eq!(call.tool.as_ref(), b"cancel");
        assert_eq!(call.answer, core::SettledAnswer::Host { error: false, body: b"Done".as_slice().into() });
    }
}
fn account_runtime() -> crate::AccountsRuntime {
    let mut issuers = skein_lib::List::with_capacity(1);
    issuers
        .push((
            1,
            skein_oauth_accounts::Endpoint {
                address: ::core::net::SocketAddr::from(([127, 0, 0, 1], 7101)),
                transport: skein_oauth_accounts::Transport::Plaintext,
            },
        ))
        .expect("issuer");
    crate::AccountsRuntime {
        root: skein_io::kernel::Fd::new(7),
        effective_user: 1000,
        issuers,
        trust_roots: 0,
        trust_root_bytes: 0,
    }
}
#[test]
fn a_signed_in_source_uses_its_runtime_private_keeper() {
    let config = fixtures::signed_config();
    let mut accounts =
        crate::accounts::Accounts::new(&config, &fixtures::files(&[]), &fixtures::seeds(), Some(account_runtime()), 64)
            .expect("signed-in runtime");
    let mut values = fixtures::values();
    let mut events = skein_lib::Queue::with_capacity(8);
    accounts.iterate(skein_lib::Time::ZERO, skein_lib::Wall::EPOCH, &mut values, &mut events);
    let open = accounts.submitted.pop().expect("private root opens");
    let skein_io::kernel::Op::Open { root, .. } = open.kind else {
        panic!("private root opens");
    };
    assert_eq!(root, skein_io::kernel::Fd::new(7), "the keeper uses the shell's root");
}
