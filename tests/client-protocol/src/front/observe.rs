//! Facts decoded independently at the byte boundary, before the root sees them.
use crate::native::Received;
use jig_client::{Message, Request as ClientRequest, contract as c};
use jig_client_protocol_wire::v8 as w;
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
        jig_client::Event::StreamGone => Some(Callback::Gone),
        jig_client::Event::Start { .. }
        | jig_client::Event::StartUnlinked { .. }
        | jig_client::Event::SignIn { .. }
        | jig_client::Event::SignOut
        | jig_client::Event::SignOutLost
        | jig_client::Event::Watch { .. }
        | jig_client::Event::Unwatch { .. }
        | jig_client::Event::Read { .. }
        | jig_client::Event::Send { .. }
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
                w::Ask::Write(value) => {
                    sent.words = Some((value.task(), value.words().to_vec()));
                    None
                }
                w::Ask::AnswerQuestion(value) => {
                    sent.words = Some((value.task(), value.words().to_vec()));
                    None
                }
                w::Ask::AnswerPerson(value) => {
                    Some((c::ObjectKey::PersonTask(value.task()), 3, value.revision().value()))
                }
                w::Ask::Release(value) => Some((c::ObjectKey::Task(value.task()), 2, value.revision().value())),
                w::Ask::DecideEscalation(value) => {
                    Some((c::ObjectKey::Escalation(value.task()), 2, value.revision().value()))
                }
                w::Ask::Amend(value) => Some((c::ObjectKey::Task(value.task()), 1, value.revision().value())),
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
                w::Ask::StartChat(_)
                | w::Ask::Take(_)
                | w::Ask::HandBack(_)
                | w::Ask::DecideProposal(_)
                | w::Ask::Stop(_)
                | w::Ask::Cancel(_)
                | w::Ask::Close(_)
                | w::Ask::TakeOver(_)
                | w::Ask::SetGoal(_)
                | w::Ask::Prioritise(_)
                | w::Ask::MarkSeen(_)
                | w::Ask::ChangePolicy(_)
                | w::Ask::SetRoles(_)
                | w::Ask::MakeService(_)
                | w::Ask::SetPool(_) => None,
            };
            if let Some((key, kind, revision)) = decided {
                sent.card = Some(jig_client_world::observe::key(key));
                sent.revision = Some((kind, revision));
            }
        }
        w::Owner::Family(_) => {
            use jig_client_test_protocol_wire::v1 as family;
            let request =
                family::Request::decode(&family::CEILINGS, &mut Reader::new(keyed.request())).expect("family request");
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
        Received::Delivery(c::Delivery::Frame(value)) => {
            return vec![Receipt::Party { party: value.party.number, watch: None }];
        }
        Received::Delivery(value) => Event::Jig(jig_client_view::Event::Watch {
            watch: Token::new(0),
            event: c::WatchEvent::Delivered(value.clone()),
        }),
        Received::Read(result) => {
            Event::Jig(jig_client_view::Event::Read { read: Token::new(0), result: result.clone() })
        }
        Received::SignedIn { .. } | Received::Sent { .. } => return Vec::new(),
    };
    jig_client_world::observe::received(&event)
}
