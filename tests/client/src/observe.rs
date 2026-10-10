//! Shared jig observations and the Test application's family vocabulary.
#![allow(clippy::wildcard_imports, reason = "boundary adapter covers the owner vocabulary")]
use crate::harness::{CardKey, CardSeen, Receipt, Seen, Sent};
use jig_client::contract::*;
use jig_client_view::{Key, NodeId, Owner, Role, Tree};
use jig_fake_person::{Face, Find, Found, TreeFace};
use jig_test_client_domain::{self as domain, Event, Request, family};
use skein_lib::RequestEnvelope;

#[must_use]
pub fn key(object: ObjectKey) -> CardKey {
    let (kind, a, b) = match object {
        ObjectKey::Task(task) => (0, task, 0),
        ObjectKey::Proposal(p) => match p.proposer {
            Proposer::Task(task) => (1, task, p.number),
            Proposer::Party(party) => (29, party, p.number),
        },
        ObjectKey::Escalation(task) => (2, task, 0),
        ObjectKey::Question { task, message } => (3, task, message),
        ObjectKey::PersonTask(task) => (4, task, 0),
        ObjectKey::Result(task) => (5, task, 0),
        ObjectKey::Note { project, note } => (6, u64::from(project), note),
    };
    CardKey::of(Key { owner: Owner::Jig, kind, a, b })
}
#[must_use]
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
        domain::Ask::Jig(ask) => return Some(jig_sent(*attempt, *k, ask)),
    };
    Some(Sent { attempt: *attempt, key: *k, card, revision, words })
}
/// Observe a keyed jig send without consulting an application's root state.
#[must_use]
pub fn jig_sent(attempt: skein_lib::Token, request_key: [u8; 16], ask: &Ask) -> Sent {
    let (card, revision, words) = match ask {
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
        Ask::EditNote { project, note, revision, .. } | Ask::DeleteNote { project, note, revision, .. } => {
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
        | Ask::ChangePolicy { .. }
        | Ask::SetRoles { .. }
        | Ask::MakeService { .. }
        | Ask::SetPool { .. } => (None, None, None),
    };
    Sent { attempt, key: request_key, card, revision, words }
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
#[must_use]
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
        Event::Jig(event) => jig_received(event),
        Event::First(
            domain::first::Event::Read { result: family::ReadResult::Value(doc), .. }
            | domain::first::Event::Watch { event: family::WatchEvent::Delivered(doc), .. },
        ) => vec![family(doc)],
        Event::SignedOut
        | Event::First(
            domain::first::Event::Read { result: family::ReadResult::SignedOut, .. }
            | domain::first::Event::Ended { end: family::WatchEnd::SignedOut, .. },
        ) => vec![Receipt::Parked],
        Event::Start { .. } | Event::Went { .. } | Event::Act { .. } | Event::First(_) => vec![],
    }
}
/// Observe delivered jig documents and sign-in state before the root callback.
#[must_use]
pub fn jig_received(event: &jig_client::Event) -> Vec<Receipt> {
    match event {
        jig_client::Event::Read { result: ReadResult::Value(value), .. } => read(value),
        jig_client::Event::Watch { event: WatchEvent::Delivered(Delivery::Task(value)), .. } => vec![task(value)],
        jig_client::Event::Watch { event: WatchEvent::Delivered(Delivery::Frame(value)), .. } => frame(value),
        jig_client::Event::SignedOut
        | jig_client::Event::Read { result: ReadResult::None(ReadNone::SignedOut), .. }
        | jig_client::Event::Ended { end: WatchEnd::SignedOut, .. } => vec![Receipt::Parked],
        jig_client::Event::Start { .. }
        | jig_client::Event::Kept { .. }
        | jig_client::Event::KeptGone { .. }
        | jig_client::Event::Went { .. }
        | jig_client::Event::Act { .. }
        | jig_client::Event::Ask { .. }
        | jig_client::Event::Progress { .. }
        | jig_client::Event::Answered { .. }
        | jig_client::Event::Read { .. }
        | jig_client::Event::Watch { .. }
        | jig_client::Event::Ended { .. }
        | jig_client::Event::ShowObjects { .. }
        | jig_client::Event::ObjectsLeft
        | jig_client::Event::Frame { .. } => vec![],
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
                    (n.key.is_some_and(card_key)
                        || (n.role == Role::Item && n.key.is_some_and(|key| matches!(key.owner, Owner::Side { .. }))))
                        && start + 1 + offset + usize::try_from(n.size).expect("subtree") > at
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
        Owner::Jig => k.kind <= 6 || k.kind == 29,
        Owner::Side { connector } => connector == domain::FIRST && ((10..=16).contains(&k.kind) || k.kind == 19),
        Owner::Other { .. } => false,
    }
}
#[must_use]
pub fn card(tree: &Tree, node: NodeId) -> Option<CardKey> {
    enclosing_card(tree, node, card_key)
}
/// Find a jig card using only its stable public key and tree ancestry.
#[must_use]
pub fn jig_card(tree: &Tree, node: NodeId) -> Option<CardKey> {
    enclosing_card(tree, node, |key| key.owner == Owner::Jig && (key.kind <= 6 || key.kind == 29))
}
fn enclosing_card(tree: &Tree, node: NodeId, admits: fn(Key) -> bool) -> Option<CardKey> {
    let at = tree.nodes().iter().position(|n| n.id == node)?;
    tree.nodes()
        .iter()
        .enumerate()
        .take(at + 1)
        .filter(|(i, n)| *i + usize::try_from(n.size).expect("subtree") > at)
        .filter_map(|(_, n)| n.key.filter(|k| admits(*k)))
        .next_back()
        .map(CardKey::of)
}
#[must_use]
pub fn seen(tree: &Tree) -> Vec<Seen> {
    visible(tree, true)
}
/// Observe jig cards, delivered words and shell state in a composed public tree.
#[must_use]
pub fn jig_seen(tree: &Tree) -> Vec<Seen> {
    visible(tree, false)
}
fn visible(tree: &Tree, include_test_family: bool) -> Vec<Seen> {
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
        // Plan tree items reuse object identity, but only Item nodes are cards.
        if (k.owner == Owner::Jig && (k.kind <= 6 || k.kind == 29) && node.role == Role::Item)
            || (include_test_family && k.owner != Owner::Jig && card_key(k))
        {
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

/// Only decoded public person acts explain changing the displayed window (QUESTIONS 34).
#[must_use]
pub fn window_changed(event: &Event) -> bool {
    let act = match event {
        Event::Act { action: domain::Action::Jig(act) } | Event::Jig(jig_client::Event::Act { act }) => act,
        Event::Start { .. }
        | Event::Went { .. }
        | Event::Act { action: domain::Action::First(_) }
        | Event::Answer { .. }
        | Event::Jig(_)
        | Event::First(_)
        | Event::SignedOut => return false,
    };
    jig_window_changed(act)
}
/// Only decoded jig person acts explain a changed displayed window (QUESTIONS 34).
#[must_use]
pub fn jig_window_changed(act: &jig_client::action::Act) -> bool {
    matches!(
        act,
        jig_client::action::Act::Go { .. }
            | jig_client::action::Act::Fold { .. }
            | jig_client::action::Act::FoldCall { .. }
            | jig_client::action::Act::Filter { .. }
            | jig_client::action::Act::More { .. }
            | jig_client::action::Act::FilterInbox { .. }
            | jig_client::action::Act::FilterTasks { .. }
            | jig_client::action::Act::FilterNotes { .. }
            | jig_client::action::Act::FilterPlan { .. }
            | jig_client::action::Act::HistoryScope { .. }
            | jig_client::action::Act::OpenEnded { .. }
            | jig_client::action::Act::OpenUpdates
    )
}

#[cfg(test)]
mod tests {
    use super::{CardKey, CardSeen, Key, Owner, Role, Seen, jig_seen};
    use jig_client_view::View;
    use skein_lib::Queue;

    #[test]
    fn a_nested_application_status_cannot_decide_a_jig_card() {
        let limits = jig_client_view::Limits {
            nodes: 16,
            depth: 4,
            text_per_node: 64,
            text_bytes: 1024,
            drawing_bytes: 0,
            styles_per_node: 0,
            patches: 128,
            bindings: 0,
            slots: 0,
            markdown_depth: 1,
        };
        let mut view = View::new(&limits);
        let mut builder = view.builder();
        let card = Key { owner: Owner::Jig, kind: 0, a: 9, b: 0 };
        builder.open(Role::Page);
        builder.open(Role::Item);
        builder.key(card);
        builder.open(Role::Item);
        builder.key(Key { owner: Owner::Side { connector: 77 }, kind: 2, a: 1, b: 5 });
        builder.open(Role::Status);
        builder.name(b"Gone");
        builder.close();
        builder.close();
        builder.close();
        builder.close();
        view.finish(builder, 1, &mut Queue::with_capacity(limits.patches));
        let seen = jig_seen(view.tree());
        assert!(
            matches!(seen.as_slice(), [Seen::Card { key, state: CardSeen::Waiting, revision: None }] if *key == CardKey::of(card))
        );
    }
    #[test]
    fn public_observations_keep_equal_task_and_party_proposal_cards_and_requests_distinct() {
        use jig_client::contract::{Ask, Decision, ObjectKey, ProposalKey, Proposer};
        let limits = jig_client_view::Limits {
            nodes: 16,
            depth: 4,
            text_per_node: 64,
            text_bytes: 1024,
            drawing_bytes: 0,
            styles_per_node: 0,
            patches: 128,
            bindings: 0,
            slots: 0,
            markdown_depth: 1,
        };
        let mut view = View::new(&limits);
        let mut builder = view.builder();
        builder.open(Role::Page);
        let mut expected = Vec::new();
        for proposer in [Proposer::Task(u64::MAX), Proposer::Party(u64::MAX)] {
            let proposal = ProposalKey { proposer, number: u64::MAX };
            let card = super::key(ObjectKey::Proposal(proposal));
            let kind = match proposer {
                Proposer::Task(_) => 1,
                Proposer::Party(_) => 29,
            };
            assert_eq!(card, CardKey::of(Key { owner: Owner::Jig, kind, a: u64::MAX, b: u64::MAX }));
            let sent = super::jig_sent(
                skein_lib::Token::new(1),
                [7; 16],
                &Ask::DecideProposal { project: 4, proposal, decision: Decision::Accepted },
            );
            assert_eq!(sent.card, Some(card.clone()));
            builder.open(Role::Item);
            builder.key(Key { owner: Owner::Jig, kind, a: u64::MAX, b: u64::MAX });
            builder.text(jig_client_view::Run::Plain, b"supplied proposal");
            builder.close();
            expected.push(Seen::Card { key: card, state: CardSeen::Waiting, revision: None });
        }
        builder.close();
        view.finish(builder, 1, &mut Queue::with_capacity(limits.patches));
        assert_ne!(
            super::key(ObjectKey::Proposal(ProposalKey { proposer: Proposer::Task(u64::MAX), number: u64::MAX })),
            super::key(ObjectKey::Proposal(ProposalKey { proposer: Proposer::Party(u64::MAX), number: u64::MAX }))
        );
        assert_eq!(jig_seen(view.tree()), expected);
        for node in view.tree().nodes().iter().filter(|node| node.role == Role::Item) {
            assert_eq!(super::jig_card(view.tree(), node.id), node.key.map(CardKey::of));
        }
    }
}
