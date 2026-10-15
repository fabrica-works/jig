//! Inbox groups reuse the same cards and their context (client/domain/pages.md, 3).
use crate::ui::{Builder, Role, Run};
use crate::{
    JigBindings, cards,
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};

use jig_domain_client::{
    Domain,
    action::Act,
    action::End,
    contract::{
        InboxDocument, InboxGroup, InboxReadGroup, InboxRow, OwnerInboxEntry, OwnerInboxObject, Proposer, Recipient,
    },
    pages::inbox::{Filter, needs},
};
/// The inbox's filters and counts while either group is shown.
pub fn header(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings) -> bool {
    let Some(project) = client.shown_inbox() else {
        return false;
    };
    if !client.shows(jig_domain_client::pages::Section::Inbox { group: InboxReadGroup::Needs, project })
        && !client.shows(jig_domain_client::pages::Section::Inbox { group: InboxReadGroup::Updates, project })
    {
        return false;
    }
    b.open(Role::Region);
    b.region(crate::regions::JigRegion::InboxHeader.region());
    b.name(&words.say(JigPhrase::Inbox, &[]));
    filters(b, client, words, bindings);
    let page = client.inbox();
    let (count, updates) = match &page.document {
        Some(InboxDocument::Owner(value)) => (u64::from(value.cached_needs), value.cached_updates),
        Some(InboxDocument::Entry(_)) | None => {
            let count = u64::from(page.needs.questions)
                .checked_add(u64::from(page.needs.proposals))
                .expect("inbox count fits")
                .checked_add(u64::from(page.needs.escalations))
                .expect("inbox count fits")
                .checked_add(u64::from(page.needs.person_tasks))
                .expect("inbox count fits");
            (count, page.updates)
        }
    };
    words.text(b, JigPhrase::Needs, &[Hole::Number(count)]);
    words.text(
        b,
        if updates.limited { JigPhrase::MoreUpdates } else { JigPhrase::Updates },
        &[Hole::Number(u64::from(updates.value))],
    );
    b.close();
    true
}
pub(crate) fn group(
    b: &mut Builder,
    client: &Domain,
    words: &Words,
    bindings: &mut JigBindings,
    parent: InboxReadGroup,
    project: Option<u32>,
) -> bool {
    if !client.shows(jig_domain_client::pages::Section::Inbox { group: parent, project }) {
        return false;
    }
    let group = parent == InboxReadGroup::Needs;
    crate::pages::work::section(
        b,
        words,
        if group { JigPhrase::NeedsYou } else { JigPhrase::InboxUpdates },
        if group { crate::regions::JigRegion::InboxNeeds } else { crate::regions::JigRegion::InboxUpdates },
    );
    let page = client.inbox();
    super::status(b, client, words, jig_domain_client::pages::Section::Inbox { group: parent, project });
    b.open(Role::List);
    for entry in &page.rows {
        if needs(entry.group()) == group {
            b.open(Role::Item);
            match entry {
                InboxRow::Entry(entry) => supplied_entry(b, client, words, bindings, entry, group),
                InboxRow::Owner(entry) => owner_entry(b, client, words, bindings, entry, group),
            }
            b.close();
        }
    }
    b.close();

    window(b, client, words, bindings, parent);
    b.close();
    true
}
pub(crate) fn status(b: &mut Builder, client: &Domain, words: &Words) {
    let page = client.inbox();
    if page.goal_context_pending {
        owner_text(b, words, crate::owner_inbox_words::InboxPhrase::GoalContextPending, &[]);
    }
    if page.live_more {
        words.status(b, JigPhrase::MoreRows, &[]);
    }
    if let Some(refusal) = &page.refusal {
        words.refusal(b, refusal);
    }
}
fn window(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings, parent: InboxReadGroup) {
    let page = client.inbox();
    if let Some(kind) = page.filter.kind
        && InboxReadGroup::of(kind) != parent
    {
        return;
    }
    let window = page.window(parent);
    crate::pages::cut(b, words, window.cut);
    if window.reading.is_some() {
        words.status(b, JigPhrase::Looking, &[]);
    }
    if window.more {
        words.status(b, JigPhrase::MoreRows, &[]);
    }
    for end in [End::Before, End::After] {
        let more = match end {
            End::Before => window.before.is_some(),
            End::After => window.after.is_some() || window.more,
        };
        if more {
            crate::frame::control(
                b,
                words,
                bindings,
                Role::Button,
                match end {
                    End::Before => JigPhrase::ReadEarlier,
                    End::After => JigPhrase::ReadLater,
                },
                &[],
                Act::More {
                    window: skein_lib::Token::new(match parent {
                        InboxReadGroup::Needs => jig_domain_client::pages::conversations::INBOX_WINDOW,
                        InboxReadGroup::Updates => jig_domain_client::pages::conversations::INBOX_UPDATES_WINDOW,
                    }),
                    end,
                },
            );
        } else {
            words.text(
                b,
                match end {
                    End::Before => JigPhrase::WindowStart,
                    End::After => JigPhrase::WindowEnd,
                },
                &[],
            );
        }
    }
}
fn filter(
    b: &mut Builder,
    words: &Words,
    bindings: &mut JigBindings,
    phrase: JigPhrase,
    holes: &[Hole],
    value: Filter,
) {
    crate::frame::control(b, words, bindings, Role::Button, phrase, holes, Act::FilterInbox { filter: value });
}
fn filters(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings) {
    let current = client.inbox().filter;
    filter(b, words, bindings, JigPhrase::AllProjects, &[], Filter { project: None, ..current });
    if let Some(frame) = &client.frame().document {
        for project in &frame.projects {
            let (phrase, hole) = match &project.name {
                Some(value) => (JigPhrase::Project, crate::words::name(value.bytes())),
                None => (JigPhrase::FrameProjectNumber, Hole::Number(u64::from(project.project))),
            };
            filter(b, words, bindings, phrase, &[hole], Filter { project: Some(project.project), ..current });
            crate::frame_facts::project_loss(b, words, project);
        }
    }
    filter(b, words, bindings, JigPhrase::AllKinds, &[], Filter { kind: None, ..current });
    for kind in [
        InboxGroup::Questions,
        InboxGroup::Proposals,
        InboxGroup::Escalations,
        InboxGroup::PersonTasks,
        InboxGroup::Results,
        InboxGroup::Replies,
    ] {
        let phrase = match kind {
            InboxGroup::Questions => JigPhrase::Question,
            InboxGroup::Proposals => JigPhrase::Proposal,
            InboxGroup::Escalations => JigPhrase::Escalation,
            InboxGroup::PersonTasks => JigPhrase::PersonTask,
            InboxGroup::Results => JigPhrase::Result,
            InboxGroup::Replies => JigPhrase::Reply,
        };
        filter(b, words, bindings, phrase, &[], Filter { kind: Some(kind), ..current });
    }
    filter(b, words, bindings, JigPhrase::AllGoals, &[], Filter { goal: None, ..current });
    for entry in &client.inbox().rows {
        if let Some(goal) = entry.goal() {
            filter(
                b,
                words,
                bindings,
                JigPhrase::OnlyGoal,
                &[Hole::Number(goal)],
                Filter { goal: Some(goal), ..current },
            );
        }
    }
}

fn unread(entry: &jig_domain_client::contract::InboxEntry, results_seen: u64) -> bool {
    match entry.group {
        InboxGroup::Results => entry.position > results_seen,
        InboxGroup::Replies => match &entry.reply {
            Some(line) => line.last_reply > line.seen,
            None => false,
        },
        InboxGroup::Questions | InboxGroup::Proposals | InboxGroup::Escalations | InboxGroup::PersonTasks => false,
    }
}

fn supplied_entry(
    b: &mut Builder,
    client: &Domain,
    words: &Words,
    bindings: &mut JigBindings,
    entry: &jig_domain_client::contract::InboxEntry,
    group: bool,
) {
    b.key(crate::pages::object_key(entry.object));
    if group {
        words.waiting(b, client.wall(), entry.at);
    } else if unread(entry, client.inbox().results_seen) {
        words.text(b, JigPhrase::NewUpdate, &[]);
    }
    if let Some(at) = entry.stall {
        words.text(b, JigPhrase::PassesUp, &[Hole::Wall(words.adjusted_wall(at))]);
    }
    match entry.recipient {
        Some(Recipient::Role { .. } | Recipient::Roles { .. }) => words.text(b, JigPhrase::OnCall, &[]),
        Some(Recipient::Party(_) | Recipient::Task(_)) | None => {}
    }
    if let Some(id) = client.objects().find(entry.object)
        && !client.owes(id)
    {
        cards::card(b, client, id, &cards::Place::Inbox { lineage: entry.lineage.clone() }, words, bindings);
    }
}
fn owner_text(b: &mut Builder, words: &Words, phrase: crate::owner_inbox_words::InboxPhrase, holes: &[Hole]) {
    let saying = words.book.say(words.language, phrase.id(), holes).expect("whole Inbox words and typed holes checked");
    b.text(Run::Plain, &saying.text);
}
fn owner_entry(
    b: &mut Builder,
    client: &Domain,
    words: &Words,
    bindings: &mut JigBindings,
    entry: &OwnerInboxEntry,
    group: bool,
) {
    use crate::owner_inbox_words::InboxPhrase;
    let task = entry.task_number();
    let (kind, a, value) = match entry.object {
        OwnerInboxObject::Question { message } => (50, task, message),
        OwnerInboxObject::Proposal { proposer, number } => match proposer {
            Proposer::Task(task) => (51, task, number),
            Proposer::Party(party) => (52, party, number),
        },
        OwnerInboxObject::Escalation { revision } => (53, task, revision),
        OwnerInboxObject::PersonTask => (54, task, 0),
        OwnerInboxObject::Reply { message } => (55, task, message),
        OwnerInboxObject::Result { position } => (56, task, position),
    };
    b.key(crate::ui::Key { owner: crate::Owner::Jig.opaque(), kind, a, b: value });
    match entry.object {
        OwnerInboxObject::Question { message } => owner_text(b, words, InboxPhrase::Question, &[Hole::Number(message)]),
        OwnerInboxObject::Proposal { proposer, number } => match proposer {
            Proposer::Task(task) => {
                owner_text(b, words, InboxPhrase::TaskProposal, &[Hole::Number(task), Hole::Number(number)]);
            }
            Proposer::Party(party) => {
                owner_text(b, words, InboxPhrase::PartyProposal, &[Hole::Number(party), Hole::Number(number)]);
            }
        },
        OwnerInboxObject::Escalation { revision } => {
            owner_text(b, words, InboxPhrase::Escalation, &[Hole::Number(revision)]);
        }
        OwnerInboxObject::PersonTask => owner_text(b, words, InboxPhrase::PersonTask, &[]),
        OwnerInboxObject::Reply { message } => owner_text(b, words, InboxPhrase::Reply, &[Hole::Number(message)]),
        OwnerInboxObject::Result { position } => owner_text(b, words, InboxPhrase::Result, &[Hole::Number(position)]),
    }
    if let Some(project) = entry.project {
        owner_text(b, words, InboxPhrase::Project, &[Hole::Number(u64::from(project))]);
    }
    if group {
        if let Some(at) = entry.at {
            words.waiting(b, client.wall(), at);
        }
    } else if entry.unread == Some(true) {
        words.text(b, JigPhrase::NewUpdate, &[]);
    }
    if let Some(activity) = entry.activity {
        owner_text(b, words, InboxPhrase::Activity, &[Hole::Number(activity)]);
    }
    crate::owner_result::link(b, client, &entry.task, words, bindings);
    if let Some(id) = client.objects().find(entry.card_key())
        && !client.owes(id)
    {
        cards::card(b, client, id, &cards::Place::Elsewhere, words, bindings);
    }
}
