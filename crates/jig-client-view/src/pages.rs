//! Page composition over domain-owned ordering and windows (client/domain/pages.md).
//! Objects stay in their shared cards; connector values stay in owner slots.
use crate::phrases::{Hole, jig::JigPhrase};
use crate::{Builder, JigBindings, Role, Run, words::Words};
use jig_client::{
    Domain,
    action::{Act, End},
    windows::{Edge, Edges},
};
use skein_lib::Token;
mod conversations;
mod inbox;
mod not_the_work;
pub use conversations::conversations;
pub use inbox::inbox;
pub use not_the_work::not_the_work;

pub(super) fn heading(b: &mut Builder, words: &Words, phrase: JigPhrase, holes: &[Hole]) {
    b.open(Role::Heading { level: 1 });
    let title = words.say(phrase, holes);
    b.name(&title);
    b.text(Run::Plain, &title);
    b.close();
}
pub(super) fn edges(b: &mut Builder, words: &Words, bindings: &mut JigBindings, edges: &Edges, window: u64) {
    edge(b, words, bindings, &edges.before, window, End::Before);
    edge(b, words, bindings, &edges.after, window, End::After);
}
fn edge(b: &mut Builder, words: &Words, bindings: &mut JigBindings, edge: &Edge, window: u64, end: End) {
    match edge {
        Edge::End => words.text(
            b,
            match end {
                End::Before => JigPhrase::WindowStart,
                End::After => JigPhrase::WindowEnd,
            },
            &[],
        ),
        Edge::More(_) => crate::frame::control(
            b,
            words,
            bindings,
            Role::Button,
            match end {
                End::Before => JigPhrase::ReadEarlier,
                End::After => JigPhrase::ReadLater,
            },
            &[],
            Act::More { window: Token::new(window), end },
        ),
    }
}
pub(super) fn excess(b: &mut Builder, client: &Domain, words: &Words) {
    match client.page() {
        jig_client::pages::Page::Work { excess, refused, .. } => {
            if *excess > 0 {
                words.status(b, JigPhrase::MoreRows, &[]);
            }
            if let Some(which) = refused {
                words.refusal(b, &jig_client::contract::Refusal::Limit { which: *which });
            }
        }
        jig_client::pages::Page::Starting
        | jig_client::pages::Page::Elsewhere
        | jig_client::pages::Page::SigningIn
        | jig_client::pages::Page::Missing
        | jig_client::pages::Page::OutOfDate
        | jig_client::pages::Page::SomeoneElse => {}
    }
}

pub(super) fn object_key(object: jig_client::contract::ObjectKey) -> crate::Key {
    use jig_client::contract::ObjectKey;
    let (kind, a, b) = match object {
        ObjectKey::Task(task) => (0, task, 0),
        ObjectKey::Proposal(key) => (1, key.proposer, key.number),
        ObjectKey::Escalation(task) => (2, task, 0),
        ObjectKey::Question { task, message } => (3, task, message),
        ObjectKey::PersonTask(task) => (4, task, 0),
        ObjectKey::Result(task) => (5, task, 0),
        ObjectKey::Note { project, note } => (6, u64::from(project), note),
    };
    crate::Key { owner: crate::Owner::Jig, kind, a, b }
}
