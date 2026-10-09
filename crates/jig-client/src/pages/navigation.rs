//! Application-chosen navigation and home, kept within a fixed entry count.
//! Entries keep only addresses and opaque application numbers; application page
//! contents belong to their owner. The latest frame decides availability, without
//! interpreting roles. See client/domain/pages.md, 2, and root.md, 5–6.
use crate::{
    address::{Address, Place},
    contract::{Frame, PartyKind},
};
use skein_lib::List;

/// One navigation destination, translated by the application's root.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Destination {
    /// A page whose address belongs to jig.
    Jig(Address),
    /// An application's own destination, identified by its owner and local number.
    Application { owner: u16, number: u32 },
}
impl Destination {
    /// Which child owns this destination; the root retains its application number.
    #[must_use]
    pub const fn place(self) -> Place {
        match self {
            Self::Jig(address) => Place::Jig(address),
            Self::Application { .. } => Place::Elsewhere,
        }
    }
}

/// The configured home and ordered navigation entries.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Navigation {
    pub home: Destination,
    pub entries: List<Destination>,
}
impl Navigation {
    /// The inbox is home unless the application chooses another destination.
    #[must_use]
    pub fn new(capacity: u32) -> Self {
        Self { home: Destination::Jig(Address::Inbox { project: None }), entries: List::with_capacity(capacity) }
    }
    /// Whether a destination applies to the party's current committed frame.
    #[must_use]
    pub fn offered(&self, destination: Destination, frame: Option<&Frame>) -> bool {
        let Some(frame) = frame else {
            return false;
        };
        match frame.party.kind {
            PartyKind::Service => return false,
            PartyKind::Person => {}
        }
        match destination {
            Destination::Application { .. } => true,
            Destination::Jig(address) => match address {
                Address::Inbox { .. } | Address::Task { .. } => true,
                Address::Conversations { project } => {
                    for offered in &frame.projects {
                        if !offered.chats.is_empty() && (project.is_none() || project == Some(offered.project)) {
                            return true;
                        }
                    }
                    false
                }
                Address::Project { project, .. } => {
                    for offered in &frame.projects {
                        if offered.project == project {
                            return true;
                        }
                    }
                    false
                }
                Address::Deployment => frame.deployment_role.is_some(),
            },
        }
    }
}
