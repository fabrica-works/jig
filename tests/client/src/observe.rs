//! The test application's public document and accessibility vocabulary.
#![allow(clippy::wildcard_imports, reason = "boundary adapter covers the owner vocabulary")]
use crate::harness::{CardKey, CardSeen, Receipt, Seen, Sent};
use jig_client::contract::*;
use jig_client_view::{Key, NodeId, Owner, Role, Tree};
use jig_fake_person::{Face, Find, Found, TreeFace};
use jig_test_client_domain::{self as domain, Event, Request, family};
use skein_lib::RequestEnvelope;

pub fn key(object: ObjectKey) -> CardKey {
    let (kind, a, b) = match object {
        ObjectKey::Task(task) => (0, task, 0),
        ObjectKey::Proposal(p) => (1, p.proposer, p.number),
        ObjectKey::Escalation(task) => (2, task, 0),
        ObjectKey::Question { task, message } => (3, task, message),
        ObjectKey::PersonTask(task) => (4, task, 0),
        ObjectKey::Result(task) => (5, task, 0),
        ObjectKey::Note { project, note } => (6, u64::from(project), note),
    };
    CardKey::of(Key { owner: Owner::Jig, kind, a, b })
}
pub fn sent(request: &Request) -> Option<Sent> {
    let Request::Send { attempt, key: k, ask } = request else {
        return None;
    };
    let (card, revision, words) = match ask {
        domain::Ask::First(family::Ask::Touch { project, item, revision, .. }) => (
            Some(CardKey::of(jig_test_client_view::family_key(&family::Subject {
                scope: family::Scope::Project(*project),
                kind: family::Kind::Item(*item),
            }))),
            Some((0, revision.value)),
            None,
        ),
        domain::Ask::Jig(ask) => match ask.as_ref() {
            Ask::Write { task, words, .. } | Ask::AnswerQuestion { task, words, .. } => {
                (None, None, Some((*task, words.bytes().to_vec())))
            }
            Ask::AnswerPerson { task, revision, .. } => {
                (Some(key(ObjectKey::PersonTask(*task))), Some((3, revision.value)), None)
            }
            Ask::Release { task, revision, .. } => (Some(key(ObjectKey::Task(*task))), Some((2, revision.value)), None),
            Ask::DecideEscalation { task, revision, .. } => {
                (Some(key(ObjectKey::Escalation(*task))), Some((2, revision.value)), None)
            }
            Ask::Amend { task, revision, .. } => (Some(key(ObjectKey::Task(*task))), Some((1, revision.value)), None),
            Ask::EditNote { project, note, revision, .. } | Ask::DeleteNote { project, note, revision } => {
                (Some(key(ObjectKey::Note { project: *project, note: *note })), Some((4, revision.value)), None)
            }
            Ask::DecideProposal { proposal, .. } => (Some(key(ObjectKey::Proposal(*proposal))), None, None),
            Ask::StartChat { .. }
            | Ask::Take { .. }
            | Ask::HandBack { .. }
            | Ask::Stop { .. }
            | Ask::Cancel { .. }
            | Ask::Close { .. }
            | Ask::TakeOver { .. }
            | Ask::SetGoal { .. }
            | Ask::Prioritise { .. }
            | Ask::MarkSeen { .. }
            | Ask::Adopt { .. }
            | Ask::ChangePolicy { .. }
            | Ask::SetRoles { .. }
            | Ask::MakeService { .. }
            | Ask::SetPool { .. } => (None, None, None),
        },
    };
    Some(Sent { attempt: *attempt, key: *k, card, revision, words })
}
fn task(t: &Task) -> Receipt {
    let mut revisions = vec![(1, t.spec_revision.value)];
    if let Some(hold) = &t.hold {
        revisions.push((2, hold.revision.value));
    }
    Receipt::Card { key: key(ObjectKey::Task(t.chip.task)), revisions }
}
fn read(v: &ReadValue) -> Vec<Receipt> {
    match v {
        ReadValue::Task(t) => vec![task(t)],
        ReadValue::Frame(f) => frame(f),
        ReadValue::Escalation(e) => vec![Receipt::Card {
            key: key(ObjectKey::Escalation(e.task.task)),
            revisions: vec![(1, e.steering.revision.value), (2, e.hold.revision.value)],
        }],
        ReadValue::PersonTask(p) => {
            vec![Receipt::Card { key: key(ObjectKey::PersonTask(p.task.task)), revisions: vec![(3, p.revision.value)] }]
        }
        ReadValue::Notes(p) => p
            .rows
            .iter()
            .filter_map(|n| match &n.scope {
                NoteScope::Project(project) | NoteScope::Resource { project, .. } => Some(Receipt::Card {
                    key: key(ObjectKey::Note { project: *project, note: n.number }),
                    revisions: vec![(4, n.revision.value)],
                }),
                NoteScope::Deployment | NoteScope::Goal(_) => None,
            })
            .collect(),
        ReadValue::Proposal(_)
        | ReadValue::Question(_)
        | ReadValue::Result(_)
        | ReadValue::Ended(_)
        | ReadValue::History(_)
        | ReadValue::Activity(_)
        | ReadValue::Inbox(_)
        | ReadValue::Conversations(_)
        | ReadValue::Goals(_)
        | ReadValue::Spend(_)
        | ReadValue::Project(_)
        | ReadValue::Deployment(_) => vec![],
    }
}
fn frame(frame: &Frame) -> Vec<Receipt> {
    if frame.versions.first > 1 || frame.versions.last < 1 {
        vec![Receipt::Parked]
    } else {
        vec![Receipt::Party { party: frame.party.number }]
    }
}
fn family(doc: &family::Document) -> Receipt {
    let revision = match doc {
        family::Document::Item(item) => item.revision,
        family::Document::Part(part) => part.parameters.map_or(part.revision, |p| p.revision),
    };
    Receipt::Card {
        key: CardKey::of(jig_test_client_view::family_key(&doc.subject())),
        revisions: vec![(0, revision.value)],
    }
}
pub fn received(event: &Event) -> Vec<Receipt> {
    match event {
        Event::Answer { attempt, envelope, answer } => {
            let words = match answer {
                Some(domain::Answered::Jig(Answer::Done(Outcome::Made { what: Made::Words { task, message } }))) => {
                    Some((*task, *message))
                }
                Some(domain::Answered::Jig(_) | domain::Answered::First(_)) | None => None,
            };
            if *envelope == RequestEnvelope::SignedOut {
                vec![Receipt::Parked]
            } else {
                vec![Receipt::Answer { attempt: *attempt, final_answer: *envelope == RequestEnvelope::Final, words }]
            }
        }
        Event::Jig(jig_client::Event::Read { result: ReadResult::Value(v), .. }) => read(v),
        Event::Jig(jig_client::Event::Watch { event: WatchEvent::Delivered(Delivery::Task(t)), .. }) => vec![task(t)],
        Event::Jig(jig_client::Event::Watch { event: WatchEvent::Delivered(Delivery::Frame(f)), .. }) => frame(f),
        Event::First(
            domain::first::Event::Read { result: family::ReadResult::Value(doc), .. }
            | domain::first::Event::Watch { event: family::WatchEvent::Delivered(doc), .. },
        ) => vec![family(doc)],
        Event::SignedOut => vec![Receipt::Parked],
        Event::Start { .. } | Event::Went { .. } | Event::Act { .. } | Event::Jig(_) | Event::First(_) => vec![],
    }
}
fn contains(tree: &Tree, start: usize, end: usize, role: Role, name: &[u8]) -> bool {
    let Some(mut face) = TreeFace::subtree(tree, tree.nodes()[start].id) else {
        return false;
    };
    match face.find(&Find::named(role, name)) {
        Found::Node(id) => {
            let Some(at) = tree.nodes().iter().position(|n| n.id == id) else {
                return false;
            };
            at < end
                && !tree.nodes()[start + 1..=at].iter().enumerate().any(|(offset, n)| {
                    n.key.is_some_and(card_key) && start + 1 + offset + usize::try_from(n.size).expect("subtree") > at
                })
        }
        Found::Absent | Found::Pending => false,
    }
}
fn explained(tree: &Tree, start: usize, end: usize) -> bool {
    let Some(mut face) = TreeFace::subtree(tree, tree.nodes()[start].id) else {
        return false;
    };
    let Found::Node(id) = face.find(&Find::named(Role::Status, b"Leaving")) else {
        return false;
    };
    let Some(at) = tree.nodes().iter().position(|n| n.id == id) else {
        return false;
    };
    let through = at + usize::try_from(tree.nodes()[at].size).expect("status subtree");
    through <= end && tree.nodes()[at..through].iter().any(|n| !n.text.is_empty())
}
fn card_key(k: Key) -> bool {
    match k.owner {
        Owner::Jig => k.kind <= 6,
        Owner::Side { connector } => connector == domain::FIRST && (10..=16).contains(&k.kind),
        Owner::Other { .. } => false,
    }
}
pub fn card(tree: &Tree, node: NodeId) -> Option<CardKey> {
    let at = tree.nodes().iter().position(|n| n.id == node)?;
    tree.nodes()
        .iter()
        .enumerate()
        .take(at + 1)
        .filter(|(i, n)| *i + usize::try_from(n.size).expect("subtree") > at)
        .filter_map(|(_, n)| n.key.filter(|k| card_key(*k)))
        .next_back()
        .map(CardKey::of)
}
pub fn seen(tree: &Tree) -> Vec<Seen> {
    let mut seen = Vec::new();
    if [b"Sign in".as_slice(), b"Someone else's work", b"Page missing", b"Client out of date"].iter().any(|name| {
        tree.nodes().iter().any(|n| {
            n.role == Role::Heading { level: 1 }
                && TreeFace::subtree(tree, n.id)
                    .is_some_and(|mut face| matches!(face.find(&Find::text(name)), Found::Node(_)))
        })
    }) {
        seen.push(Seen::HiddenPage);
    }
    let offline = matches!(TreeFace::new(tree).find(&Find::named(Role::Status, b"Offline")), Found::Node(_));
    let live = matches!(TreeFace::new(tree).find(&Find::named(Role::Status, b"Live")), Found::Node(_));
    if offline || live {
        seen.push(Seen::Link { live });
    }
    for (at, node) in tree.nodes().iter().enumerate() {
        let Some(k) = node.key else {
            continue;
        };
        let end = at + usize::try_from(node.size).expect("subtree");
        if card_key(k) {
            let deciding = contains(tree, at, end, Role::Status, b"In flight")
                || contains(tree, at, end, Role::Status, b"Retrying")
                || contains(tree, at, end, Role::Status, b"Parked")
                || contains(tree, at, end, Role::Status, b"Being decided");
            let leaving = contains(tree, at, end, Role::Status, b"Leaving");
            let looking = contains(tree, at, end, Role::Status, b"Leaving")
                && contains(tree, at, end, Role::Text, b"Finding how this ended");
            let done = contains(tree, at, end, Role::Status, b"Touched")
                || contains(tree, at, end, Role::Status, b"Moved to revision")
                || contains(tree, at, end, Role::Status, b"Gone");
            let state = if leaving && !looking {
                CardSeen::Left { reason: explained(tree, at, end) }
            } else if deciding || looking {
                CardSeen::Deciding
            } else if done {
                CardSeen::Done
            } else {
                CardSeen::Waiting
            };
            seen.push(Seen::Card { key: CardKey::of(k), state, revision: None });
        }
        if k.owner == Owner::Jig
            && (k.kind == 25 || k.kind == 26)
            && !contains(tree, at, end, Role::Status, b"Being sent")
        {
            let status = tree.nodes()[at..end].iter().position(|n| {
                n.role == Role::Status
                    && (contains(tree, at, end, Role::Status, b"Sent")
                        || contains(tree, at, end, Role::Status, b"Read"))
            });
            if let Some(status) = status
                && let Some(text) = tree.nodes()[at..at + status].iter().rev().find(|n| !n.text.is_empty())
            {
                seen.push(Seen::Words {
                    task: k.a,
                    message: (k.kind == 25).then_some(k.b),
                    words: text.text.iter().flat_map(|r| r.text.iter().copied()).collect(),
                });
            }
        }
    }
    seen
}
