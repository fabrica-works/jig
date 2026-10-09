use super::spine::{LIMITS, delivered, drain, drive, frame, start};
use crate::{
    Domain, Event, Request,
    action::Act,
    address::{Address, Place},
    contract::*,
    fields::{Fields, Purpose},
    intents::{ByValue, Progress, send_card},
    objects::{Card, Source},
    pages::Page,
    signin::Provider,
    store::{self, Record, STORE_VERSION, StoreKey},
};
use skein_lib::{Id, List, Queue, Time, Token};
fn text(bytes: &[u8]) -> Text {
    Text::new(Box::from(bytes), 64).unwrap()
}
fn ask(asked: u64) -> ByValue {
    ByValue { asked: Token::new(asked), scope: 7, ask: Box::new(Ask::Stop { project: 1, task: 8 }) }
}
fn send(domain: &mut Domain, asked: u64) -> Token {
    let out = drive(domain, Time::ZERO, Event::Ask { ask: ask(asked) });
    let Request::Send { request, .. } = out[0] else {
        panic!("one send");
    };
    request
}

#[test]
fn intents_join_progress_and_one_terminal_and_refuse_past_the_limit() {
    let mut domain = Domain::new(&LIMITS, 1);
    let f = start(&mut domain, List::with_capacity(0));
    delivered(&mut domain, Time::ZERO, f, frame(7));
    let key = ObjectKey::Task(8);
    let id = domain.objects.name(key, Source::Window { generation: 0, number: 1 }).unwrap();
    let mut out = Queue::with_capacity(16);
    let request =
        send_card(&mut domain, key, ActKind::Stop, Box::new(Ask::Stop { project: 1, task: 8 }), &mut out).unwrap();
    assert_eq!(
        send_card(&mut domain, key, ActKind::Stop, Box::new(Ask::Stop { project: 1, task: 8 }), &mut out),
        Some(request)
    );
    assert_eq!(drain(out).len(), 1);
    let program = send(&mut domain, 40);
    assert!(drive(&mut domain, Time::ZERO, Event::Ask { ask: ask(40) }).is_empty());
    let full = drive(&mut domain, Time::ZERO, Event::Ask { ask: ask(41) });
    assert_eq!(full.len(), 2);
    assert_eq!(
        full[1],
        Request::Answered { asked: Token::new(41), answer: Answer::Refused(Refusal::Limit { which: Limit::Requests }) }
    );
    assert!(drive(&mut domain, Time::ZERO, Event::Progress { request, progress: Progress::Parked }).is_empty());
    assert_eq!(
        domain.objects.get(id).unwrap().card,
        Card::Deciding { request: Some(request), progress: Progress::Parked }
    );
    let answer = Answer::Refused(Refusal::NotOffered);
    assert!(drive(&mut domain, Time::ZERO, Event::Answered { request, answer: answer.clone() }).is_empty());
    assert_eq!(domain.intents().get(Id::from_token(request)).unwrap().answer, Some(answer.clone()));
    assert_eq!(domain.objects.get(id).unwrap().card, Card::Refused { refusal: Refusal::NotOffered });
    assert_eq!(
        drive(&mut domain, Time::ZERO, Event::Answered { request: program, answer: answer.clone() }).as_ref(),
        &[Request::Answered { asked: Token::new(40), answer: answer.clone() }]
    );
    assert!(drive(&mut domain, Time::ZERO, Event::Answered { request: program, answer }).is_empty());
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
    let mut domain = Domain::new(&LIMITS, 2);
    let f = start(&mut domain, kept);
    assert_eq!(domain.records().as_slice(), core::slice::from_ref(&draft));
    delivered(&mut domain, Time::ZERO, f, frame(7));
    assert!(
        drive(
            &mut domain,
            Time::ZERO,
            Event::Act { act: Act::Edit { field: Purpose::NewChat, text: text(b"edited") } }
        )
        .is_empty()
    );
    assert_eq!(domain.fields().get(Purpose::NewChat).unwrap().writes, 0);
    let mut out = Queue::with_capacity(16);
    assert!(store::save(&mut domain, draft.clone(), &mut out));
    store::erase(&mut domain, StoreKey::Draft { task: 8 }, &mut out);
    assert_eq!(
        drain(out).as_ref(),
        &[Request::Save { record: draft }, Request::Erase { key: StoreKey::Draft { task: 8 } }]
    );
    assert!(domain.records().is_empty());
}

#[test]
fn signed_out_from_answer_read_or_watch_preserves_intents_and_returns_to_the_address() {
    for path in 0..3 {
        let mut domain = Domain::new(&LIMITS, path);
        domain.provider(Provider { number: 2, name: text(b"Provider") }).unwrap();
        let f = start(&mut domain, List::with_capacity(0));
        delivered(&mut domain, Time::ZERO, f, frame(7));
        let pending = send(&mut domain, 50);
        let event = match path {
            0 => Event::SignedOut,
            1 => {
                let mut out = Queue::with_capacity(8);
                let id = domain
                    .reads
                    .ask(0, Source::Window { generation: 0, number: 1 }, Read::Deployment, &mut out)
                    .unwrap();
                Event::Read { read: id.token(), result: ReadResult::None(ReadNone::SignedOut) }
            }
            2 => Event::Ended { watch: f, end: WatchEnd::SignedOut },
            _ => unreachable!(),
        };
        let ended = drive(&mut domain, Time::ZERO, event);
        assert!(ended.contains(&Request::Confirmed { party: None }));
        assert_eq!(domain.page(), &Page::SigningIn);
        assert_eq!(domain.frame().party(), None);
        assert!(domain.intents().get(Id::from_token(pending)).unwrap().answer.is_none());
        assert_eq!(domain.signin().providers().len(), 1);
        assert_eq!(
            drive(&mut domain, Time::ZERO, Event::Act { act: Act::SignIn { provider: 2 } }).as_ref(),
            &[Request::SignIn { provider: 2, back: Address::Inbox { project: None } }]
        );
        assert!(drive(&mut domain, Time::ZERO, Event::Act { act: Act::SignIn { provider: 3 } }).is_empty());
        let returned =
            drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::Inbox { project: None }) });
        let opened = if path == 2 {
            returned
        } else {
            drive(&mut domain, Time::ZERO, Event::Ended { watch: f, end: WatchEnd::Closed })
        };
        let mut fresh = None;
        for request in &opened {
            if let Request::Open { watch, subject: Subject::Frame } = request {
                fresh = Some(*watch);
            }
        }
        let fresh = fresh.expect("return reopens the frame after its old terminal");
        let confirmed = delivered(&mut domain, Time::ZERO, fresh, frame(7));
        assert!(confirmed.contains(&Request::Confirmed { party: Some(7) }));
        let Page::Work { address, .. } = domain.page() else {
            panic!("returned work page");
        };
        assert_eq!(*address, Address::Inbox { project: None });
    }
}

#[test]
fn a_service_never_opens_a_work_or_signin_page() {
    let mut domain = Domain::new(&LIMITS, 3);
    let f = start(&mut domain, List::with_capacity(0));
    let mut service = frame(7);
    service.party.kind = PartyKind::Service;
    let out = delivered(&mut domain, Time::ZERO, f, service);
    assert_eq!(out.len(), 3);
    assert_eq!(domain.page(), &Page::Elsewhere);
    drive(&mut domain, Time::ZERO, Event::SignedOut);
    assert_eq!(domain.page(), &Page::Elsewhere);
    drive(&mut domain, Time::ZERO, Event::SignedOut);
    assert_eq!(domain.page(), &Page::Elsewhere);
}
