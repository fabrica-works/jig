//! Inbox groups reuse the same cards and their context (client/domain/pages.md, 3).
use crate::{
    Builder, JigBindings, Role, Run, cards,
    phrases::{Hole, jig::JigPhrase},
    words::Words,
};
use jig_client::{
    Domain,
    action::Act,
    action::End,
    contract::{InboxGroup, InboxReadGroup, Recipient},
    pages::inbox::{Filter, needs},
};
/// Render the bounded, already ordered inbox, filters and both window edges.
pub fn inbox(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings) {
    super::heading(b, words, JigPhrase::Inbox, &[]);
    filters(b, client, words, bindings);
    let page = client.inbox();
    let count = u64::from(page.needs.questions)
        .checked_add(u64::from(page.needs.proposals))
        .expect("inbox count fits")
        .checked_add(u64::from(page.needs.escalations))
        .expect("inbox count fits")
        .checked_add(u64::from(page.needs.person_tasks))
        .expect("inbox count fits");
    words.text(b, JigPhrase::Needs, &[Hole::Number(count)]);
    words.text(
        b,
        if page.updates.limited { JigPhrase::MoreUpdates } else { JigPhrase::Updates },
        &[Hole::Number(u64::from(page.updates.value))],
    );
    for group in [true, false] {
        b.open(Role::Region);
        let title = words.say(if group { JigPhrase::NeedsYou } else { JigPhrase::InboxUpdates }, &[]);
        b.name(&title);
        b.open(Role::Heading { level: 2 });
        b.text(Run::Plain, &title);
        b.close();
        b.open(Role::List);
        for entry in &page.rows {
            if needs(entry.group) == group {
                b.open(Role::Item);
                b.key(super::object_key(entry.object));
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
                if let Some(id) = client.objects().find(entry.object) {
                    cards::card(
                        b,
                        client,
                        id,
                        &cards::Place::Inbox { lineage: entry.lineage.clone() },
                        words,
                        bindings,
                    );
                }
                b.close();
            }
        }
        b.close();
        let parent = if group { InboxReadGroup::Needs } else { InboxReadGroup::Updates };
        window(b, client, words, bindings, parent);
        b.close();
    }
    if page.live_more {
        words.status(b, JigPhrase::MoreRows, &[]);
    }
    if let Some(refusal) = &page.refusal {
        words.refusal(b, refusal);
    }
    super::excess(b, client, words);
}
fn window(b: &mut Builder, client: &Domain, words: &Words, bindings: &mut JigBindings, parent: InboxReadGroup) {
    let page = client.inbox();
    if let Some(kind) = page.filter.kind
        && InboxReadGroup::of(kind) != parent
    {
        return;
    }
    let window = page.window(parent);
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
                        InboxReadGroup::Needs => jig_client::pages::conversations::INBOX_WINDOW,
                        InboxReadGroup::Updates => jig_client::pages::conversations::INBOX_UPDATES_WINDOW,
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
        if let Some(goal) = entry.goal {
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

fn unread(entry: &jig_client::contract::InboxEntry, results_seen: u64) -> bool {
    match entry.group {
        InboxGroup::Results => entry.position > results_seen,
        InboxGroup::Replies => match &entry.reply {
            Some(line) => line.last_reply > line.seen,
            None => false,
        },
        InboxGroup::Questions | InboxGroup::Proposals | InboxGroup::Escalations | InboxGroup::PersonTasks => false,
    }
}
