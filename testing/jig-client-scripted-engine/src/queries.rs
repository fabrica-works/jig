//! The scripted owner's task query, in the same directions as the real owner.
use jig_client_domain::contract::{self as c, ExecutorKind, Phase, PhaseClass, QueryOrder, Task, TaskLine, TaskQuery};
use std::cmp::Ordering;

pub(crate) fn selects(query: &TaskQuery, task: &Task) -> bool {
    let phase = match task.chip.phase {
        Phase::Waiting => PhaseClass::Waiting,
        Phase::Held => PhaseClass::Held,
        Phase::Closing => PhaseClass::Closing,
        Phase::Ended | Phase::Done | Phase::Failed | Phase::Cancelled => PhaseClass::Ended,
        Phase::Idle
        | Phase::Due
        | Phase::Preparing
        | Phase::Claimed
        | Phase::Running
        | Phase::BackingOff
        | Phase::Active => PhaseClass::Working,
    };
    let executor = match task.chip.executor {
        c::Executor::Agent { .. } => ExecutorKind::Agent,
        c::Executor::Procedure { .. } => ExecutorKind::Procedure,
        c::Executor::Party { .. } | c::Executor::Role { .. } => ExecutorKind::Person,
    };
    query.project.is_none_or(|v| v == task.project)
        && query.requester.is_none_or(|v| v == task.requester)
        && query.root.is_none_or(|v| v == task.lineage.as_slice().first().copied().unwrap_or(task.chip.task))
        && query.parent.is_none_or(|v| Some(v) == task.lineage.as_slice().last().copied())
        && query.label.is_none_or(|v| v == task.label)
        && query.phase.map_or(phase != PhaseClass::Ended, |v| v == phase)
        && query.executor.is_none_or(|v| v == executor)
        && query.tracked.is_none_or(|v| v == task.priority.is_some())
}
pub(crate) fn line<'a>(
    task: &Task,
    subtree: impl Iterator<Item = &'a Task>,
    conversation: Option<c::ConversationRead>,
) -> TaskLine {
    let mut phases = c::OwnerPhaseCounts {
        waiting: 0,
        idle: 0,
        starting: 0,
        running: 0,
        backing_off: 0,
        closing: 0,
        held: 0,
        done: 0,
        failed: 0,
        cancelled: 0,
    };
    for member in subtree {
        match member.chip.phase {
            Phase::Waiting => phases.waiting += 1,
            Phase::Idle => phases.idle += 1,
            Phase::Due | Phase::Preparing | Phase::Claimed => phases.starting += 1,
            Phase::Running | Phase::Active => phases.running += 1,
            Phase::BackingOff => phases.backing_off += 1,
            Phase::Closing => phases.closing += 1,
            Phase::Held => phases.held += 1,
            Phase::Ended | Phase::Done | Phase::Failed | Phase::Cancelled => {}
        }
    }
    TaskLine {
        chip: task.chip.clone(),
        project: task.project,
        requester: task.requester,
        tracked: task.priority,
        label: task.label,
        flags: task.flags,
        changed: task.changed,
        last_activity: task.last_activity,
        ended_at: (!jig_client_view::page::conversations::live(task.chip.phase)).then_some(task.changed),
        budget: task.budget.clone(),
        waits_on: task.waits_on,
        held_below: phases.held - u32::from(task.chip.phase == Phase::Held),
        phases,
        conversation,
    }
}
pub(crate) fn order(rows: &mut [TaskLine], order: QueryOrder) {
    rows.sort_by(|a, b| {
        let ordering = match order {
            QueryOrder::Number => Ordering::Equal,
            QueryOrder::Ended => b.ended_at.cmp(&a.ended_at),
            QueryOrder::Changed => b.changed.cmp(&a.changed),
            QueryOrder::Activity => b.last_activity.cmp(&a.last_activity),
            QueryOrder::Priority => (a.tracked.is_none(), a.tracked).cmp(&(b.tracked.is_none(), b.tracked)),
        };
        ordering.then(a.chip.task.cmp(&b.chip.task))
    });
}
impl<A: crate::Application> crate::Engine<A> {
    pub(crate) fn task_lines(&self, party: u64, query: &TaskQuery) -> Vec<TaskLine> {
        use c::ReadValue;
        let mut rows = Vec::new();
        for doc in self.truth.objects.values() {
            if let ReadValue::Task(task) = doc {
                if !crate::queries::selects(query, task) || !self.truth.visible(party, task.project) {
                    continue;
                }
                let subtree = self.truth.objects.values().filter_map(|value| {
                    if let ReadValue::Task(member) = value {
                        if member.chip.task == task.chip.task || member.lineage.as_slice().contains(&task.chip.task) {
                            Some(member.as_ref())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });
                rows.push(crate::queries::line(task, subtree, self.conversation_read(task)));
            }
        }
        crate::queries::order(&mut rows, query.order);
        rows
    }
    pub(crate) fn conversation_read(&self, task: &Task) -> Option<c::ConversationRead> {
        let c::OwnerRequester::Person(requester) = task.requester else { return None };
        if !task.flags.conversation {
            return None;
        }
        for source in &self.truth.sources {
            if source.party != requester {
                continue;
            }
            if let crate::Rows::Tasks(rows) = &source.rows
                && let Some(row) = rows.iter().find(|row| row.chip.task == task.chip.task)
            {
                return row.conversation;
            }
        }
        Some(c::ConversationRead { seen: 0, unread: c::Count { value: 0, limited: false } })
    }
}
