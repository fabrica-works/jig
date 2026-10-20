//! Facts decoded independently at the byte boundary, before the root sees them.
use crate::native::Received;
use jig_client::{Message, Request as ClientRequest, contract as c};
use jig_client_protocol_wire::v10 as w;
use jig_client_test_domain::{Event, Request};
use jig_client_world::harness::{Receipt, Sent};
use skein_lib::{Reader, RequestEnvelope, Token};

#[derive(PartialEq, Eq)]
pub(super) enum Callback {
    Reply(Token),
    Record(Box<[u8]>),
    Gone,
    Heartbeat,
}
pub(super) fn callback(event: &jig_client::Event) -> Option<Callback> {
    match event {
        jig_client::Event::Reply { exchange, .. } => Some(Callback::Reply(*exchange)),
        jig_client::Event::Streamed { message: jig_client::Streamed::Record(bytes) } => {
            Some(Callback::Record(bytes.clone()))
        }
        jig_client::Event::Streamed { message: jig_client::Streamed::Heartbeat } => Some(Callback::Heartbeat),
        jig_client::Event::Streamed { message: jig_client::Streamed::SignedOut } | jig_client::Event::StreamGone => {
            Some(Callback::Gone)
        }
        jig_client::Event::Start { .. }
        | jig_client::Event::StartUnlinked { .. }
        | jig_client::Event::SignIn { .. }
        | jig_client::Event::SignOut
        | jig_client::Event::SignOutLost
        | jig_client::Event::Watch { .. }
        | jig_client::Event::Unwatch { .. }
        | jig_client::Event::Read { .. }
        | jig_client::Event::Send { .. }
        | jig_client::Event::Carry { .. }
        | jig_client::Event::Lost { .. }
        | jig_client::Event::Stored
        | jig_client::Event::NotStored
        | jig_client::Event::Up { .. } => None,
    }
}

#[must_use]
pub fn sent(request: &Request) -> Option<Sent> {
    let Request::Client { request } = request else { return None };
    client_sent(request)
}

/// Observe a linked client's request independently of its application's root type.
#[must_use]
pub fn client_sent(request: &ClientRequest) -> Option<Sent> {
    let ClientRequest::Exchange { exchange, message: Message::Request(bytes), .. } = request else { return None };
    let keyed = w::Keyed::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("keyed bytes");
    let mut sent = Sent { attempt: *exchange, key: *keyed.key(), card: None, revision: None, words: None };
    match keyed.owner() {
        w::Owner::Jig => {
            let ask = w::Ask::decode(&w::CEILINGS, &mut Reader::new(keyed.request())).expect("jig ask bytes");
            let decided = match ask {
                w::Ask::Compound(value) => {
                    let mut reader = Reader::new(value.operations());
                    let length = reader.u32().expect("piece length");
                    let bytes = reader.bytes(length).expect("first operation");
                    match w::Operation::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("operation") {
                        w::Operation::Message(v) => {
                            sent.words = Some((v.task(), v.words().to_vec()));
                            None
                        }
                        w::Operation::Amend(v) => {
                            v.revision().as_ref().map(|r| (c::ObjectKey::Task(v.task()), 1, r.value()))
                        }
                        w::Operation::Cancel(_)
                        | w::Operation::Release(_)
                        | w::Operation::Stop(_)
                        | w::Operation::MoveTask(_) => None,
                    }
                }
                w::Ask::Message(value) => {
                    sent.words = Some((value.task(), value.words().to_vec()));
                    None
                }
                w::Ask::Answer(value) => match value.on() {
                    w::AnswerOn::Question(v) => {
                        sent.words = Some((v.task(), v.words().to_vec()));
                        None
                    }
                    w::AnswerOn::Person(v) => Some((c::ObjectKey::PersonTask(v.task()), 3, v.revision().value())),
                },
                w::Ask::Release(value) => Some((c::ObjectKey::Task(value.task()), 2, value.revision().value())),
                w::Ask::Decide(value) => match value.on() {
                    w::DecideOn::Proposal(_) => None,
                    w::DecideOn::Escalation(v) => Some((c::ObjectKey::Escalation(v.task()), 2, v.revision().value())),
                },
                w::Ask::Amend(value) => {
                    value.revision().as_ref().map(|v| (c::ObjectKey::Task(value.task()), 1, v.value()))
                }
                w::Ask::EditNote(value) => Some((
                    c::ObjectKey::Note { project: value.project(), note: value.note() },
                    4,
                    value.revision().value(),
                )),
                w::Ask::DeleteNote(value) => Some((
                    c::ObjectKey::Note { project: value.project(), note: value.note() },
                    4,
                    value.revision().value(),
                )),
                w::Ask::Propose(value) => match value.action() {
                    w::ProposeAction::Batch(_) => None,
                    w::ProposeAction::Amendment(v) => Some((c::ObjectKey::Task(v.task()), 1, v.revision().value())),
                },
                w::Ask::Withdraw(_)
                | w::Ask::Create(_)
                | w::Ask::Take(_)
                | w::Ask::HandBack(_)
                | w::Ask::Stop(_)
                | w::Ask::Cancel(_)
                | w::Ask::MoveTask(_)
                | w::Ask::Seen(_)
                | w::Ask::ChangePolicy(_)
                | w::Ask::SetRoles(_)
                | w::Ask::MakeService(_)
                | w::Ask::SignIn(_)
                | w::Ask::AccountChange(_)
                | w::Ask::SignOut(_)
                | w::Ask::SetPool(_) => None,
            };
            if let Some((key, kind, revision)) = decided {
                sent.card = Some(jig_client_world::observe::key(key));
                sent.revision = Some((kind, revision));
            }
        }
        w::Owner::Family(_) => {
            use jig_client_test_protocol_wire::v1 as family;
            let envelope = w::FamilyRequest::decode(&w::CEILINGS, &mut Reader::new(keyed.request()))
                .expect("family request envelope");
            let request = family::Request::decode(&family::CEILINGS, &mut Reader::new(envelope.record()))
                .expect("family request");
            let subject = jig_client_test_domain::family::Subject {
                scope: jig_client_test_domain::family::Scope::Project(request.project()),
                kind: jig_client_test_domain::family::Kind::Item(request.value().object()),
            };
            sent.card = Some(jig_client_world::harness::CardKey::of(jig_client_test_view::family_key(&subject)));
            sent.revision = Some((0, request.value().revision()));
        }
    }
    Some(sent)
}

#[must_use]
pub fn receipts(fact: &Received) -> Vec<Receipt> {
    let event = match fact {
        Received::Answer { attempt, answer } => Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt: *attempt,
                    envelope: RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(answer.clone())),
                    why: None,
                },
            },
        },
        Received::Delivery(c::Delivery::Me(value)) => {
            return vec![Receipt::Party { party: value.party.number, watch: None }];
        }
        Received::Delivery(value) => Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch: Token::new(0), event: c::WatchEvent::Delivered(value.clone()) },
            },
        },
        Received::Read(result) => Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone { read: Token::new(0), result: jig_client::Readout::Jig(result.clone()) },
            },
        },
        Received::SignedIn { .. } | Received::Sent { .. } => return Vec::new(),
    };
    jig_client_world::observe::received(&event)
}
