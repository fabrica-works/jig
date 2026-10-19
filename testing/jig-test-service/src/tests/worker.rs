use super::fixtures;
use crate::{Service, Worker, iterate};
use jig_test_worker as root;
use skein_io as io;
use skein_lib::{Time, Token, Wall};

fn worker(service: &mut Service) -> &mut Worker {
    service.workers.get_mut(0).expect("worker layer")
}
fn assign(service: &mut Service) {
    worker(service).step(root::Event::Link(fixtures::worker_assignment()));
}
fn spawn(service: &mut Service) -> (Token, io::kernel::Spawn) {
    let layer = worker(service);
    for _ in 0..layer.io.len() {
        if let Some(io::Request::Spawn { owner, spawn }) = layer.io.pop() {
            return (owner, spawn);
        }
    }
    panic!("decoded start asks for a process")
}

#[test]
fn io_events_reach_only_the_layer_whose_tokens_they_carry() {
    let mut service = fixtures::two_workers();
    let owner = {
        let layer = worker(&mut service);
        layer.iterate(Time::ZERO, Wall::EPOCH);
        let io::Request::Connect { owner, .. } = layer.io.pop().expect("dial") else {
            panic!("dial");
        };
        owner
    };
    let other = service.workers.get_mut(1).expect("second worker");
    other.iterate(Time::ZERO, Wall::EPOCH);
    let io::Request::Connect { owner: other_owner, .. } = other.io.pop().expect("second dial") else {
        panic!("second dial");
    };
    assert_ne!(owner, other_owner);
    service.told.push(io::Event::Failed { owner, error: io::Error::Busy });
    iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert!(service.end().is_none());
    assert!(worker(&mut service).next_deadline().expect("only the dial's layer retries") > Time::ZERO);
    assert_eq!(service.workers.get(1).expect("second worker").next_deadline(), None);
}
#[test]
fn a_decoded_charter_comes_back_in_the_iterate_that_asked() {
    let mut service = fixtures::worker_service();
    assign(&mut service);
    let (_, launched) = spawn(&mut service);
    assert_eq!(launched.program.as_ref(), b"agent-program");
    assert_eq!(launched.root, io::kernel::Fd::new(9));
    assert_eq!(launched.dir.as_ref(), b".");
}
#[test]
fn a_turn_is_encoded_against_the_worker_s_endpoints() {
    let mut agent = fixtures::agent();
    let turn = fixtures::concrete_turn();
    let root::Encoded::Turn(encoded) =
        crate::worker_route::encode(root::Encodable::Turn(turn), &agent.limits, &agent.endpoints)
    else {
        panic!("turn encoded");
    };
    assert_eq!(encoded.turn, 1);
    assert_eq!(encoded.spent, 2);
    assert!(skein_lib::bytes::find(&encoded.body, b"fake").is_some());
    let mut entries = skein_lib::List::with_capacity(1);
    entries
        .push(smith_protocol_channel::Endpoint { name: b"different".as_slice().into(), number: 1, dialect: 1 })
        .expect("endpoint");
    agent.endpoints = smith_protocol_channel::Endpoints::new(entries);
    let root::Encoded::Turn(encoded) =
        crate::worker_route::encode(root::Encodable::Turn(fixtures::concrete_turn()), &agent.limits, &agent.endpoints)
    else {
        panic!("turn encoded");
    };
    assert!(skein_lib::bytes::find(&encoded.body, b"different").is_some());
    assert!(skein_lib::bytes::find(&encoded.body, b"fake").is_none());
}
#[test]
fn a_grant_s_value_reaches_smith_s_host_half_in_its_envelope() {
    let largest_id = Box::new([b'i'; 256]);
    let largest_token = Box::new([b't'; 7934]);
    for (label, id, token) in [
        ("short envelope", b"account".as_slice(), b"access".as_slice()),
        ("full envelope", largest_id.as_slice(), largest_token.as_slice()),
    ] {
        let mut service = fixtures::worker_service();
        let value = jig_protocol_accounts::table::Value {
            token: skein_lib::Secret::new(token.into()),
            account_id: skein_lib::Secret::new(id.into()),
        };
        worker(&mut service).granted(1, 1, value, Time::from_nanos(30_000_000_000));
        let body = start_on_pipes(&mut service);
        let agent = fixtures::agent();
        let start =
            smith_channel::Start::decode(&agent.limits.bodies, &mut skein_lib::Reader::new(&body)).expect(label);
        let grant = start.grants().get(0).expect(label);
        assert_eq!(grant.account(), 1, "{label}");
        assert_eq!(grant.generation(), 1, "{label}");
        let smith_channel::GrantValue::Withdrawn(value) = grant.value() else { panic!("{label}: withdrawn grant") };
        assert_eq!(value.id(), id, "{label}");
        assert_eq!(value.bearer(), &skein_lib::Secret::new(token.into()), "{label}");
    }
}
#[test]
fn the_agent_starts_with_no_credential_in_its_environment() {
    let mut service = fixtures::worker_service();
    assign(&mut service);
    let (_, launched) = spawn(&mut service);
    assert!(launched.env.is_empty());
    assert_eq!(launched.args.as_ref(), [b"agent".as_slice().into()]);
}
#[test]
fn stopped_the_worker_cancels_its_runs_and_never_redials() {
    let mut service = fixtures::worker_service();
    assign(&mut service);
    let (owner, _) = spawn(&mut service);
    worker(&mut service).stop();
    worker(&mut service).up(Time::ZERO, io::Event::Failed { owner, error: io::Error::Other });
    worker(&mut service).up(Time::ZERO, io::Event::Closed { owner });
    worker(&mut service).iterate(Time::ZERO, Wall::EPOCH);
    worker(&mut service)
        .step(root::Event::Link(jig_domain_host::LinkDown::Acknowledge { run: Token::new(7), attempt: Token::new(3) }));
    for _ in 0_u8..3 {
        worker(&mut service).iterate(Time::ZERO, Wall::EPOCH);
    }
    assert!(worker(&mut service).closed());
    worker(&mut service).iterate(Time::from_nanos(1_000_000_000_000), Wall::EPOCH);
    assert!(worker(&mut service).io.is_empty());
}
#[test]
fn a_worker_without_its_own_runtime_is_refused_at_its_root_path() {
    let agent = fixtures::agent();
    let files = fixtures::worker_files();
    let seeds = fixtures::seeds();
    for (label, runtime) in [
        ("runtime absent", None),
        ("descriptor absent", Some(crate::WorkerRuntime { roots: skein_lib::List::with_capacity(0) })),
    ] {
        let found =
            crate::startup(fixtures::WORKER_DOCUMENT, &files, Some(&agent), &seeds, None, runtime).err().expect(label);
        assert_eq!(
            found.problems.get(0),
            Some(&jig_protocol_config::Problem {
                path: jig_protocol_config::Path::root()
                    .member(b"application")
                    .member(b"workers")
                    .child(jig_protocol_config::Segment::Index(0))
                    .member(b"root"),
                what: jig_protocol_config::What::Refers,
            }),
            "{label}"
        );
    }
}

fn start_on_pipes(service: &mut Service) -> Box<[u8]> {
    use skein_lib::stream;
    let agent = fixtures::agent();
    let schema = smith_protocol_channel::schema_for_start(&agent.limits.bodies, agent.limits.start_bytes)
        .expect("channel schema");
    let mut peer_limits = agent.limits.channel;
    for version in &schema.versions {
        for kind in &version.kinds {
            if kind.direction == skein_channel::Direction::FromResponder {
                peer_limits.output_bytes =
                    peer_limits.output_bytes.max(kind.largest.checked_add(8).expect("frame bytes"));
            }
        }
    }
    let version = u16::try_from(smith_channel::KindRule::FORMAT_VERSION).expect("channel version");
    let mut peer = skein_fake_channel::ScriptedPeer::new(
        schema,
        skein_channel::Role::Responder,
        peer_limits,
        version,
        Box::default(),
    )
    .expect("agent peer");
    assign(service);
    let (owner, _) = spawn(service);
    worker(service).up(
        Time::ZERO,
        io::Event::Spawned {
            owner,
            child: Token::new(70),
            pipes: Box::new([Token::new(71), Token::new(72), Token::new(73)]),
        },
    );
    let mut buffer = skein_lib::Intake::with_capacity(1024);
    let mut read = None;
    for _ in 0_u16..500 {
        worker(service).iterate(Time::ZERO, Wall::EPOCH);
        for _ in 0..worker(service).io.len() {
            match worker(service).io.pop().expect("pipe requests") {
                io::Request::Output { stream, down: stream::OutputDown::Room { right, .. } } => worker(service).up(
                    Time::ZERO,
                    io::Event::Output {
                        owner: stream,
                        up: stream::OutputUp::Settled { right, outcome: stream::OutputOutcome::Granted },
                    },
                ),
                io::Request::Output { down: stream::OutputDown::Send { bytes, .. }, .. } => {
                    peer.feed(&bytes).expect("host bytes");
                }
                io::Request::Stream { stream, down: stream::Down::Demand { read: demand, .. } }
                    if stream == Token::new(72) =>
                {
                    read = Some(demand);
                }
                io::Request::Stream { stream, .. } if stream == Token::new(73) => {}
                io::Request::Connect { owner, .. } => {
                    worker(service).up(Time::ZERO, io::Event::Failed { owner, error: io::Error::Busy });
                }
                other @ (io::Request::Listen { .. }
                | io::Request::Bind { .. }
                | io::Request::Reject { .. }
                | io::Request::Stream { .. }
                | io::Request::Output { .. }
                | io::Request::Spawn { .. }
                | io::Request::Signal { .. }
                | io::Request::Usage { .. }
                | io::Request::Window { .. }
                | io::Request::Close { .. }
                | io::Request::Abort { .. }) => panic!("opening request: {other:?}"),
            }
        }
        while let Some(bytes) = peer.pop_output() {
            buffer.append(&bytes).expect("peer controls fit");
        }
        if let Some(demand) = read
            && let Some(bytes) = buffer.meet(demand)
        {
            read = None;
            worker(service).up(Time::ZERO, io::Event::Stream { owner: Token::new(72), up: stream::Up::Bytes(bytes) });
        }

        for record in peer.observed() {
            if record.kind == 0x0100 {
                return record.body.clone();
            }
        }
    }
    panic!("agent receives its start");
}

#[test]
fn a_stopped_worker_closes_the_root_its_shell_supplied() {
    for (label, two, expected) in [("one root", false, &[9_i32][..]), ("separate roots", true, &[9_i32, 10_i32][..])] {
        let mut service = if two { fixtures::two_workers() } else { fixtures::worker_service() };
        service.shutdown();
        iterate(&mut service, Time::ZERO, Wall::EPOCH);
        assert_eq!(service.submissions.len(), u32::try_from(expected.len()).expect("root count"), "{label}");
        for fd in expected {
            let submit = service.submissions.pop().expect(label);
            assert_eq!(submit.kind, io::kernel::Op::Close { fd: io::kernel::Fd::new(*fd) }, "{label}");
            service.completions.push(io::kernel::Complete {
                op: submit.op,
                kind: submit.kind,
                result: Ok(io::kernel::Done::Nothing),
            });
        }
        iterate(&mut service, Time::ZERO, Wall::EPOCH);
        assert_eq!(service.end(), Some(&crate::End::Drained), "{label}");
        assert!(service.is_empty(), "{label}");
    }
}

#[test]
fn the_engine_s_listener_opens_before_its_workers_start() {
    let document = fixtures::combined_document();
    let mut service = crate::startup(
        &document,
        &fixtures::combined_files(&document),
        Some(&fixtures::agent()),
        &fixtures::seeds(),
        None,
        Some(fixtures::worker_runtime()),
    )
    .expect("combined service");
    fixtures::memory_store(service.engine().expect("engine"));
    iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert!(!worker(&mut service).started);
    assert!(service.engine().expect("engine").store.above.is_empty());
    iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert!(worker(&mut service).started);
    let Some(jig_protocol_store::Event::Restored { .. }) = service.engine().expect("engine").store.above.pop() else {
        panic!("restart begins after listener");
    };
}

#[test]
fn a_worker_refuses_limits_that_cannot_carry_its_account_envelope() {
    let large =
        fixtures::with(fixtures::WORKER_DOCUMENT, b"\"hub.turns\":1", b"\"hub.turns\":1,\"accounts.token\":8192");
    let id = jig_protocol_worker_wire::CEILINGS.grant_account_id;
    let bearer = smith_channel::CEILINGS.withdrawn_value_bearer;
    for (label, document, id, bearer) in [
        ("caller bearer too small", large.as_ref(), id, bearer - 1),
        ("caller account id too small", fixtures::WORKER_DOCUMENT, id - 1, bearer),
    ] {
        let mut agent = fixtures::agent();
        agent.limits.bodies.withdrawn_value_id = id;
        agent.limits.bodies.withdrawn_value_bearer = bearer;
        let found = crate::startup(
            document,
            &fixtures::worker_files(),
            Some(&agent),
            &fixtures::seeds(),
            None,
            Some(fixtures::worker_runtime()),
        )
        .err()
        .expect(label);
        assert_eq!(
            found.problems.get(0),
            Some(&jig_protocol_config::Problem {
                path: jig_protocol_config::Path::root()
                    .member(b"workers")
                    .child(jig_protocol_config::Segment::Index(0))
                    .member(b"limits"),
                what: jig_protocol_config::What::Bound,
            }),
            "{label}"
        );
    }
}

#[test]
fn a_roots_step_count_moves_with_an_input_and_not_with_an_idle_iterate() {
    let mut service = fixtures::worker_service();
    let before = service.root_steps();
    iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert_eq!(service.root_steps(), before);
    assign(&mut service);
    assert!(service.root_steps() > before);
    let after = service.root_steps();
    iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert_eq!(service.root_steps(), after);
}
