//! The fixed task query and the single task line (contract.md, 5.1 and 6.1).
use super::{Chip, Count, OwnerBudget, OwnerPhaseCounts, OwnerRequester, TaskFlags};
use skein_lib::Wall;
/// A query's order; ties go by task number.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum QueryOrder {
    Number,
    Changed,
    Activity,
    Priority,
    Ended,
}
/// The selected phase class.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PhaseClass {
    Working,
    Waiting,
    Held,
    Closing,
    Ended,
}
/// The selected executor kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutorKind {
    Agent,
    Procedure,
    Person,
}
/// A conjunction of fixed keys with an owner-supplied order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TaskQuery {
    pub project: Option<u32>,
    pub requester: Option<OwnerRequester>,
    pub root: Option<u64>,
    pub parent: Option<u64>,
    pub label: Option<u32>,
    pub phase: Option<PhaseClass>,
    pub executor: Option<ExecutorKind>,
    pub tracked: Option<bool>,
    pub order: QueryOrder,
}
/// The party an entry waits for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Whom {
    Person(u64),
    Role { project: u32, role: u32 },
}
/// The kind of the oldest waiting entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WaitedFor {
    Question,
    Proposal,
    Escalation,
    PersonTask,
}
/// The task's wait, supplied by its owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WaitsOn {
    Party { whom: Whom, entries: u32, task: u64, kind: WaitedFor },
    Delegates,
    News,
    Nothing,
}
/// A conversation's read position and unread count.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ConversationRead {
    pub seen: u64,
    pub unread: Count,
}
/// One task's facts, shared by every list of tasks.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TaskLine {
    pub chip: Chip,
    pub project: u32,
    pub requester: OwnerRequester,
    pub tracked: Option<u32>,
    pub label: u32,
    pub flags: TaskFlags,
    pub changed: Wall,
    pub last_activity: Wall,
    pub ended_at: Option<Wall>,
    pub budget: OwnerBudget,
    pub waits_on: WaitsOn,
    pub phases: OwnerPhaseCounts,
    pub held_below: u32,
    pub conversation: Option<ConversationRead>,
}
