use super::fixtures;
use crate::{Event, FromApplication, Request, ToApplication, translate::watches};
use jig_domain_core as core;
use jig_protocol_client_wire::v6 as w;
use skein_lib::{Reader, Token};
fn stream_world() -> (fixtures::Exchange, [u8; 16]) {
    let mut world = fixtures::Exchange::new();
    let stream = world.open_stream();
    (world, stream)
}
fn request_task(world: &mut fixtures::Exchange, stream: [u8; 16], name: u64) -> (Token, Token) {
    let owner = world.open_record(stream, name, fixtures::owned_subject(&fixtures::task_subject()));
    (owner, world.take_watch())
}
#[test]
fn a_zero_default_window_refuses_to_start() {
    let mut settings = fixtures::settings();
    settings.rows = 0;
    let mut out = skein_lib::Queue::with_capacity(4);
    assert_eq!(crate::Component::new(settings, &fixtures::env(), &mut out).err(), Some(crate::Unusable::Limits));
    assert!(out.is_empty());
}
#[test]
fn every_watch_subject_preserves_its_identity_and_window() {
    for (label, source, subject, rows) in fixtures::subject_cases() {
        let Some(core::Event::Watch { watcher, sign_in, subject: actual, window }) =
            watches::watch(Token::new(11), 41, source, 32)
        else {
            panic!("{label}: watch");
        };
        assert_eq!(
            (watcher, sign_in, actual, window.rows, window.after),
            (Token::new(11), 41, subject, rows, None),
            "{label}"
        );
    }
}
#[test]
fn an_application_watch_keeps_its_owner_and_rejects_foreign_deliveries() {
    let (mut world, stream) = stream_world();
    let owner = world.open_record(stream, 17, fixtures::family_subject());
    let Some(Request::Application(ToApplication::Watch { watcher, sign_in, family, version, record })) =
        world.out.pop()
    else {
        panic!("family watch");
    };
    assert_eq!((sign_in, family, version, record), (41, 9, 2, Box::from([77])));
    world.component.down(&world.env, Event::Admitted { watcher }, &mut world.out);
    assert_eq!(
        w::OpenAnswer::decode(&fixtures::wire(), &mut Reader::new(world.response(owner).1)),
        Ok(w::OpenAnswer::Admitted)
    );
    world.component.down(
        &world.env,
        Event::Application(FromApplication::Delivery { watcher, family: 8, record: Box::new([77]) }),
        &mut world.out,
    );
    assert!(world.stream().watches.get(&watcher).expect("watch").delivery.is_none());
    world.component.down(
        &world.env,
        Event::Application(FromApplication::Delivery { watcher, family: 9, record: Box::new([88]) }),
        &mut world.out,
    );
    let packet =
        &world.stream().watches.get(&watcher).expect("watch").delivery.as_ref().expect("family delivery").event.data;
    let bytes = skein_lib::base64::decode(skein_lib::base64::Alphabet::Standard, packet, 256).expect("record");
    let w::StreamEvent::Delivery(value) =
        w::StreamEvent::decode(&fixtures::wire(), &mut Reader::new(&bytes)).expect("event")
    else {
        panic!("delivery");
    };
    assert_eq!((value.watch(), value.document().version(), value.document().record()), (17, 2, &[88][..]));
}
#[test]
fn a_watch_admission_or_refusal_cannot_answer_another_exchange() {
    let (mut world, stream) = stream_world();
    let (first, watcher) = request_task(&mut world, stream, 17);
    world.component.down(&world.env, Event::Admitted { watcher: Token::new(999) }, &mut world.out);
    assert!(world.stream().watches.get(&watcher).expect("watch").opening.is_some());
    world.component.down(&world.env, Event::WatchRefused { watcher, why: core::WatchRefusal::Busy }, &mut world.out);
    let expected = w::OpenAnswer::Refused(
        w::OpenAnswerRefused::new(&fixtures::wire(), w::OpenAnswerRefusedParts { why: w::WatchRefusal::Busy })
            .expect("busy"),
    );
    assert_eq!(w::OpenAnswer::decode(&fixtures::wire(), &mut Reader::new(world.response(first).1)), Ok(expected));
    let (second, other) = request_task(&mut world, stream, 18);
    world.component.down(&world.env, Event::Admitted { watcher }, &mut world.out);
    assert!(world.stream().watches.get(&other).expect("other watch").opening.is_some());
    assert!(
        world
            .component
            .pending
            .exchanges
            .get(&world.component.connections.get(&second).expect("second").exchange.expect("exchange"))
            .is_some()
    );
}
#[test]
fn waiting_for_room_keeps_one_delivery_per_watch() {
    let (mut world, stream) = stream_world();
    let (_, watcher) = request_task(&mut world, stream, 17);
    world.component.down(&world.env, Event::Admitted { watcher }, &mut world.out);
    let document = core::documents::Document::Frame(fixtures::frame_two_projects());
    for _ in 0_u8..3 {
        world.component.down(
            &world.env,
            Event::Delivery { watcher, document: Box::new(document.clone()) },
            &mut world.out,
        );
    }
    assert!(world.stream().waiting);
    assert!(world.stream().watches.get(&watcher).expect("watch").delivery.is_some());
    assert_eq!(world.stream().below.len(), 1);
    assert!(world.out.is_empty());
}

#[test]
fn each_watch_ending_preserves_its_reason() {
    use jig_domain_views::End;
    for (label, source, expected) in [
        ("Closed", End::Unwatched, w::WatchEnd::Closed),
        ("SubjectEnded", End::Gone, w::WatchEnd::SubjectEnded),
        ("NotAllowed", End::NotAllowed, w::WatchEnd::NotAllowed),
        ("SignedOut", End::SignedOut, w::WatchEnd::SignedOut),
        ("Stopping", End::Stopping, w::WatchEnd::Stopping),
    ] {
        assert_eq!(watches::ended(source), expected, "{label}");
    }
}
#[test]
fn each_watch_refusal_preserves_its_reason() {
    use core::WatchRefusal;
    for (label, source, expected) in [
        ("Unknown", WatchRefusal::Unknown, w::WatchRefusal::Unknown),
        ("NotAllowed", WatchRefusal::NotAllowed, w::WatchRefusal::NotAllowed),
        ("Busy", WatchRefusal::Busy, w::WatchRefusal::Busy),
        ("SignedOut", WatchRefusal::SignedOut, w::WatchRefusal::SignedOut),
    ] {
        assert_eq!(watches::refused(source), expected, "{label}");
    }
}
#[test]
fn unwatch_answers_only_after_its_released_ending() {
    let (mut world, stream) = stream_world();
    let (_, watcher) = request_task(&mut world, stream, 17);
    world.component.down(&world.env, Event::Admitted { watcher }, &mut world.out);
    let owner = world.unwatch(stream, 17);
    let Some(Request::Core(event)) = world.out.pop() else {
        panic!("unwatch");
    };
    let core::Event::Unwatch { watcher: actual } = *event else {
        panic!("unwatch");
    };
    assert_eq!(actual, watcher);
    let exchange = world.component.connections.get(&owner).expect("connection").exchange.expect("exchange");
    world.component.down(&world.env, Event::Admitted { watcher }, &mut world.out);
    assert!(world.component.pending.exchanges.get(&exchange).is_some());
    world.component.down(
        &world.env,
        Event::WatchEnded { watcher, why: jig_domain_views::End::Unwatched },
        &mut world.out,
    );
    assert_eq!(
        w::CloseAnswer::decode(&fixtures::wire(), &mut Reader::new(world.response(owner).1)),
        Ok(w::CloseAnswer::Closed)
    );
}
#[test]
fn a_run_watch_keeps_the_complete_provisional_position() {
    let limits = fixtures::wire();
    let rows = [
        ("Absent", None, None),
        (
            "Full",
            Some(
                w::ProvisionalTurnName::new(
                    &limits,
                    w::ProvisionalTurnNameParts { attempt: u64::MAX, number: u32::MAX, retry: u32::MAX },
                )
                .expect("turn"),
            ),
            Some(jig_domain_views::TurnName { attempt: u64::MAX, turn: u32::MAX, retry: u32::MAX }),
        ),
    ];
    for (label, turn, expected) in rows {
        let source = w::Subject::Run(
            w::SubjectRun::new(&limits, w::SubjectRunParts { task: u64::MAX, turn, offset: u32::MAX }).expect("run"),
        );
        let Some(core::Event::WatchRun { watcher, sign_in, task, turn, offset }) =
            watches::watch(Token::new(11), 41, source, 32)
        else {
            panic!("{label}: run");
        };
        assert_eq!(
            (watcher, sign_in, task, turn, offset),
            (Token::new(11), 41, u64::MAX, expected, u32::MAX),
            "{label}"
        );
    }
}

#[test]
fn a_heartbeat_waiting_for_room_keeps_its_stall_deadline() {
    let env = fixtures::env();
    let mut stream = crate::stream::Stream::new([7; 16], 41, &env);
    stream.heartbeat(&env);
    assert!(stream.waiting);
    assert_eq!(stream.deadline, env.now.saturating_add(env.limits.stall));
    assert_eq!(stream.below.len(), 1);
}

#[test]
fn an_absent_stream_digest_is_answered_signed_out() {
    let mut world = fixtures::Exchange::new();
    let owner = world.post_to_method(b"/jig/v1/stream", Box::new([]), skein_http::Method::Get);
    let Some(Request::Secrets(crate::SecretsRequest::Get { owner: wanted })) = world.out.pop() else {
        panic!("digest get");
    };
    world.component.down(
        &world.env,
        Event::Secrets(crate::SecretsAnswer::Got { owner: wanted, value: None }),
        &mut world.out,
    );
    assert_eq!(world.response(owner), (401, &[][..]));
    assert!(world.component.streams.is_empty());
}
