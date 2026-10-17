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
