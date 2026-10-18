//! Jig's names, pictures and the browser parts' English phrases (protocol/browser.md, 3.3).
use jig_client_view::{Owner, regions::JigRegion};
use skein_ui::look::Names;

pub const NAMES: Names<'static> = Names {
    owner: Owner::Jig.opaque(),
    prefix: b"jig",
    styles: &[
        b"jig-card",
        b"jig-chip",
        b"jig-muted",
        b"jig-warning",
        b"jig-danger",
        b"jig-success",
        b"jig-button-weight",
        b"jig-dense-list",
    ],
    regions: &[
        JigRegion::Link.name(),
        JigRegion::Needs.name(),
        JigRegion::Party.name(),
        JigRegion::Projects.name(),
        JigRegion::GoToTask.name(),
        JigRegion::Navigation.name(),
        JigRegion::Owed.name(),
        JigRegion::Confirmation.name(),
        JigRegion::Notices.name(),
        JigRegion::InboxHeader.name(),
        JigRegion::InboxNeeds.name(),
        JigRegion::InboxUpdates.name(),
        JigRegion::Conversations.name(),
        JigRegion::TaskHeader.name(),
        JigRegion::TaskTabs.name(),
        JigRegion::TaskOverview.name(),
        JigRegion::TaskActivity.name(),
        JigRegion::TaskPlan.name(),
        JigRegion::TaskHistory.name(),
        JigRegion::TaskBudget.name(),
        JigRegion::TaskPanels.name(),
        JigRegion::ProjectHeader.name(),
        JigRegion::ProjectTabs.name(),
        JigRegion::ProjectOverview.name(),
        JigRegion::ProjectGoals.name(),
        JigRegion::ProjectTasks.name(),
        JigRegion::ProjectNotes.name(),
        JigRegion::ProjectSettings.name(),
        JigRegion::ProjectResources.name(),
        JigRegion::ProjectPolicy.name(),
        JigRegion::ProjectRoles.name(),
        JigRegion::ProjectSpend.name(),
        JigRegion::ProjectRecurring.name(),
        JigRegion::Deployment.name(),
        JigRegion::Conversation.name(),
        JigRegion::Composer.name(),
        JigRegion::NewChat.name(),
        JigRegion::NewGoal.name(),
        JigRegion::NotesSearch.name(),
        JigRegion::Hosts.name(),
        JigRegion::Accounts.name(),
        JigRegion::Connectors.name(),
        JigRegion::Capacity.name(),
        JigRegion::Starting.name(),
        JigRegion::SigningIn.name(),
        JigRegion::Missing.name(),
        JigRegion::OutOfDate.name(),
        JigRegion::SomeoneElse.name(),
    ],
};
pub const PICTURES: (skein_ui::Owner, &[&[u8]]) = (Owner::Jig.opaque(), &[b"jig.svg"]);
/// Borrowed words let an application translate the parts without retaining credentials.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Phrases<'a> {
    pub failed: &'a str,
    pub credential: &'a str,
    pub keep: &'a str,
    pub close: &'a str,
}
#[must_use]
pub const fn phrases() -> Phrases<'static> {
    Phrases {
        failed: "The application failed.",
        credential: "Service credential",
        keep: "Copy this credential now. It will not be shown again.",
        close: "Close",
    }
}
