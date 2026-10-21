use super::{
    fixtures::{self, CREDENTIAL, Client, NEW_CREDENTIAL, SERVICE_CREDENTIAL, STOP, TASK},
    *,
};
use jig_client_protocol_wire::v11 as w;
use skein_lib::{Duration, Time};
const FIRST_BACKOFF: Duration = Duration::from_millis(10);
const LEAST_BACKOFF: Duration = Duration::from_millis(5);
const HELD_WATCHES: u32 = 2;

#[test]
fn a_watch_asked_before_signing_in_opens_once_signed_in() {
    let mut c = Client::new();
    c.step(Event::Watch { handle: ME, subject: Watchable::Jig(contract::Subject::Me) });
    assert!(c.requests().is_empty());
    c.step(Event::Watch { handle: TASK, subject: task() });
    assert!(c.requests().is_empty());
    c.step(Event::SignIn { with: With::Held(fixtures::credential(&CREDENTIAL)) });
    let [Request::Told { told }, Request::OpenStream { credential }] = c.requests() else {
        panic!("starting with held credential: {:?}", c.requests());
    };
    assert_eq!(*told, Told::Link { link: Link::Starting });
    assert_eq!(credential, &Some(fixtures::credential(&CREDENTIAL)));
    c.stream(fixtures::hello(jig_client_protocol_wire::VERSION));
    assert_eq!(c.requests.len(), 3);
    let Request::Save { records } = c.requests.last().expect("hello saves its stream") else {
        panic!("hello saves its stream");
    };
    assert_eq!(records.len(), 1);
    assert_eq!(records.as_slice().first().unwrap().key.as_ref(), STREAM_KEY);
    c.requests.truncate(HELD_WATCHES);
    assert_eq!(c.requests().len(), 2);
    for request in c.requests() {
        let Request::Exchange { credential, message: Message::Watch(_), .. } = request else {
            panic!("one exchange for each held watch: {request:?}");
        };
        assert_eq!(credential, &Some(fixtures::credential(&CREDENTIAL)));
    }
}

#[test]
fn signed_out_ends_each_handle_once_and_parks_each_request() {
    for terminal in fixtures::SIGNING_TERMINALS {
        let mut c = Client::narrow_signed_in();
        c.step(Event::Watch { handle: TASK, subject: task() });
        let opening = c.exchange();
        let watch = c.opened_watch();
        c.step(Event::Send { name: None, key: STOP, request: fixtures::stop() });
        c.step(Event::Stored);
        let request = c.exchange();
        c.step(Event::Read { handle: ME, read: Readable::Jig(contract::Read::Project { project: PROJECT }) });
        let read = c.exchange();
        c.signed_out_by(terminal, &fixtures::SigningExchanges { request, read, opening, watch });
        assert_eq!(fixtures::ended(c.requests()).as_slice(), &[(TASK, Why::SignedOut)], "{terminal:?}");
        assert_eq!(c.machine.link().state, Link::SignedOut, "{terminal:?}");
        assert_eq!(c.machine.link().party, None, "{terminal:?}");
        assert_eq!(c.machine.link().credential, None, "{terminal:?}");
        assert_eq!(c.machine.pending(STOP).unwrap().progress, Progress::Parked, "{terminal:?}");
        assert_eq!(c.machine.document(TASK).unwrap().state, State::Ended, "{terminal:?}");
        c.step(Event::Reply { exchange: request, message: fixtures::answer(&w::KeyedAnswer::SignedOut) });
        assert!(c.requests().is_empty(), "{terminal:?}");
        c.step(Event::Reply { exchange: read, message: fixtures::unknown_read() });
        assert!(c.requests().is_empty(), "{terminal:?}");
    }
}
#[test]
fn a_lost_sign_out_keeps_the_credential_and_says_so() {
    let mut c = Client::signed_in();
    c.step(Event::Watch { handle: TASK, subject: task() });
    c.step(Event::SignOut);
    c.step(Event::Lost { exchange: c.exchange() });
    assert_eq!(c.machine.link().state, Link::Live);
    assert_eq!(c.machine.link().credential, Some(&fixtures::credential(&CREDENTIAL)));
    assert!(c.machine.link().sign_out_lost);
    assert_eq!(fixtures::ended(c.requests()).as_slice(), &[]);
    assert_eq!(c.machine.document(TASK).unwrap().state, State::Opening);
}
#[test]
fn an_earlier_sign_in_s_sign_out_answer_is_dropped() {
    let mut c = Client::signed_in();
    c.step(Event::SignOut);
    let exchange = c.exchange();
    c.step(Event::SignIn { with: With::Held(fixtures::credential(&NEW_CREDENTIAL)) });
    c.stream(fixtures::hello(jig_client_protocol_wire::VERSION));
    let watch = c.opened_watch();
    c.stream(fixtures::frame(watch));
    c.step(Event::Reply { exchange, message: fixtures::sign_out_answer(&w::SignOutAnswer::SignedOut) });
    assert_eq!(c.machine.link().state, Link::Live);
    assert_eq!(c.machine.link().credential, Some(&fixtures::credential(&NEW_CREDENTIAL)));
    assert!(!c.machine.link().sign_out_lost);
}
#[test]
fn a_service_signs_in_with_its_credential_as_the_message_s() {
    let mut c = Client::new();
    c.step(Event::SignIn { with: With::Service(fixtures::credential(&SERVICE_CREDENTIAL)) });
    let [Request::Exchange { exchange, credential, message }] = c.requests() else {
        panic!("one service sign-in: {:?}", c.requests());
    };
    assert_eq!(message, &Message::SignInService);
    assert_eq!(credential, &Some(fixtures::credential(&SERVICE_CREDENTIAL)));
    let exchange = *exchange;
    c.step(Event::Reply { exchange, message: fixtures::signed_in(&NEW_CREDENTIAL) });
    let [Request::Told { told }, Request::OpenStream { credential }] = c.requests() else {
        panic!("starting with new credential: {:?}", c.requests());
    };
    assert_eq!(*told, Told::Link { link: Link::Starting });
    assert_eq!(credential, &Some(fixtures::credential(&NEW_CREDENTIAL)));
}
#[test]
fn under_transport_exchanges_carry_no_credential() {
    let mut c = Client::new();
    c.step(Event::SignIn { with: With::Transport });
    let [Request::Told { told }, Request::OpenStream { credential }] = c.requests() else {
        panic!("starting with transport credential: {:?}", c.requests());
    };
    assert_eq!(*told, Told::Link { link: Link::Starting });
    assert_eq!(credential, &None);
    c.stream(fixtures::hello(jig_client_protocol_wire::VERSION));
    let watch = c.opened_watch();
    c.stream(fixtures::frame(watch));
    c.step(Event::Watch { handle: TASK, subject: task() });
    let [Request::Exchange { credential, .. }] = c.requests() else {
        panic!("one watch exchange: {:?}", c.requests());
    };
    assert_eq!(credential, &None);
    assert_eq!(c.machine.link().credential, None);
}
#[test]
fn a_held_credential_never_reaches_the_small_store() {
    let mut c = Client::signed_in();
    c.step(Event::Send { name: None, key: STOP, request: fixtures::stop() });
    let records = fixtures::saved(c.requests());
    assert_eq!(records.len(), 1);
    for record in &records {
        for bytes in record.key.windows(CREDENTIAL.len()) {
            assert_ne!(bytes, CREDENTIAL);
        }
        for bytes in record.bytes.windows(CREDENTIAL.len()) {
            assert_ne!(bytes, CREDENTIAL);
        }
    }
}
#[test]
fn signing_out_again_after_a_lost_sign_out_is_answered_signed_out() {
    let mut c = Client::signed_in();
    c.step(Event::SignOut);
    c.step(Event::Lost { exchange: c.exchange() });
    assert!(c.machine.link().sign_out_lost);
    c.step(Event::SignOut);
    assert!(!c.machine.link().sign_out_lost);
    c.step(Event::Reply { exchange: c.exchange(), message: fixtures::sign_out_answer(&w::SignOutAnswer::SignedOut) });
    assert_eq!(c.machine.link().state, Link::SignedOut);
    assert_eq!(c.machine.link().credential, None);
    assert!(!c.machine.link().sign_out_lost);
}
#[test]
fn a_sign_out_answered_not_ready_did_not_reach_the_engine() {
    let mut c = Client::signed_in();
    c.step(Event::SignOut);
    c.step(Event::Reply { exchange: c.exchange(), message: fixtures::sign_out_answer(&w::SignOutAnswer::NotReady) });
    assert_eq!(c.machine.link().state, Link::Live);
    assert_eq!(c.machine.link().credential, Some(&fixtures::credential(&CREDENTIAL)));
    assert!(c.machine.link().sign_out_lost);
}
#[test]
fn a_service_s_sign_in_answered_not_ready_is_sent_again_after_its_backoff() {
    let mut c = Client::new();
    c.step(Event::SignIn { with: With::Service(fixtures::credential(&SERVICE_CREDENTIAL)) });
    let first = c.exchange();
    c.step(Event::Reply { exchange: first, message: fixtures::sign_in_answer(&w::SignInAnswer::NotReady) });
    assert!(c.requests().is_empty());
    let until = c.machine.next_deadline().unwrap();
    assert!(until >= Time::ZERO.saturating_add(LEAST_BACKOFF));
    assert!(until <= Time::ZERO.saturating_add(FIRST_BACKOFF));
    c.fire(until);
    let [Request::Exchange { exchange, credential, message }] = c.requests() else {
        panic!("one service retry: {:?}", c.requests());
    };
    assert_ne!(*exchange, first);
    assert_eq!(message, &Message::SignInService);
    assert_eq!(credential, &Some(fixtures::credential(&SERVICE_CREDENTIAL)));
}

#[test]
fn a_sign_out_that_did_not_reach_the_engine_tells_the_link() {
    let mut c = Client::signed_in();
    c.step(Event::SignOut);
    c.step(Event::Lost { exchange: c.exchange() });
    let mut links = skein_lib::List::with_capacity(1);
    for request in c.requests() {
        match request {
            Request::Told { told: Told::Link { link } } => links.push(*link).unwrap(),
            Request::Exchange { .. }
            | Request::OpenStream { .. }
            | Request::ServiceCredential { .. }
            | Request::CloseStream
            | Request::Save { .. }
            | Request::Erase { .. }
            | Request::Told { .. }
            | Request::Down { .. }
            | Request::SignOut => {}
        }
    }
    assert_eq!(links.as_slice(), &[Link::Live]);
    assert!(c.machine.link().sign_out_lost);
}

const MADE_PARTY: u64 = 7;
const CREDENTIALS_PER_REPLY: u32 = 1;
#[test]
fn the_link_s_service_credential_leaves_once_as_a_host_duty() {
    let mut c = Client::signed_in();
    c.step(Event::Send { name: None, key: STOP, request: fixtures::stop() });
    c.step(Event::Stored);
    let exchange = c.exchange();
    let reply = service_answer();
    c.step(Event::Reply { exchange, message: reply.clone() });
    let mut credentials = skein_lib::List::with_capacity(CREDENTIALS_PER_REPLY);
    for request in c.requests() {
        match request {
            Request::ServiceCredential { party, credential } => credentials.push((*party, credential.clone())).unwrap(),
            Request::Exchange { .. }
            | Request::OpenStream { .. }
            | Request::CloseStream
            | Request::Save { .. }
            | Request::Erase { .. }
            | Request::Told { .. }
            | Request::Down { .. }
            | Request::SignOut => {}
        }
    }
    assert_eq!(credentials.as_slice(), &[(MADE_PARTY, fixtures::credential(&SERVICE_CREDENTIAL))]);
    c.step(Event::Reply { exchange, message: reply });
    assert!(c.requests().is_empty());
}
fn service_answer() -> Box<[u8]> {
    let limits = w::CEILINGS;
    let service = w::MadeService::new(
        &limits,
        w::MadeServiceParts { party: MADE_PARTY, credential: fixtures::credential(&SERVICE_CREDENTIAL) },
    )
    .unwrap();
    let made = w::OutcomeMade::new(&limits, w::OutcomeMadeParts { what: w::Made::Service(service) }).unwrap();
    let done = w::Done::new(&limits, w::DoneParts { outcome: w::Outcome::Made(made) }).unwrap();
    let value = w::Final::Done(done);
    let mut bytes = skein_lib::Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut bytes).unwrap();
    let owned = w::Owned::new(
        &limits,
        w::OwnedParts { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: bytes.finish() },
    )
    .unwrap();
    fixtures::answer(&w::KeyedAnswer::FinalAnswer(owned))
}
