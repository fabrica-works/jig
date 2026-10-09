//! Acts decoded by the view, owned by this domain; client/domain/client.md, 4.1.
//! Jig 21 adds object act parameters and confirmations; jig 22 adds page filters.
use crate::{
    address::Address,
    contract::{ActKind, InboxGroup, Text},
    fields::Purpose,
};
use alloc::boxed::Box;
use skein_lib::Token;
/// A person action already decoded from the semantic tree.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Act {
    /// Go to a jig page.
    Go { address: Address },
    /// Edit a field.
    Edit { field: Purpose, text: Text },
    /// Submit a composer or form.
    Submit { field: Purpose },
    /// Open a confirmation for an object act.
    Intend { object: Token, act: ActKind },
    /// Choose an option in the confirmation.
    Choose { option: u16 },
    /// Answer that a person task cannot be done.
    CannotDo,
    /// Choose whether to repeat an uncertain effect on release.
    Again { again: bool },
    /// Choose a live delegate's disposal when closing a chat.
    Dispose { task: u64, take_over: bool },
    /// Supply the task amendment in the open confirmation.
    Change { change: Box<crate::contract::Amendment> },
    /// Write words and release a task stopped by this party in the same request.
    WriteRelease { release: bool },
    /// Set the one goal priority in the open confirmation.
    Priority { priority: u32 },
    /// Start a new chat naming a card by its committed title.
    ChatAbout { object: Token },
    /// Read an object again after it moved or an answer became unknown.
    LookAgain { object: Token },
    /// Confirm the open act.
    Confirm,
    /// Dismiss the confirmation or notice.
    Dismiss,
    /// Open or fold an object.
    Fold { object: Token, open: bool },
    /// Open or fold the details of a committed turn call.
    FoldCall { name: crate::conversation::CallName, open: bool },
    /// Filter an inbox group.
    Filter { group: InboxGroup, project: Option<u32> },
    /// Read more at a window end.
    More { window: Token, end: End },
    /// Set the inbox's project, kind and goal filters.
    FilterInbox { filter: crate::pages::inbox::Filter },
    /// Open updates and mark the newest result shown as seen.
    OpenUpdates,
    /// Start a chat in the selected project from its first-words field.
    StartChat { project: u32 },
    /// Open a listed chat and mark its replies through what was displayed.
    OpenChat { task: u64 },
    /// Choose the new goal charter, budget, deadline and priority.
    GoalForm { form: crate::pages::project::goals::GoalForm },
    /// Filter the project live task source.
    FilterTasks { filter: crate::contract::TaskFilter },
    /// Select note scope and description search.
    FilterNotes { filter: crate::pages::project::NoteFilter },
    /// Select held or live plan rows.
    FilterPlan { filter: crate::pages::task::PlanFilter },
    /// Read the task history alone or with its subtree.
    HistoryScope { subtree: bool },
    /// Read an ended subtree of the shown plan.
    OpenEnded { task: u64 },
    /// Choose a sign in provider.
    SignIn { provider: u16 },
}

/// The end of a window requested by a person.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum End {
    /// Earlier rows.
    Before,
    /// Later rows.
    After,
}
