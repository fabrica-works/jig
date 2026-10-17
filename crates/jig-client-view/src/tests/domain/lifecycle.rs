use super::spine::{LIMITS, delivered, drain, drive, frame, start};
use crate::tests::fixtures::refusal_answer;
use crate::tests::fixtures::{AnsweredByEngine, Ended, Fixture, ReadPage, Reported};
use crate::{
    Event, Request,
    action::Act,
    address::{Address, Place},
    fields::{Fields, Purpose},
    intents::{Progress, send_card},
    objects::{Card, Source},
    page::Page,
    store::{self, Record, STORE_VERSION, StoreKey},
};
use jig_client::contract::*;
use skein_lib::{Id, List, Queue, Time};
fn text(bytes: &[u8]) -> Text {
    Text::new(Box::from(bytes), 64).unwrap()
}
fn ask(asked: u64) -> crate::tests::fixtures::SendIntent {
    crate::tests::fixtures::intent(7, asked, Ask::Stop { project: 1, task: 8 })
}
fn send(domain: &mut Fixture, asked: u64) -> jig_client::Key {
    let out = drive(domain, Time::ZERO, ask(asked));
    let Request::Client { asked: jig_client::Event::Send { key: request, .. } } = out[0] else {
        panic!("one send");
    };
    request
}

#[test]
fn intents_join_progress_and_one_terminal_and_refuse_past_the_limit() {
    let mut domain = Fixture::new(&LIMITS);
    let f = start(&mut domain, List::with_capacity(0));
    delivered(&mut domain, Time::ZERO, f, frame(7));
    let key = ObjectKey::Task(8);
    let id = domain.objects.name(key, Source::Window { number: 1 }).unwrap();
    let mut out = Queue::with_capacity(16);
    let request = send_card(
        &mut domain.ui,
        key,
        ActKind::Stop,
        Box::new(Ask::Stop { project: 1, task: 8 }),
        &mut out,
        &domain.client,
    )
    .unwrap();
    assert_eq!(
        send_card(
            &mut domain.ui,
            key,
            ActKind::Stop,
            Box::new(Ask::Stop { project: 1, task: 8 }),
            &mut out,
            &domain.client
        ),
        Some(request)
    );
    assert_eq!(drain(out).len(), 1);
    let program = send(&mut domain, 40);
    assert!(drive(&mut domain, Time::ZERO, ask(40)).is_empty());
    let full = drive(&mut domain, Time::ZERO, ask(41));
    assert!(full.is_empty());
    assert!(drive(&mut domain, Time::ZERO, Reported { request, progress: Progress::Parked }).is_empty());
    assert_eq!(
        domain.objects().get(id).unwrap().card,
        Card::Deciding { request: Some(request), progress: Progress::Parked }
    );
    let answer = refusal_answer(Refusal::NotOffered);
    assert!(drive(&mut domain, Time::ZERO, AnsweredByEngine { request, answer: answer.clone() }).is_empty());
    assert_eq!(domain.intents().get(Id::from_token(request)).unwrap().answer, Some(answer.clone()));
    assert_eq!(domain.objects().get(id).unwrap().card, Card::Refused { refusal: Refusal::NotOffered });
    assert!(drive(&mut domain, Time::ZERO, AnsweredByEngine { request: program, answer: answer.clone() }).is_empty());
    assert_eq!(domain.intents().get(Id::from_token(domain.intent(program))).unwrap().answer, Some(answer.clone()));
    assert!(drive(&mut domain, Time::ZERO, AnsweredByEngine { request: program, answer }).is_empty());
    domain.reclaim();
    assert!(domain.intents().get(Id::from_token(request)).is_none());
}

#[test]
fn fields_count_only_domain_writes_and_records_restore_save_and_erase() {
    let mut fields = Fields::new(1, 4);
    assert_eq!(fields.edit(Purpose::NewChat, text(b"one")), Ok(()));
    assert_eq!(fields.get(Purpose::NewChat).unwrap().writes, 0);
    assert_eq!(fields.write(Purpose::NewChat, text(b"two")), Ok(()));
    assert_eq!(fields.edit(Purpose::NewChat, text(b"next")), Ok(()));
    assert_eq!(fields.get(Purpose::NewChat).unwrap().writes, 1);
    assert_eq!(fields.edit(Purpose::Reason, text(b"why")), Err(Limit::Fields));
    assert_eq!(fields.edit(Purpose::NewChat, text(b"large")), Err(Limit::Words { limit: 4 }));
    assert_eq!(fields.get(Purpose::NewChat).unwrap().text.bytes(), b"next");
    let mut kept = List::with_capacity(3);
    let draft = Record::Draft { version: STORE_VERSION, party: 7, task: 8, words: text(b"draft"), touched: 1 };
    kept.push(draft.clone()).unwrap();
    kept.push(Record::Place { version: 9, party: 8, address: Address::Deployment }).unwrap();
    let mut domain = Fixture::new(&LIMITS);
    let f = start(&mut domain, kept);
    assert_eq!(domain.records().as_slice(), core::slice::from_ref(&draft));
    delivered(&mut domain, Time::ZERO, f, frame(7));
    assert!(
        drive(
            &mut domain,
            Time::ZERO,
            Event::Act { action: Act::Edit { field: Purpose::NewChat, text: text(b"edited") } }
        )
        .is_empty()
    );
    assert_eq!(domain.fields().get(Purpose::NewChat).unwrap().writes, 0);
    let mut out = Queue::with_capacity(16);
    assert!(store::save(&mut domain, draft.clone(), &mut out));
    store::erase(&mut domain, StoreKey::Draft { task: 8 }, &mut out);
    assert_eq!(
        drain(out).as_ref(),
        &[
            Request::Save { records: crate::tests::fixtures::one(draft) },
            Request::Erase { records: crate::tests::fixtures::one(StoreKey::Draft { task: 8 }) }
        ]
    );
    assert!(domain.records().is_empty());
}

#[test]
fn signed_out_from_answer_read_or_watch_preserves_intents_and_returns_to_the_address() {
    for path in 0_u64..3 {
        let mut domain = Fixture::new(&LIMITS);
        let f = start(&mut domain, List::with_capacity(0));
        delivered(&mut domain, Time::ZERO, f, frame(7));
        let pending = send(&mut domain, 50);
        match path {
            0 => drive(&mut domain, Time::ZERO, jig_client::Up::SignedOut),
            1 => {
                let id = domain.request_read(Source::Window { number: 1 }, Read::Deployment).unwrap();
                drive(&mut domain, Time::ZERO, ReadPage { read: id, result: ReadResult::None(ReadNone::SignedOut) });
                drive(&mut domain, Time::ZERO, jig_client::Up::SignedOut)
            }
            2 => {
                drive(&mut domain, Time::ZERO, Ended { watch: f, end: WatchEnd::SignedOut });
                drive(&mut domain, Time::ZERO, jig_client::Up::SignedOut)
            }
            _ => unreachable!(),
        };
        assert_eq!(domain.page(), &Page::SigningIn);
        assert_eq!(domain.frame().party(), None);
        assert!(domain.intents().get(Id::from_token(domain.intent(pending))).unwrap().answer.is_none());
        drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Inbox { project: None }) });
        drive(&mut domain, Time::ZERO, jig_client::Up::SignedIn);
        drive(&mut domain, Time::ZERO, crate::tests::fixtures::Served { served: true });
        let fresh = domain.frame_watch.expect("the client holds the frame handle through sign-in");
        delivered(&mut domain, Time::ZERO, fresh, frame(7));
        assert_eq!(domain.frame().party(), Some(7));
        let Page::Work { address, .. } = domain.page() else {
            panic!("returned work page");
        };
        assert_eq!(*address, Address::Inbox { project: None });
    }
}

#[test]
fn a_service_never_opens_a_work_or_signin_page() {
    let mut domain = Fixture::new(&LIMITS);
    let f = start(&mut domain, List::with_capacity(0));
    let mut service = frame(7);
    service.party.kind = PartyKind::Service;
    let out = delivered(&mut domain, Time::ZERO, f, service);
    assert_eq!(out.len(), 0);
    assert_eq!(domain.page(), &Page::Elsewhere);
    drive(&mut domain, Time::ZERO, jig_client::Up::SignedOut);
    assert_eq!(domain.page(), &Page::Elsewhere);
    drive(&mut domain, Time::ZERO, jig_client::Up::SignedOut);
    assert_eq!(domain.page(), &Page::Elsewhere);
}
