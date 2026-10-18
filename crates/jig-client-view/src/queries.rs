//! The reference UI's task lists (client/domain/pages.md, 12.5).
use jig_client::contract::{OwnerRequester, QueryOrder, TaskQuery};
/// The person's live chats, by their last activity.
#[must_use]
pub const fn conversations(party: u64, chat_label: u32, project: Option<u32>) -> TaskQuery {
    TaskQuery {
        project,
        requester: Some(OwnerRequester::Person(party)),
        root: None,
        parent: None,
        label: Some(chat_label),
        phase: None,
        executor: None,
        tracked: None,
        order: QueryOrder::Activity,
    }
}
pub(crate) fn chats(me: &jig_client::contract::Me, project: Option<u32>) -> Option<TaskQuery> {
    for held in &me.projects {
        let selected = match project {
            Some(project) => project == held.project,
            None => true,
        };
        if selected && let Some(template) = jig_client::shapes::template(me, held.project, jig_client::shapes::CHAT) {
            return Some(conversations(me.party.number, template.label, project));
        }
    }
    None
}

/// The project's live tracked tasks, in the owner's priority order.
#[must_use]
pub const fn goals(project: u32) -> TaskQuery {
    TaskQuery {
        project: Some(project),
        requester: None,
        root: None,
        parent: None,
        label: None,
        phase: None,
        executor: None,
        tracked: Some(true),
        order: QueryOrder::Priority,
    }
}
/// The project's offered goal template, shared by list and page guards.
pub(crate) fn goal_template(me: &jig_client::contract::Me, project: u32) -> Option<&jig_client::contract::Template> {
    jig_client::shapes::template(me, project, jig_client::shapes::GOAL)
}
/// The project's ended tasks of its goal template, latest first.
#[must_use]
pub const fn ended_goals(project: u32, goal_label: u32) -> TaskQuery {
    TaskQuery {
        label: Some(goal_label),
        phase: Some(jig_client::contract::PhaseClass::Ended),
        tracked: None,
        order: QueryOrder::Ended,
        ..goals(project)
    }
}
