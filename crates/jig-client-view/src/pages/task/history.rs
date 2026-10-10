//! Historical pages retain before/after facts and opaque edges (pages.md, 5.2).
use crate::{
    Builder, JigBindings, Role, Run,
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};
use jig_client::{
    Domain,
    action::Act,
    contract::{HistoryChange, HistoryRow},
};
pub(super) fn history(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings) {
    for (subtree, phrase) in [(false, JigPhrase::OwnHistory), (true, JigPhrase::SubtreeHistory)] {
        super::super::work::control(
            b,
            words,
            bindings,
            phrase,
            Act::HistoryScope { subtree },
            client.task_page().subtree_history == subtree,
        );
    }
    b.open(Role::List);
    for row in &client.history_rows().rows {
        b.open(Role::Item);
        match row {
            HistoryRow::Legacy(row) => legacy(b, row, words, bindings),
            HistoryRow::Change(row) => {
                b.key(crate::Key { owner: crate::Owner::Jig, kind: 41, a: row.task, b: row.revision });
                task_link(b, row.task, words, bindings);
                crate::owner_history::render(b, row, words, bindings);
            }
            HistoryRow::Milestone(row) => {
                b.key(crate::Key { owner: crate::Owner::Jig, kind: 42, a: row.task, b: row.position });
                task_link(b, row.task, words, bindings);
                crate::owner_milestone::render(b, row, client, words, bindings);
            }
        }
        b.close();
    }
    b.close();
    super::super::cut(b, words, client.history_rows().cut);
    super::super::edges(b, words, bindings, &client.history_rows().edges, jig_client::pages::task::HISTORY_WINDOW);
}

fn task_link(b: &mut Builder, task: u64, words: &Words, bindings: &mut JigBindings) {
    crate::frame::control(
        b,
        words,
        bindings,
        Role::Link,
        JigPhrase::TaskPage,
        &[Hole::Number(task)],
        Act::Go { address: jig_client::address::Address::task(task) },
    );
}
fn legacy(b: &mut Builder, row: &jig_client::contract::HistoryLine, words: &Words, bindings: &mut JigBindings) {
    b.key(crate::Key { owner: crate::Owner::Jig, kind: 21, a: row.task, b: row.position });
    crate::frame::control(
        b,
        words,
        bindings,
        Role::Link,
        JigPhrase::TaskPage,
        &[Hole::Number(row.task)],
        Act::Go { address: jig_client::address::Address::task(row.task) },
    );
    words.actor(b, row.by);
    words.wall(b, row.at);
    match &row.change {
        HistoryChange::Made { batch } => words.text(b, JigPhrase::BatchMade, &[Hole::Number(*batch)]),
        HistoryChange::Amended { before, after } => {
            words.text(b, JigPhrase::Before, &[]);
            crate::markdown::read(before.bytes(), 2, b);
            words.text(b, JigPhrase::After, &[]);
            crate::markdown::read(after.bytes(), 2, b);
        }
        HistoryChange::Held(hold) => crate::cards::hold::reason(b, &hold.reason, words, bindings),
        HistoryChange::Released => words.text(b, JigPhrase::Released, &[]),
        HistoryChange::Cancelled { reason } => {
            words.text(b, JigPhrase::Cancelled, &[crate::words::name(reason.bytes())]);
        }
        HistoryChange::Moved { from, to, reason } => {
            words.text(b, JigPhrase::MovedWork, &[]);
            words.text(b, JigPhrase::Before, &[]);
            words.actor(b, *from);
            words.text(b, JigPhrase::After, &[]);
            words.actor(b, *to);
            b.text(Run::Plain, reason.bytes());
        }
    }
}
