//! An escalation preserves its task owner’s whole document and its independent
//! offered decisions (client/domain/contract.md, 5.1–5.3; objects.md, 5).
use super::{Offered, Task};
use alloc::boxed::Box;
use skein_lib::List;

/// A held task and the decisions offered on its escalation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerEscalation {
    pub acts: List<Offered>,
    pub task: Box<Task>,
}
