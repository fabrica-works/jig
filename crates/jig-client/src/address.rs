//! Jig addresses as typed places, never platform address text.
//! Navigation is owned by the client; see client/domain/client.md, 5.
/// The section of a project at an address.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProjectSection {
    /// Project overview.
    Overview,
    /// Goal board.
    Goals,
    /// Project tasks.
    Tasks,
    /// Project notes.
    Notes,
    /// Adopted resources.
    Resources,
    /// Project policy.
    Policy,
    /// Party roles and services.
    Roles,
    /// Period spend.
    Spend,
    /// Recurring tasks.
    Recurring,
}

/// The section of a task at an address.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TaskSection {
    /// Task overview.
    Overview,
    /// Conversation or executor activity.
    Activity,
    /// Live and ended delegates.
    Plan,
    /// Task history.
    History,
    /// Budget and authority.
    Budget,
    /// Connectors with something to say about the task.
    Panels,
}

/// A page owned by jig.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Address {
    /// The inbox, optionally filtered.
    Inbox { project: Option<u32> },
    /// Conversations, optionally filtered.
    Conversations { project: Option<u32> },
    /// One task section.
    Task { task: u64, section: TaskSection },
    /// One project section.
    Project { project: u32, section: ProjectSection },
    /// Deployment capacity.
    Deployment,
}

/// Whether the root address belongs to jig.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Place {
    /// A jig page.
    Jig(Address),
    /// An application page.
    Elsewhere,
    /// No page at this place.
    Missing,
}

impl Address {
    /// Every task chip links by its never-reused task number.
    #[must_use]
    pub const fn task(task: u64) -> Self {
        Self::Task { task, section: TaskSection::Overview }
    }
}
