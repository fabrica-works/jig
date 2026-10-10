//! Whole heterogeneous history rows (client/domain/objects.md, 11;
//! pages.md, 5.2). A revision and an activity position name different facts.
use super::{HistoryLine, OwnerHistoryLine, OwnerMilestoneLine};
use alloc::boxed::Box;

/// One supplied semantic change or lifecycle milestone, in source page order.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum HistoryRow {
    /// Previously supplied application history, including its independent position.
    Line(Box<HistoryLine>),
    /// An immutable semantic change named by its task and revision.
    Change(Box<OwnerHistoryLine>),
    /// A lifecycle transition named by its task and activity position.
    Milestone(Box<OwnerMilestoneLine>),
}
