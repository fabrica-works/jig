use super::Borrowed;
use crate::page::task::{Plan, PlanFilter, TaskPage};
use jig_client::{
    Latest,
    contract::{Delivery, OwnerTree, Phase, PhaseCounts, TreeNode},
};
#[derive(Debug)]
pub struct Task<'a> {
    pub state: &'a TaskPage,
    pub plan: PlanView<'a>,
}
impl core::ops::Deref for Task<'_> {
    type Target = TaskPage;
    fn deref(&self) -> &TaskPage {
        self.state
    }
}
#[derive(Debug)]
pub struct PlanView<'a> {
    pub state: &'a Plan,
    pub owner: Option<&'a OwnerTree>,
    pub nodes: &'a [TreeNode],
    pub folded: PhaseCounts,
    pub held_below: u32,
}
impl core::ops::Deref for PlanView<'_> {
    type Target = Plan;
    fn deref(&self) -> &Plan {
        self.state
    }
}
impl PlanView<'_> {
    #[must_use]
    pub fn owner_visible(&self, row: &jig_client::contract::OwnerTreeRow) -> bool {
        match self.filter {
            PlanFilter::All => true,
            PlanFilter::Live => crate::page::conversations::live(row.chip.phase),
            PlanFilter::Held => {
                let mut held = row.chip.phase == Phase::Held;
                if let Some(owner) = self.owner {
                    for fold in &owner.folded {
                        held |= fold.parent == row.chip.task && fold.held_below > 0;
                    }
                }
                held
            }
        }
    }
}
impl<'a> Borrowed<'a> {
    #[must_use]
    pub fn task_page(&self) -> Task<'a> {
        let mut plan = PlanView {
            state: &self.ui.task_page.plan,
            owner: None,
            nodes: &[],
            folded: crate::page::task::counts(),
            held_below: 0,
        };
        for (handle, _) in &self.ui.handles.watches {
            let Some(document) = self.client.document(*handle) else {
                continue;
            };
            match document.latest {
                Some(Latest::Jig(Delivery::OwnerTree(tree))) if Some(tree.root.task) == self.ui.shown_task() => {
                    plan.owner = Some(tree);
                }
                Some(Latest::Jig(Delivery::Tree(tree))) => {
                    plan.folded = tree.folded;
                    plan.held_below = tree.held_below;
                    let count = usize::try_from(self.ui.limits.rows)
                        .expect("row bound fits usize")
                        .min(tree.nodes.as_slice().len());
                    plan.nodes = tree.nodes.as_slice().get(..count).expect("visible rows fit the delivered slice");
                    for row in tree.nodes.as_slice().get(count..).expect("folded rows follow visible rows") {
                        add(&mut plan.folded, row.chip.phase);
                    }
                }
                Some(Latest::Jig(_) | Latest::Family(_)) | None => {}
            }
        }
        Task { state: &self.ui.task_page, plan }
    }
}

fn add(counts: &mut PhaseCounts, phase: Phase) {
    let value = match phase {
        Phase::Waiting | Phase::Idle => &mut counts.waiting,
        Phase::Due | Phase::Preparing | Phase::Claimed | Phase::Running | Phase::BackingOff | Phase::Active => {
            &mut counts.running
        }
        Phase::Held => &mut counts.held,
        Phase::Closing => &mut counts.closing,
        Phase::Ended => &mut counts.ended,
        Phase::Done => &mut counts.done,
        Phase::Failed => &mut counts.failed,
        Phase::Cancelled => &mut counts.cancelled,
    };
    *value = value.checked_add(1).expect("a bounded tree count fits u32");
}
