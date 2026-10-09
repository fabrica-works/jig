//! Volatile party inbox projections from the task child's durable rows.
//! The core routes these sibling facts on save and restore (domain/engine.md, 4.4).

use alloc::boxed::Box;
use jig_core_authority as authority;
use jig_core_people as people;
use jig_core_tasks as tasks;
use skein_lib::List;

use crate::{Core, Limits};

/// Inbox words, role-routed proposals and the four scalar references: a
/// person-held proposal, escalation, answerable person task and stall update.
pub(crate) fn waiting_capacity(limits: &Limits) -> Option<u32> {
    limits.tasks.inbox_messages.checked_add(limits.authority.roles)?.checked_add(4)
}

fn add(
    entries: &mut List<people::Entry>,
    row: &tasks::TaskRecord,
    whom: people::Whom,
    kind: people::EntryKind,
    at: skein_lib::Wall,
) {
    entries
        .push(people::Entry { task: row.number, project: row.project, whom, kind, at })
        .expect("one bounded projection per task state");
}

fn proposal_kind(kind: tasks::ProposalKind) -> authority::ProposalKind {
    match kind {
        tasks::ProposalKind::Batch => authority::ProposalKind::Batch,
        tasks::ProposalKind::Effect => authority::ProposalKind::Effect,
        tasks::ProposalKind::Amend => authority::ProposalKind::Amend,
        tasks::ProposalKind::Widen => authority::ProposalKind::Widen,
        tasks::ProposalKind::Release => authority::ProposalKind::Escalation,
    }
}

fn question_answered(row: &tasks::TaskRecord, question: u64) -> bool {
    for word in &row.inbox {
        match word.kind {
            tasks::MessageKind::Answer { question: answered } if answered == question => return true,
            tasks::MessageKind::Answer { .. }
            | tasks::MessageKind::Question
            | tasks::MessageKind::EffectNotMade { .. }
            | tasks::MessageKind::Words
            | tasks::MessageKind::Amendment { .. }
            | tasks::MessageKind::Proposal { .. }
            | tasks::MessageKind::ProposalDecision { .. }
            | tasks::MessageKind::Escalation { .. }
            | tasks::MessageKind::Notice { .. }
            | tasks::MessageKind::Timer { .. }
            | tasks::MessageKind::News { .. }
            | tasks::MessageKind::PersonStalled { .. }
            | tasks::MessageKind::Result(_) => {}
        }
    }
    false
}

impl Core {
    /// The authenticated requester of a chat, also usable for a loaded ended task.
    #[must_use]
    pub fn chat_requester(&self, row: &tasks::TaskRecord) -> Option<u64> {
        if row.tracked.is_some() || row.recurring.is_some() {
            return None;
        }
        match row.executor {
            tasks::Executor::Agent { charter } if charter == self.settings.charter => {}
            tasks::Executor::Agent { .. } | tasks::Executor::Person(_) | tasks::Executor::Procedure { .. } => {
                return None;
            }
        }
        match row.requester {
            tasks::Party::Person(party) => Some(party),
            tasks::Party::Task(_) | tasks::Party::Deployment { .. } => None,
        }
    }
    /// Project the currently waiting person-facing references of one task.
    #[must_use]
    #[expect(clippy::too_many_lines, reason = "one exhaustive task projection covers all person-facing waiting kinds")]
    pub fn waiting_entries(&self, limits: &Limits, row: &tasks::TaskRecord) -> Box<[people::Entry]> {
        let capacity = waiting_capacity(limits).expect("validated task inbox projection bound");
        let mut entries = List::with_capacity(capacity);
        match &row.phase {
            tasks::Phase::Ended(_) | tasks::Phase::Closing(_) => return entries.into_boxed(),
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } => {}
        }
        match row.requester {
            tasks::Party::Person(person) => {
                for word in &row.inbox {
                    match word.kind {
                        tasks::MessageKind::Question => {
                            if !question_answered(row, word.number) {
                                add(
                                    &mut entries,
                                    row,
                                    people::Whom::Person(person),
                                    people::EntryKind::Question { message: word.number },
                                    word.at,
                                );
                            }
                        }
                        tasks::MessageKind::Answer { .. } => add(
                            &mut entries,
                            row,
                            people::Whom::Person(person),
                            if self.chat_requester(row).is_some() {
                                people::EntryKind::ChatReply { message: word.number }
                            } else {
                                people::EntryKind::Reply { message: word.number }
                            },
                            word.at,
                        ),
                        tasks::MessageKind::ProposalDecision { .. }
                        | tasks::MessageKind::Proposal { .. }
                        | tasks::MessageKind::EffectNotMade { .. }
                        | tasks::MessageKind::Words
                        | tasks::MessageKind::Amendment { .. }
                        | tasks::MessageKind::Notice { .. }
                        | tasks::MessageKind::Timer { .. }
                        | tasks::MessageKind::News { .. }
                        | tasks::MessageKind::PersonStalled { .. }
                        | tasks::MessageKind::Result(_)
                        | tasks::MessageKind::Escalation { .. } => {}
                    }
                }
            }
            tasks::Party::Task(_) | tasks::Party::Deployment { .. } => {}
        }
        if let Some(proposal) = &row.proposal {
            match proposal.state {
                tasks::ProposalState::Pending { holder, since } => match holder {
                    tasks::ProposalHolder::Person(person) => add(
                        &mut entries,
                        row,
                        people::Whom::Person(person),
                        people::EntryKind::Proposal { number: proposal.number },
                        since,
                    ),
                    tasks::ProposalHolder::Policy { project, kind } => {
                        if let Some(project_policy) = self.authority.policy(project) {
                            for policy in &project_policy.roles {
                                if !policy.decides.allows(proposal_kind(kind)) {
                                    continue;
                                }
                                add(
                                    &mut entries,
                                    row,
                                    people::Whom::Role { project, role: policy.number },
                                    people::EntryKind::Proposal { number: proposal.number },
                                    since,
                                );
                            }
                        }
                    }
                    tasks::ProposalHolder::Task(_) => {}
                },
                tasks::ProposalState::Accepted { .. }
                | tasks::ProposalState::Rejected { .. }
                | tasks::ProposalState::Withdrawn => {}
            }
        }
        match row.escalation {
            tasks::Escalation::Waiting { holder, revision, since, .. } => match holder {
                tasks::EscalationHolder::Person(person) => {
                    add(
                        &mut entries,
                        row,
                        people::Whom::Person(person),
                        people::EntryKind::Escalation { revision },
                        since,
                    );
                }
                tasks::EscalationHolder::Role { project, role } => {
                    add(
                        &mut entries,
                        row,
                        people::Whom::Role { project, role },
                        people::EntryKind::Escalation { revision },
                        since,
                    );
                }
                tasks::EscalationHolder::Task(_) => {}
            },
            tasks::Escalation::Unheld { .. }
            | tasks::Escalation::Routing { .. }
            | tasks::Escalation::Rejected { .. } => {}
        }
        if let Some(wait) = row.person_wait
            && let Some(notice) = wait.notice
            && let tasks::Party::Person(person) = row.requester
        {
            add(
                &mut entries,
                row,
                people::Whom::Person(person),
                people::EntryKind::Reply { message: notice.message },
                notice.at,
            );
        }
        let answerable = match row.phase {
            tasks::Phase::Active(tasks::Active::Due | tasks::Active::Idle) => true,
            tasks::Phase::Waiting
            | tasks::Phase::Active(_)
            | tasks::Phase::Held { .. }
            | tasks::Phase::Closing(_)
            | tasks::Phase::Ended(_) => false,
        };
        match row.executor {
            tasks::Executor::Person(_) if !answerable => {}
            tasks::Executor::Person(tasks::PersonAddress::Person(person)) => {
                if row.taken_by.is_none() {
                    let since = row.person_wait.expect("active person wait").since;
                    add(&mut entries, row, people::Whom::Person(person), people::EntryKind::PersonTask, since);
                }
            }
            tasks::Executor::Person(tasks::PersonAddress::Role(role)) => {
                let since = row.person_wait.expect("active person wait").since;
                let whom = match row.taken_by {
                    Some(person) => people::Whom::Person(person),
                    None => people::Whom::Role { project: row.project, role },
                };
                add(&mut entries, row, whom, people::EntryKind::PersonTask, since);
            }
            tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. } => {}
        }
        entries.into_boxed()
    }
}
