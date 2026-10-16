//! Independent document reference from the store's durable task rows.
use jig_domain_core::{self as core, waits::WaitsOn};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::Wall;
use std::collections::BTreeMap;

fn live(rows: &BTreeMap<root::Key, root::Record>, task: u64) -> Option<&tasks::TaskRecord> {
    match rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Live(task)))) {
        Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))) => Some(row),
        _ => None,
    }
}

/// Derive waits from durable topology and the scenario's waiting objects.
#[must_use]
pub fn waits_on(rows: &BTreeMap<root::Key, root::Record>, task: u64) -> WaitsOn {
    let Some(owner) = live(rows, task) else { return WaitsOn::Nothing };
    let mut pending = vec![task];
    let mut entries = Vec::new();
    while let Some(number) = pending.pop() {
        let Some(row) = live(rows, number) else { continue };
        pending.extend(row.delegates.iter().rev());
        if matches!(row.phase, tasks::Phase::Closing(_) | tasks::Phase::Ended(_)) {
            continue;
        }
        waiting(row, &mut entries);
    }
    entries.sort_by_key(|(at, number, _, _)| (*at, *number));
    if let Some((_, number, whom, kind)) = entries.first() {
        return WaitsOn::Party {
            whom: *whom,
            entries: u32::try_from(entries.len()).expect("bounded entries"),
            task: *number,
            kind: *kind,
        };
    }
    if matches!(owner.phase, tasks::Phase::Active(tasks::Active::Claimed { .. } | tasks::Active::Running { .. })) {
        WaitsOn::Nothing
    } else if !owner.delegates.is_empty() {
        WaitsOn::Delegates
    } else if matches!(owner.phase, tasks::Phase::Active(tasks::Active::Idle)) {
        WaitsOn::News
    } else {
        WaitsOn::Nothing
    }
}

type Waiting = (Wall, u64, people::Whom, people::EntryKind);

fn waiting(row: &tasks::TaskRecord, entries: &mut Vec<Waiting>) {
    if let tasks::Party::Person(party) = row.requester {
        for word in &row.inbox {
            if word.kind == tasks::MessageKind::Question
                && !row.inbox.iter().any(|answer| answer.kind == tasks::MessageKind::Answer { question: word.number })
            {
                entries.push((
                    word.at,
                    row.number,
                    people::Whom::Person(party),
                    people::EntryKind::Question { message: word.number },
                ));
            }
        }
    }
    if let Some(proposal) = &row.proposal
        && let tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Person(party), since } = proposal.state
    {
        entries.push((
            since,
            row.number,
            people::Whom::Person(party),
            people::EntryKind::Proposal { number: proposal.number },
        ));
    }
    if let tasks::Escalation::Waiting { holder, revision, since, .. } = row.escalation {
        let whom = match holder {
            tasks::EscalationHolder::Person(party) => Some(people::Whom::Person(party)),
            tasks::EscalationHolder::Role { project, role } => Some(people::Whom::Role { project, role }),
            tasks::EscalationHolder::Task(_) => None,
        };
        if let Some(whom) = whom {
            entries.push((since, row.number, whom, people::EntryKind::Escalation { revision }));
        }
    }
    if matches!(row.phase, tasks::Phase::Active(tasks::Active::Due | tasks::Active::Idle)) {
        let whom = match (row.executor, row.taken_by) {
            (tasks::Executor::Person(tasks::PersonAddress::Person(party)), None)
            | (tasks::Executor::Person(tasks::PersonAddress::Role(_)), Some(party)) => {
                Some(people::Whom::Person(party))
            }
            (tasks::Executor::Person(tasks::PersonAddress::Role(role)), None) => {
                Some(people::Whom::Role { project: row.project, role })
            }
            _ => None,
        };
        if let Some(whom) = whom {
            entries.push((
                row.person_wait.expect("person interval").since,
                row.number,
                whom,
                people::EntryKind::PersonTask,
            ));
        }
    }
}

/// Select and order live task numbers using only durable owner rows.
#[must_use]
pub fn lines(rows: &BTreeMap<root::Key, root::Record>, party: u64, query: &core::documents::TaskQuery) -> Vec<u64> {
    use core::documents::{ExecutorKind, PhaseClass, QueryOrder};
    let mut selected: Vec<_> = rows
        .values()
        .filter_map(|record| {
            let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) = record else { return None };
            let class = match row.phase {
                tasks::Phase::Active(_) => PhaseClass::Working,
                tasks::Phase::Waiting => PhaseClass::Waiting,
                tasks::Phase::Held { .. } => PhaseClass::Held,
                tasks::Phase::Closing(_) => PhaseClass::Closing,
                tasks::Phase::Ended(_) => return None,
            };
            let kind = match row.executor {
                tasks::Executor::Agent { .. } => ExecutorKind::Agent,
                tasks::Executor::Procedure { .. } => ExecutorKind::Procedure,
                tasks::Executor::Person(_) => ExecutorKind::Person,
            };
            let parent = match row.requester {
                tasks::Party::Task(number) => Some(number),
                tasks::Party::Person(_) | tasks::Party::Deployment { .. } => None,
            };
            let matches = query.project.is_none_or(|key| key == row.project)
                && query.requester.is_none_or(|key| key == row.requester)
                && query.root.is_none_or(|key| key == row.root)
                && query.parent.is_none_or(|key| Some(key) == parent)
                && query.label.is_none_or(|key| key == row.label)
                && query.phase.is_none_or(|key| key == class)
                && query.executor.is_none_or(|key| key == kind)
                && query.tracked.is_none_or(|key| key == row.tracked.is_some())
                && (query.project.is_some() || query.requester == Some(tasks::Party::Person(party)));
            matches.then_some(row.as_ref())
        })
        .collect();
    selected.sort_by(|a, b| {
        let order = match query.order {
            QueryOrder::Number | QueryOrder::Ended => std::cmp::Ordering::Equal,
            QueryOrder::Changed => b.changed.cmp(&a.changed),
            QueryOrder::Activity => b.last_activity.cmp(&a.last_activity),
            QueryOrder::Priority => (a.tracked.is_none(), a.tracked).cmp(&(b.tracked.is_none(), b.tracked)),
        };
        order.then(a.number.cmp(&b.number))
    });
    selected.into_iter().map(|row| row.number).collect()
}
