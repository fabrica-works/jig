//! Faults are shell operations or engine-boundary faults, scheduled or drawn.
use super::{ClientId, FrontFault};
use jig_view::ui::NodeId;
/// One fault against a public boundary, without an application's object types.
#[derive(Clone, Debug)]
pub enum Fault {
    Front(FrontFault),
    DoublePress { client: ClientId, node: NodeId },
    Reload { client: ClientId },
    TwoTabs { first: ClientId, first_node: NodeId, second: ClientId, second_node: NodeId },
    StalePress { client: ClientId, node: NodeId },
    OtherParty { client: ClientId, party: u64 },
}
impl From<FrontFault> for Fault {
    fn from(fault: FrontFault) -> Self {
        Self::Front(fault)
    }
}
