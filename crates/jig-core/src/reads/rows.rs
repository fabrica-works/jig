//! Read-only construction from bounded authenticated owner rows.
use super::vocabulary::{row_bytes, row_owner, row_visible};
use super::{Answer, Cursor, Filter, Group, Historical, Page, PageRow, Past, Position, Range, Read, Row, Scope, Why};
use crate::documents::{self, Document, Missing, Proposer};
use crate::{Core, CoreKey, CoreRecord, Key, Limits, Record};
use alloc::boxed::Box;
use jig_core_notes as notes;
use jig_core_tasks as tasks;
use skein_lib::List;

fn note_project(scope: &notes::Scope) -> u32 {
    match scope {
        notes::Scope::Deployment => 0,
        notes::Scope::Project { project }
        | notes::Scope::Goal { project, .. }
        | notes::Scope::Resources { project, .. } => *project,
    }
}

pub(super) fn scope_allowed(core: &Core, party: u64, scope: &Scope) -> bool {
    match scope {
        Scope::Goals { project } | Scope::Notes { project } | Scope::Spend { project } => {
            core.people.role(party, *project).is_some()
        }
        Scope::ScopedNotes { scope, .. } => core.people.role(party, note_project(scope)).is_some(),
        Scope::Inbox { filter, .. } => match filter.project {
            Some(project) => core.people.role(party, project).is_some(),
            None => true,
        },
        Scope::Task { task }
        | Scope::Escalation { task }
        | Scope::Question { task, .. }
        | Scope::PersonTask { task }
        | Scope::Result { task }
        | Scope::Reply { task, .. }
        | Scope::Turn { task, .. }
        | Scope::EndedDelegates { task }
        | Scope::History { task, .. }
        | Scope::Activity { task }
        | Scope::Proposal { proposer: Proposer::Task(task), .. } => match core.tasks.task(*task) {
            Some(row) => core.people.role(party, row.project).is_some(),
            None => true,
        },
        Scope::Proposal { proposer: Proposer::Party(_), .. } | Scope::Conversations => true,
    }
}

fn subtree(core: &Core, task: u64, root: u64, depth: u32) -> bool {
    let mut number = task;
    for _ in 0..=depth {
        if number == root {
            return true;
        }
        let Some(row) = core.tasks.task(number) else {
            return false;
        };
        match row.requester {
            tasks::Party::Task(parent) => number = parent,
            tasks::Party::Person(_) | tasks::Party::Deployment { .. } => return false,
        }
    }
    false
}

fn inbox_filter(core: &Core, limits: &Limits, filter: Filter, entry: &documents::InboxEntry) -> bool {
    if filter.project.is_some() && entry.project != filter.project {
        return false;
    }
    match filter.goal {
        Some(root) => {
            let task = match entry.task {
                documents::TaskLink::Live(ref chip) | documents::TaskLink::Retained { ref chip, .. } => chip.number,
                documents::TaskLink::Stored { number, .. } => number,
            };
            subtree(core, task, root, limits.tasks.depth)
        }
        None => true,
    }
}

pub(super) fn needs_store(read: &Read, document: &Document) -> bool {
    match document {
        Document::Inbox(value) => match read {
            Read::Inbox { group: Group::Updates | Group::Results | Group::Replies, .. } => {
                if !value.updates_time_complete {
                    return true;
                }
                for entry in &value.updates {
                    match entry.object {
                        documents::InboxObject::Reply { .. } if entry.activity.is_none() => return true,
                        documents::InboxObject::Reply { .. }
                        | documents::InboxObject::Result { .. }
                        | documents::InboxObject::Question { .. }
                        | documents::InboxObject::Proposal { .. }
                        | documents::InboxObject::Escalation { .. }
                        | documents::InboxObject::PersonTask => {}
                    }
                }
                false
            }
            Read::Inbox {
                group: Group::Needs | Group::Proposals | Group::Escalations | Group::Questions | Group::PersonTasks,
                ..
            }
            | Read::Frame
            | Read::Task { .. }
            | Read::Proposal { .. }
            | Read::Escalation { .. }
            | Read::Question { .. }
            | Read::PersonTask { .. }
            | Read::Result { .. }
            | Read::Reply { .. }
            | Read::Turn { .. }
            | Read::EndedDelegates { .. }
            | Read::History { .. }
            | Read::Activity { .. }
            | Read::Conversations { .. }
            | Read::Goals { .. }
            | Read::Project { .. }
            | Read::Notes { .. }
            | Read::ScopedNotes { .. }
            | Read::Spend { .. }
            | Read::Deployment
            | Read::Application { .. } => false,
        },
        Document::Frame(_)
        | Document::Chips(_)
        | Document::Run(_)
        | Document::Task(_)
        | Document::Tree(_)
        | Document::Proposal(_)
        | Document::Escalation(_)
        | Document::Question(_)
        | Document::PersonTask(_)
        | Document::Result(_)
        | Document::Conversations(_)
        | Document::Goals(_)
        | Document::Tasks(_)
        | Document::Project(_)
        | Document::Note(_)
        | Document::Deployment(_) => false,
    }
}

fn inbox_position(group: Group, cursor: documents::Cursor, activity: Option<u64>) -> Option<Position> {
    match cursor {
        documents::Cursor::Inbox { task, object, at } => {
            let key = match group {
                Group::Updates | Group::Results | Group::Replies => match object {
                    documents::InboxObject::Result { .. } => Key::Tasks(tasks::Key::Ended(task)),
                    documents::InboxObject::Reply { .. } => {
                        Key::Tasks(tasks::Key::Activity { task, position: activity? })
                    }
                    documents::InboxObject::Question { .. }
                    | documents::InboxObject::Proposal { .. }
                    | documents::InboxObject::Escalation { .. }
                    | documents::InboxObject::PersonTask => return None,
                },
                Group::Needs | Group::Proposals | Group::Escalations | Group::Questions | Group::PersonTasks => {
                    match object {
                        documents::InboxObject::Proposal { proposer: Proposer::Party(_), number } => {
                            Key::Tasks(tasks::Key::PersonProposal(number))
                        }
                        documents::InboxObject::Question { .. }
                        | documents::InboxObject::Proposal { proposer: Proposer::Task(_), .. }
                        | documents::InboxObject::Escalation { .. }
                        | documents::InboxObject::PersonTask => Key::Tasks(tasks::Key::Live(task)),
                        documents::InboxObject::Result { .. } | documents::InboxObject::Reply { .. } => return None,
                    }
                }
            };
            Some(Position::Inbox { at, task, object, key })
        }
        documents::Cursor::Number { .. } => None,
    }
}

#[expect(clippy::too_many_lines, reason = "the exhaustive live-to-store seam keeps each document family visible")]
pub(super) fn live_page(core: &Core, limits: &Limits, _party: u64, read: &Read, document: Document) -> Answer {
    let mut rows = List::with_capacity(limits.documents.rows);
    let (page_scope, next) = match document {
        Document::Inbox(value) => {
            let (group, filter) = match read {
                Read::Inbox { group, filter, .. } => (group, filter),
                Read::Frame
                | Read::Task { .. }
                | Read::Proposal { .. }
                | Read::Escalation { .. }
                | Read::Question { .. }
                | Read::PersonTask { .. }
                | Read::Result { .. }
                | Read::Reply { .. }
                | Read::Turn { .. }
                | Read::EndedDelegates { .. }
                | Read::History { .. }
                | Read::Activity { .. }
                | Read::Conversations { .. }
                | Read::Goals { .. }
                | Read::Project { .. }
                | Read::Notes { .. }
                | Read::ScopedNotes { .. }
                | Read::Spend { .. }
                | Read::Deployment
                | Read::Application { .. } => unreachable!("inbox subject comes from inbox read"),
            };
            let needs_done = !value.updates.is_empty();
            let source = match group {
                Group::Needs | Group::Proposals | Group::Escalations | Group::Questions | Group::PersonTasks => {
                    value.needs
                }
                Group::Updates | Group::Results | Group::Replies => value.updates,
            };
            let mut last = None;
            for entry in source.into_boxed() {
                last = Some((entry.cursor, entry.activity));
                if group.includes(entry.object) && inbox_filter(core, limits, *filter, &entry) {
                    rows.push(PageRow::Inbox(entry)).expect("live page fits");
                }
            }
            let scope = Scope::Inbox { group: *group, filter: *filter };
            let next = match value.next {
                Some(after) if group.updates() || !needs_done => Some(Cursor::Live {
                    scope,
                    store_after: match last {
                        Some((cursor, activity)) => inbox_position(*group, cursor, activity),
                        None => match super::cursor(read) {
                            Some(Cursor::Live { store_after, .. }) => store_after.clone(),
                            Some(Cursor::Store { .. }) | None => None,
                        },
                    },
                    after: match last {
                        Some((after, _)) => after,
                        None => after,
                    },
                }),
                Some(_) | None => {
                    let after = match last {
                        Some((cursor, activity)) => inbox_position(*group, cursor, activity),
                        None => match super::cursor(read) {
                            Some(Cursor::Live { store_after, .. }) => store_after.clone(),
                            Some(Cursor::Store { .. }) | None => None,
                        },
                    };
                    Some(Cursor::Store { scope, after })
                }
            };
            return Answer::Page(Box::new(Page { rows, next, cut: false }));
        }
        Document::Conversations(value) => {
            for row in value.lines.into_boxed() {
                rows.push(PageRow::Conversation(row)).expect("live conversations fit");
            }
            (Scope::Conversations, value.next)
        }
        Document::Goals(value) => {
            let project = match read {
                Read::Goals { project, .. } => project,
                Read::Frame
                | Read::Task { .. }
                | Read::Proposal { .. }
                | Read::Escalation { .. }
                | Read::Question { .. }
                | Read::PersonTask { .. }
                | Read::Result { .. }
                | Read::Reply { .. }
                | Read::Turn { .. }
                | Read::EndedDelegates { .. }
                | Read::History { .. }
                | Read::Activity { .. }
                | Read::Inbox { .. }
                | Read::Conversations { .. }
                | Read::Project { .. }
                | Read::Notes { .. }
                | Read::ScopedNotes { .. }
                | Read::Spend { .. }
                | Read::Deployment
                | Read::Application { .. } => unreachable!("goal subject comes from goal read"),
            };
            for row in value.lines.into_boxed() {
                rows.push(PageRow::Goal(row)).expect("live goals fit");
            }
            (Scope::Goals { project: *project }, value.next)
        }
        document @ (Document::Frame(_)
        | Document::Chips(_)
        | Document::Run(_)
        | Document::Task(_)
        | Document::Tree(_)
        | Document::Proposal(_)
        | Document::Escalation(_)
        | Document::Question(_)
        | Document::PersonTask(_)
        | Document::Result(_)
        | Document::Project(_)
        | Document::Tasks(_)
        | Document::Note(_)
        | Document::Deployment(_)) => return Answer::Document(Box::new(document)),
    };
    let next = match next {
        Some(after) => Cursor::Live { scope: page_scope, after, store_after: None },
        None => Cursor::Store { scope: page_scope, after: None },
    };
    Answer::Page(Box::new(Page { rows, next: Some(next), cut: false }))
}

/// Stable address supplied by the owning record, never by a client row number.
#[must_use]
pub fn row_key(row: &Row) -> Option<Position> {
    let key = match &row.value {
        Record::Tasks(value) => Some(Key::Tasks(value.key())),
        Record::Notes(notes::Record::Entry(value)) => Some(Key::Notes(notes::Key::Entry { name: value.name })),
        Record::Core(CoreRecord::Turn(value)) => {
            Some(Key::Core(CoreKey::Turn { task: value.task, attempt: value.attempt, turn: value.turn }))
        }
        Record::Core(CoreRecord::EscalationDecision(value)) => {
            Some(Key::Core(CoreKey::EscalationDecision { task: value.task, revision: value.revision }))
        }
        Record::Core(CoreRecord::ProposalDecision(value)) => {
            Some(Key::Core(CoreKey::ProposalDecision { proposer: value.proposer, proposal: value.proposal }))
        }
        Record::Core(CoreRecord::Terminal(value)) => {
            Some(Key::Core(CoreKey::Terminal { task: value.task, attempt: value.attempt }))
        }
        Record::Core(
            CoreRecord::Deployment(_) | CoreRecord::RunProof(_) | CoreRecord::Call(_) | CoreRecord::Projection(_),
        )
        | Record::People(_)
        | Record::Notes(notes::Record::Line(_)) => None,
    }?;
    match row.inbox {
        Some(documents::Cursor::Inbox { task, object, at }) => Some(Position::Inbox { at, task, object, key }),
        Some(documents::Cursor::Number { .. }) => None,
        None => Some(Position::Owner(key)),
    }
}

fn is_page(scope: &Scope) -> bool {
    match scope {
        Scope::EndedDelegates { .. }
        | Scope::History { .. }
        | Scope::Activity { .. }
        | Scope::Inbox { .. }
        | Scope::Conversations
        | Scope::Goals { .. }
        | Scope::Notes { .. }
        | Scope::ScopedNotes { .. }
        | Scope::Spend { .. } => true,
        Scope::Task { .. }
        | Scope::Proposal { .. }
        | Scope::Escalation { .. }
        | Scope::Question { .. }
        | Scope::PersonTask { .. }
        | Scope::Result { .. }
        | Scope::Reply { .. }
        | Scope::Turn { .. } => false,
    }
}

fn later(previous: Option<&Position>, key: &Position, backwards: bool) -> bool {
    match previous {
        Some(previous) => {
            if backwards {
                key < previous
            } else {
                key > previous
            }
        }
        None => true,
    }
}

fn owner_matches(row: &Row, task: u64) -> bool {
    match row_owner(row) {
        Some(owner) => owner.number == task,
        None => false,
    }
}

fn ended(task: &tasks::TaskRecord) -> bool {
    match task.phase {
        tasks::Phase::Ended(_) => true,
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => false,
    }
}

#[expect(clippy::manual_map, reason = "the strict subset uses an exhaustive match rather than a closure")]
pub(super) fn from_store(
    core: &Core,
    limits: &Limits,
    range: &Range,
    raw: Box<[Row]>,
    next: Option<Position>,
) -> Answer {
    if raw.len() > usize::try_from(range.rows).expect("row bound fits") {
        return Answer::None(Why::Busy);
    }
    let mut total = 0_u64;
    let mut previous = range.after.clone();
    for row in &raw {
        let Some(bytes) = row_bytes(row) else {
            return Answer::None(Why::Busy);
        };
        let Some(sum) = total.checked_add(bytes) else {
            return Answer::None(Why::Busy);
        };
        total = sum;
        if row.linked.len()
            > usize::try_from(linked_limit(limits).expect("linked cap fits")).expect("linked count fits")
        {
            return Answer::None(Why::Busy);
        }
        if total > range.bytes || row.ancestors.len() > usize::try_from(limits.tasks.depth).expect("depth fits") {
            return Answer::None(Why::Busy);
        }
        let Some(key) = row_key(row) else {
            return Answer::None(Why::Unknown);
        };
        if !position_allowed(&range.scope, &key) || !later(previous.as_ref(), &key, range.backwards) {
            return Answer::None(Why::Unknown);
        }
        previous = Some(key);
    }
    if let Some(next) = &next {
        // A continuation names the last examined row, including filtered rows.
        if !position_allowed(&range.scope, next)
            || previous.as_ref() != Some(next)
            || !later(range.after.as_ref(), next, range.backwards)
        {
            return Answer::None(Why::Unknown);
        }
    }
    let mut page = Page {
        rows: List::with_capacity(range.rows),
        next: match next {
            Some(after) => Some(Cursor::Store { scope: range.scope.clone(), after: Some(after) }),
            None => None,
        },
        cut: false,
    };
    for row in raw {
        if !row_visible(core, range.party, &row) {
            continue;
        }
        if is_page(&range.scope) {
            match page_row(core, limits, range, row) {
                Ok(Some(value)) => {
                    page.rows.push(value).expect("one bounded output per input row");
                }
                Ok(None) | Err(Missing::NotAllowed | Missing::Unknown | Missing::Ended) => {}
                Err(Missing::TooLarge) => return Answer::None(Why::Busy),
            }
        } else {
            match object(core, limits, range, row) {
                Ok(Some(answer)) => return answer,
                Ok(None) | Err(Missing::Unknown | Missing::NotAllowed | Missing::Ended) => {}
                Err(Missing::TooLarge) => return Answer::None(Why::Busy),
            }
        }
    }
    if is_page(&range.scope) {
        Answer::Page(Box::new(page))
    } else if let Scope::Reply { .. } = &range.scope {
        Answer::None(Why::Gone)
    } else {
        Answer::None(Why::Unknown)
    }
}

fn historical(value: Past, at: Option<skein_lib::Wall>, by: Option<tasks::Party>) -> Answer {
    Answer::Historical(Box::new(Historical { value, at, by }))
}

#[expect(clippy::manual_map, reason = "the strict subset uses an exhaustive match rather than a function pointer")]
fn result_actor(row: &tasks::TaskRecord) -> Option<tasks::Party> {
    match &row.phase {
        tasks::Phase::Ended(tasks::Ending::Cancelled { by, .. }) => Some(*by),
        tasks::Phase::Ended(tasks::Ending::Done(_) | tasks::Ending::Failed { .. }) => match row.executor {
            tasks::Executor::Person(_) => match row.person_answer {
                Some(answer) => Some(tasks::Party::Person(answer.by)),
                None => None,
            },
            tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. } => None,
        },
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => None,
    }
}

#[expect(clippy::manual_map, reason = "the strict subset uses exhaustive matches rather than closures")]
#[expect(clippy::too_many_lines, reason = "exhaustive read construction keeps each owner conversion visible")]
fn object(core: &Core, limits: &Limits, range: &Range, row: Row) -> Result<Option<Answer>, Missing> {
    match &range.scope {
        Scope::Task { task } | Scope::PersonTask { task } | Scope::Result { task } => {
            let Some(owner) = row_owner(&row) else {
                return Ok(None);
            };
            if owner.number != *task || !ended(owner) {
                return Ok(None);
            }
            let result = hydrate_result(
                core,
                limits,
                range.party,
                &row,
                documents::result_from_row(core, limits, range.party, owner)?,
            )?;
            let answer = match range.scope {
                Scope::Task { .. } => historical(
                    Past::Task {
                        document: Box::new(hydrate_task(
                            core,
                            limits,
                            range.party,
                            &row,
                            documents::task_from_row(core, limits, range.party, owner)?,
                        )?),
                        result: Box::new(result),
                    },
                    owner.ended_at,
                    result_actor(owner),
                ),
                Scope::PersonTask { .. } => historical(
                    Past::PersonTask {
                        document: Box::new(hydrate_person(
                            core,
                            limits,
                            range.party,
                            &row,
                            documents::person_from_row(core, limits, range.party, owner)?,
                        )?),
                        result: Box::new(result),
                    },
                    owner.ended_at,
                    result_actor(owner),
                ),
                Scope::Result { .. } => Answer::Document(Box::new(Document::Result(Box::new(result)))),
                Scope::Proposal { .. }
                | Scope::Escalation { .. }
                | Scope::Question { .. }
                | Scope::Reply { .. }
                | Scope::Turn { .. }
                | Scope::EndedDelegates { .. }
                | Scope::History { .. }
                | Scope::Activity { .. }
                | Scope::Inbox { .. }
                | Scope::Conversations
                | Scope::Goals { .. }
                | Scope::Notes { .. }
                | Scope::ScopedNotes { .. }
                | Scope::Spend { .. } => unreachable!("task object scope"),
            };
            Ok(Some(answer))
        }
        Scope::Proposal { proposer, proposal } => match &row.value {
            Record::Tasks(tasks::Stored::History(value)) => {
                let Some(kept) = &value.proposal else {
                    return Ok(None);
                };
                if *proposer != Proposer::Task(kept.proposer) || kept.number != *proposal {
                    return Ok(None);
                }
                let made = documents::history_from_row(limits, value)?;
                Ok(match made.proposal {
                    Some(proposal) => Some(historical(Past::Proposal(proposal), Some(value.at), Some(value.by))),
                    None => None,
                })
            }
            Record::Tasks(tasks::Stored::PersonProposal(value)) => {
                if *proposer != Proposer::Party(value.proposer) || value.number != *proposal {
                    return Ok(None);
                }
                let decision = match &row.related {
                    Some(record) => match proposal_decision(record) {
                        Some(decision)
                            if decision.proposal == *proposal
                                && decision.project == value.project
                                && decision.proposer == Proposer::Party(value.proposer) =>
                        {
                            Some(*decision)
                        }
                        Some(_) | None => None,
                    },
                    None => None,
                };
                match value.state {
                    tasks::PersonProposalState::Accepted { .. } if decision.is_none() => {
                        return Ok(Some(Answer::None(Why::Gone)));
                    }
                    tasks::PersonProposalState::Accepted { .. }
                    | tasks::PersonProposalState::Pending { .. }
                    | tasks::PersonProposalState::Rejected { .. } => {}
                }
                let (at, by) = match &value.state {
                    tasks::PersonProposalState::Pending { since } => (Some(*since), None),
                    tasks::PersonProposalState::Accepted { by } => (
                        match &decision {
                            Some(value) => Some(value.at),
                            None => None,
                        },
                        Some(*by),
                    ),
                    tasks::PersonProposalState::Rejected { by, at, .. } => (Some(*at), Some(*by)),
                };
                Ok(Some(historical(Past::PersonProposal { proposal: value.clone(), decision }, at, by)))
            }
            Record::Tasks(
                tasks::Stored::Live(_)
                | tasks::Stored::Ended(_)
                | tasks::Stored::Activity(_)
                | tasks::Stored::Milestone(_)
                | tasks::Stored::Stub(_)
                | tasks::Stored::Ledger(_)
                | tasks::Stored::Writer(_)
                | tasks::Stored::Pool(_),
            )
            | Record::Core(_)
            | Record::People(_)
            | Record::Notes(_) => Ok(None),
        },
        Scope::Escalation { task } => {
            let Some(decision) = escalation(&row.value) else {
                return Ok(None);
            };
            if decision.task != *task || !owner_matches(&row, *task) {
                return Ok(None);
            }
            let owner = row_owner(&row).expect("matched owner");
            let hold = match &row.related {
                Some(record) => match milestone_record(record) {
                    Some(value) if value.task == *task => Some(documents::milestone_from_row(limits, value)?),
                    Some(_) | None => None,
                },
                None => None,
            };
            Ok(Some(historical(
                Past::Escalation {
                    task: Box::new(documents::task_from_row(core, limits, range.party, owner)?),
                    decision: decision.clone(),
                    hold,
                },
                Some(decision.at),
                Some(tasks::Party::Person(decision.by)),
            )))
        }
        Scope::Question { task, message } => {
            let Some(line) = activity_record(&row.value) else {
                return Ok(None);
            };
            let (number, by, words) = match &line.line {
                tasks::Line::Words { message, by, kind, words } => match kind {
                    tasks::MessageKind::Question => (message, by, words),
                    tasks::MessageKind::Words
                    | tasks::MessageKind::Answer { .. }
                    | tasks::MessageKind::EffectNotMade { .. }
                    | tasks::MessageKind::Amendment { .. }
                    | tasks::MessageKind::Proposal { .. }
                    | tasks::MessageKind::ProposalDecision { .. }
                    | tasks::MessageKind::Escalation { .. }
                    | tasks::MessageKind::Notice { .. }
                    | tasks::MessageKind::Timer { .. }
                    | tasks::MessageKind::News { .. }
                    | tasks::MessageKind::PersonStalled { .. }
                    | tasks::MessageKind::Result(_) => return Ok(None),
                },
                tasks::Line::Heard(_)
                | tasks::Line::Notified { .. }
                | tasks::Line::RunBegan { .. }
                | tasks::Line::RunEnded { .. }
                | tasks::Line::Turn { .. }
                | tasks::Line::Decided { .. }
                | tasks::Line::Milestone { .. } => return Ok(None),
            };
            if number != message || line.task != *task || !owner_matches(&row, *task) {
                return Ok(None);
            }
            if words.len() > usize::try_from(limits.documents.text_bytes).expect("text cap") {
                return Err(Missing::TooLarge);
            }
            let owner = row_owner(&row).expect("matched owner");
            let answer = match &row.related {
                Some(record) => match activity_record(record) {
                    Some(answer) if answer.task == *task && answer.position > line.position => match &answer.line {
                        tasks::Line::Words { kind: tasks::MessageKind::Answer { question }, .. }
                            if *question == *number =>
                        {
                            Some(answer.clone())
                        }
                        tasks::Line::Words { .. }
                        | tasks::Line::Heard(_)
                        | tasks::Line::Notified { .. }
                        | tasks::Line::RunBegan { .. }
                        | tasks::Line::RunEnded { .. }
                        | tasks::Line::Turn { .. }
                        | tasks::Line::Decided { .. }
                        | tasks::Line::Milestone { .. } => None,
                    },
                    Some(_) | None => None,
                },
                None => None,
            };
            let ending = if ended(owner) {
                Some(Box::new(documents::result_from_row(core, limits, range.party, owner)?))
            } else {
                None
            };
            Ok(Some(historical(
                Past::Question {
                    question: documents::Question {
                        acts: List::with_capacity(0),
                        task: documents::chip_from_row_with_core(core, limits, owner)?,
                        message: *number,
                        from: *by,
                        at: line.at,
                        words: words.clone(),
                    },
                    answer,
                    ending,
                },
                Some(line.at),
                Some(*by),
            )))
        }
        Scope::Reply { task, activity } => {
            let Some(line) = activity_record(&row.value) else {
                return Ok(None);
            };
            if line.task != *task || line.position != *activity || !owner_matches(&row, *task) {
                return Ok(None);
            }
            match &line.line {
                tasks::Line::Notified { recipient: tasks::Party::Person(person), word } if *person == range.party => {
                    Ok(Some(historical(Past::Reply(word.clone()), Some(line.at), Some(word.from))))
                }
                tasks::Line::Turn { reply, attempt, turn, .. } => {
                    let owner = row_owner(&row).expect("matched reply owner");
                    if core.chat_requester(owner) != Some(range.party) || *reply > owner.last_reply {
                        return Ok(None);
                    }
                    let Some(related) = &row.related else {
                        return Ok(None);
                    };
                    let Some(value) = turn_record(related) else {
                        return Ok(None);
                    };
                    if value.task != *task || value.attempt != *attempt || value.turn != *turn {
                        return Err(Missing::Unknown);
                    }
                    Ok(Some(historical(Past::Turn(value.clone()), Some(value.at), None)))
                }
                tasks::Line::Words { .. }
                | tasks::Line::Heard(_)
                | tasks::Line::Notified { .. }
                | tasks::Line::RunBegan { .. }
                | tasks::Line::RunEnded { .. }
                | tasks::Line::Decided { .. }
                | tasks::Line::Milestone { .. } => Ok(None),
            }
        }
        Scope::Turn { task, attempt, turn } => {
            let Some(value) = turn_record(&row.value) else {
                return Ok(None);
            };
            if value.task != *task || value.attempt != *attempt || value.turn != *turn || !owner_matches(&row, *task) {
                return Ok(None);
            }
            Ok(Some(historical(Past::Turn(value.clone()), Some(value.at), None)))
        }
        Scope::EndedDelegates { .. }
        | Scope::History { .. }
        | Scope::Activity { .. }
        | Scope::Inbox { .. }
        | Scope::Conversations
        | Scope::Goals { .. }
        | Scope::Notes { .. }
        | Scope::ScopedNotes { .. }
        | Scope::Spend { .. } => Ok(None),
    }
}

#[expect(clippy::too_many_lines, reason = "exhaustive read construction keeps each owner conversion visible")]
fn page_row(core: &Core, limits: &Limits, range: &Range, row: Row) -> Result<Option<PageRow>, Missing> {
    match &range.scope {
        Scope::EndedDelegates { task } => {
            let Some(owner) = row_owner(&row) else {
                return Ok(None);
            };
            if owner.requester != tasks::Party::Task(*task) || !ended(owner) {
                return Ok(None);
            }
            Ok(Some(PageRow::EndedDelegate(documents::ended_delegate_from_row(limits, owner)?)))
        }
        Scope::History { task, subtree } => {
            let Some(owner) = row_owner(&row) else {
                return Ok(None);
            };
            if owner.number != *task && (!*subtree || !row.ancestors.contains(task)) {
                return Ok(None);
            }
            match &row.value {
                Record::Tasks(tasks::Stored::History(value)) if value.task == owner.number => {
                    Ok(Some(PageRow::History(documents::history_from_row(limits, value)?)))
                }
                Record::Tasks(tasks::Stored::Milestone(value)) if value.task == owner.number => {
                    Ok(Some(PageRow::Milestone(documents::milestone_from_row(limits, value)?)))
                }
                Record::Tasks(
                    tasks::Stored::History(_)
                    | tasks::Stored::Milestone(_)
                    | tasks::Stored::Activity(_)
                    | tasks::Stored::PersonProposal(_)
                    | tasks::Stored::Live(_)
                    | tasks::Stored::Ended(_)
                    | tasks::Stored::Stub(_)
                    | tasks::Stored::Ledger(_)
                    | tasks::Stored::Writer(_)
                    | tasks::Stored::Pool(_),
                )
                | Record::Core(_)
                | Record::People(_)
                | Record::Notes(_) => Ok(None),
            }
        }
        Scope::Activity { task } => {
            let Some(line) = activity_record(&row.value) else {
                return Ok(None);
            };
            if line.task != *task
                || match &row.owner {
                    Some(owner) => Some(owner.number),
                    None => None,
                } != Some(*task)
            {
                return Ok(None);
            }
            Ok(Some(PageRow::Activity(line.clone())))
        }
        Scope::ScopedNotes { scope, description } => {
            let Some(value) = note(&row.value) else { return Ok(None) };
            if value.scope != *scope
                || match description {
                    Some(query) => skein_lib::bytes::find(&value.description, query).is_none(),
                    None => false,
                }
            {
                return Ok(None);
            }
            Ok(Some(PageRow::Note(documents::note_from_entry(core, limits, range.party, value)?)))
        }
        Scope::Notes { project } => {
            let Some(value) = note(&row.value) else {
                return Ok(None);
            };
            let belongs = match value.scope {
                notes::Scope::Deployment => *project == 0,
                notes::Scope::Project { project: owner }
                | notes::Scope::Goal { project: owner, .. }
                | notes::Scope::Resources { project: owner, .. } => owner == *project,
            };
            if !belongs {
                return Ok(None);
            }
            Ok(Some(PageRow::Note(documents::note_from_entry(core, limits, range.party, value)?)))
        }
        Scope::Spend { project } => {
            let Some(value) = ledger(&row.value) else {
                return Ok(None);
            };
            match value.funder {
                tasks::Funder::Period { project: owner, .. } if owner == *project => Ok(Some(PageRow::Spend(*value))),
                tasks::Funder::Period { .. }
                | tasks::Funder::Task(_)
                | tasks::Funder::Pool { .. }
                | tasks::Funder::Recurring { .. } => Ok(None),
            }
        }
        Scope::Goals { project } => {
            let Some(owner) = row_owner(&row) else {
                return Ok(None);
            };
            if owner.project != *project
                || owner.tracked.is_none()
                || !ended(owner)
                || core.tasks.task(owner.number).is_some()
            {
                return Ok(None);
            }
            let mut phases = documents::PhaseCounts::ZERO;
            for linked in &row.linked {
                if !linked_below(&row, linked, owner.number, limits.tasks.depth) {
                    continue;
                }
                match linked.phase {
                    tasks::Phase::Ended(tasks::Ending::Done(_)) => phases.done = phases.done.saturating_add(1),
                    tasks::Phase::Ended(tasks::Ending::Failed { .. }) => {
                        phases.failed = phases.failed.saturating_add(1);
                    }
                    tasks::Phase::Ended(tasks::Ending::Cancelled { .. }) => {
                        phases.cancelled = phases.cancelled.saturating_add(1);
                    }
                    tasks::Phase::Waiting
                    | tasks::Phase::Active(_)
                    | tasks::Phase::Closing(_)
                    | tasks::Phase::Held { .. } => return Err(Missing::Unknown),
                }
            }
            match owner.phase {
                tasks::Phase::Ended(tasks::Ending::Done(_)) => phases.done = phases.done.saturating_add(1),
                tasks::Phase::Ended(tasks::Ending::Failed { .. }) => phases.failed = phases.failed.saturating_add(1),
                tasks::Phase::Ended(tasks::Ending::Cancelled { .. }) => {
                    phases.cancelled = phases.cancelled.saturating_add(1);
                }
                tasks::Phase::Waiting
                | tasks::Phase::Active(_)
                | tasks::Phase::Closing(_)
                | tasks::Phase::Held { .. } => unreachable!("ended goal"),
            }
            Ok(Some(PageRow::Goal(documents::GoalLine {
                chip: documents::chip_from_row_with_core(core, limits, owner)?,
                priority: owner.tracked.expect("tracked goal"),
                phases,
                budget: documents::budget_from_row(owner),
                held_below: 0,
            })))
        }
        Scope::Conversations => {
            let Some(owner) = row_owner(&row) else {
                return Ok(None);
            };
            if owner.requester != tasks::Party::Person(range.party)
                || owner.tracked.is_some()
                || owner.recurring.is_some()
                || !ended(owner)
                || core.tasks.task(owner.number).is_some()
            {
                return Ok(None);
            }
            match owner.executor {
                tasks::Executor::Agent { .. } => {}
                tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) => return Ok(None),
            }
            Ok(Some(PageRow::Conversation(documents::conversation_from_row(core, limits, range.party, owner)?)))
        }
        Scope::Inbox { group, filter } => inbox_row(core, limits, range.party, *group, *filter, row),
        Scope::Task { .. }
        | Scope::Proposal { .. }
        | Scope::Escalation { .. }
        | Scope::Question { .. }
        | Scope::PersonTask { .. }
        | Scope::Result { .. }
        | Scope::Reply { .. }
        | Scope::Turn { .. } => Ok(None),
    }
}

#[expect(clippy::too_many_lines, reason = "One exhaustive projection of the two inbox groups.")]
fn inbox_row(
    core: &Core,
    limits: &Limits,
    party: u64,
    group: Group,
    filter: Filter,
    row: Row,
) -> Result<Option<PageRow>, Missing> {
    match row.inbox {
        Some(documents::Cursor::Inbox { object, .. }) if !group.includes(object) => return Ok(None),
        Some(documents::Cursor::Inbox { .. } | documents::Cursor::Number { .. }) | None => {}
    }
    match &row.value {
        Record::Tasks(tasks::Stored::PersonProposal(proposal)) => {
            return person_proposal_need(core, limits, party, group, filter, &row, proposal);
        }
        Record::Tasks(
            tasks::Stored::Activity(_)
            | tasks::Stored::History(_)
            | tasks::Stored::Milestone(_)
            | tasks::Stored::Live(_)
            | tasks::Stored::Ended(_)
            | tasks::Stored::Stub(_)
            | tasks::Stored::Ledger(_)
            | tasks::Stored::Writer(_)
            | tasks::Stored::Pool(_),
        )
        | Record::Core(_)
        | Record::People(_)
        | Record::Notes(_) => {}
    }
    let Some(owner) = row_owner(&row) else {
        return Ok(None);
    };
    if filter.project.is_some() && filter.project != Some(owner.project) {
        return Ok(None);
    }
    if let Some(goal) = filter.goal
        && owner.number != goal
        && !row.ancestors.contains(&goal)
    {
        return Ok(None);
    }
    match group {
        Group::Updates | Group::Results | Group::Replies => {
            if let Some(line) = activity_record(&row.value) {
                return reply_update(core, limits, party, &row, owner, line);
            }
            if !ended(owner)
                || owner.requester != tasks::Party::Person(party)
                || owner.result_position <= core.people.results_seen(party).unwrap_or(0)
            {
                return Ok(None);
            }
            let object = documents::InboxObject::Result { position: owner.result_position };
            let cursor = documents::Cursor::Inbox { task: owner.number, object, at: owner.ended_at };
            if row.inbox != Some(cursor) {
                return Err(Missing::Unknown);
            }
            Ok(Some(PageRow::Inbox(documents::InboxEntry {
                reply: None,
                task: documents::TaskLink::Retained {
                    chip: documents::chip_from_row_with_core(core, limits, owner)?,
                    at: owner.ended_at,
                },
                project: Some(owner.project),
                object,
                activity: None,
                at: owner.ended_at,
                unread: Some(true),
                cursor,
            })))
        }
        Group::Needs | Group::Proposals | Group::Escalations | Group::Questions | Group::PersonTasks => {
            // The store may page current owner rows beyond the people's cache.
            // One waiting reference per row is selected by its related activity.
            let entries = core.waiting_entries(limits, owner);
            for entry in &entries {
                let addressed = match entry.whom {
                    jig_core_people::Whom::Person(person) => person == party,
                    jig_core_people::Whom::Role { project, role } => match core.people.role(party, project) {
                        Some(held) => held.number() == role,
                        None => false,
                    },
                };
                if !addressed {
                    continue;
                }
                let object = match entry.kind {
                    jig_core_people::EntryKind::Question { message } => documents::InboxObject::Question { message },
                    jig_core_people::EntryKind::Proposal { number } => {
                        documents::InboxObject::Proposal { proposer: Proposer::Task(owner.number), number }
                    }
                    jig_core_people::EntryKind::Escalation { revision } => {
                        documents::InboxObject::Escalation { revision }
                    }
                    jig_core_people::EntryKind::PersonTask => documents::InboxObject::PersonTask,
                    jig_core_people::EntryKind::Reply { .. } | jig_core_people::EntryKind::ChatReply { .. } => continue,
                };
                let cursor = documents::Cursor::Inbox { task: owner.number, object, at: Some(entry.at) };
                if row.inbox != Some(cursor) {
                    continue;
                }
                return Ok(Some(PageRow::Inbox(documents::InboxEntry {
                    reply: None,
                    task: documents::TaskLink::Live(documents::chip_from_row_with_core(core, limits, owner)?),
                    project: Some(owner.project),
                    object,
                    activity: None,
                    at: Some(entry.at),
                    unread: None,
                    cursor,
                })));
            }
            Ok(None)
        }
    }
}

fn person_proposal_need(
    core: &Core,
    _limits: &Limits,
    party: u64,
    group: Group,
    filter: Filter,
    row: &Row,
    proposal: &tasks::PersonProposal,
) -> Result<Option<PageRow>, Missing> {
    if !group.includes(documents::InboxObject::Proposal {
        proposer: Proposer::Party(proposal.proposer),
        number: proposal.number,
    }) || (filter.project.is_some() && filter.project != Some(proposal.project))
        || (filter.goal.is_some() && filter.goal != Some(proposal.goal.number))
    {
        return Ok(None);
    }
    let at = match proposal.state {
        tasks::PersonProposalState::Pending { since } => since,
        tasks::PersonProposalState::Accepted { .. } | tasks::PersonProposalState::Rejected { .. } => return Ok(None),
    };
    let mut addressed = false;
    for entry in core.person_proposal_entries(proposal) {
        match entry.whom {
            jig_core_people::Whom::Person(person) => addressed = addressed || person == party,
            jig_core_people::Whom::Role { project, role } => {
                if let Some(held) = core.people.role(party, project) {
                    addressed = addressed || held.number() == role;
                }
            }
        }
    }
    if !addressed {
        return Ok(None);
    }
    let object =
        documents::InboxObject::Proposal { proposer: Proposer::Party(proposal.proposer), number: proposal.number };
    let cursor = documents::Cursor::Inbox { task: proposal.goal.number, object, at: Some(at) };
    if row.inbox != Some(cursor) {
        return Err(Missing::Unknown);
    }
    Ok(Some(PageRow::Inbox(documents::InboxEntry {
        reply: None,
        task: documents::TaskLink::Stored { number: proposal.goal.number, status: None },
        project: Some(proposal.project),
        object,
        activity: None,
        at: Some(at),
        unread: None,
        cursor,
    })))
}

fn reply_update(
    core: &Core,
    limits: &Limits,
    party: u64,
    row: &Row,
    owner: &tasks::TaskRecord,
    line: &tasks::ActivityLine,
) -> Result<Option<PageRow>, Missing> {
    if line.task != owner.number {
        return Err(Missing::Unknown);
    }
    let (message, reply) = match &line.line {
        tasks::Line::Turn { reply, .. } => {
            if core.chat_requester(owner) != Some(party)
                || *reply > owner.last_reply
                || *reply <= core.people.chat_seen(party, owner.number).unwrap_or(0)
            {
                return Ok(None);
            }
            (*reply, Some(Box::new(documents::conversation_from_row(core, limits, party, owner)?)))
        }
        tasks::Line::Notified { recipient: tasks::Party::Person(recipient), word } if *recipient == party => {
            if word.number <= core.people.results_seen(party).unwrap_or(0) {
                return Ok(None);
            }
            (word.number, None)
        }
        tasks::Line::Notified { .. }
        | tasks::Line::Words { .. }
        | tasks::Line::Heard(_)
        | tasks::Line::RunBegan { .. }
        | tasks::Line::RunEnded { .. }
        | tasks::Line::Decided { .. }
        | tasks::Line::Milestone { .. } => return Ok(None),
    };
    let object = documents::InboxObject::Reply { message };
    let cursor = documents::Cursor::Inbox { task: owner.number, object, at: Some(line.at) };
    if row.inbox != Some(cursor) {
        return Err(Missing::Unknown);
    }
    Ok(Some(PageRow::Inbox(documents::InboxEntry {
        reply,
        task: documents::TaskLink::Retained {
            chip: documents::chip_from_row_with_core(core, limits, owner)?,
            at: owner.ended_at,
        },
        project: Some(owner.project),
        object,
        activity: Some(line.position),
        at: Some(line.at),
        unread: Some(true),
        cursor,
    })))
}

fn proposal_decision(record: &Record) -> Option<&crate::ProposalDecisionRecord> {
    match record {
        Record::Core(CoreRecord::ProposalDecision(value)) => Some(value),
        Record::Core(
            CoreRecord::Deployment(_)
            | CoreRecord::Projection(_)
            | CoreRecord::RunProof(_)
            | CoreRecord::Call(_)
            | CoreRecord::Terminal(_)
            | CoreRecord::Turn(_)
            | CoreRecord::EscalationDecision(_),
        )
        | Record::Tasks(_)
        | Record::Notes(_)
        | Record::People(_) => None,
    }
}

fn milestone_record(record: &Record) -> Option<&tasks::Milestone> {
    match record {
        Record::Tasks(tasks::Stored::Milestone(value)) => Some(value),
        Record::Tasks(
            tasks::Stored::Activity(_)
            | tasks::Stored::PersonProposal(_)
            | tasks::Stored::History(_)
            | tasks::Stored::Live(_)
            | tasks::Stored::Ended(_)
            | tasks::Stored::Stub(_)
            | tasks::Stored::Ledger(_)
            | tasks::Stored::Writer(_)
            | tasks::Stored::Pool(_),
        )
        | Record::Core(_)
        | Record::People(_)
        | Record::Notes(_) => None,
    }
}

fn activity_record(record: &Record) -> Option<&tasks::ActivityLine> {
    match record {
        Record::Tasks(tasks::Stored::Activity(value)) => Some(value),
        Record::Tasks(
            tasks::Stored::PersonProposal(_)
            | tasks::Stored::History(_)
            | tasks::Stored::Milestone(_)
            | tasks::Stored::Live(_)
            | tasks::Stored::Ended(_)
            | tasks::Stored::Stub(_)
            | tasks::Stored::Ledger(_)
            | tasks::Stored::Writer(_)
            | tasks::Stored::Pool(_),
        )
        | Record::Core(_)
        | Record::Notes(_)
        | Record::People(_) => None,
    }
}

fn escalation(record: &Record) -> Option<&crate::EscalationDecisionRecord> {
    match record {
        Record::Core(CoreRecord::EscalationDecision(value)) => Some(value),
        Record::Core(
            CoreRecord::Projection(_)
            | CoreRecord::Deployment(_)
            | CoreRecord::Call(_)
            | CoreRecord::RunProof(_)
            | CoreRecord::Turn(_)
            | CoreRecord::Terminal(_)
            | CoreRecord::ProposalDecision(_),
        )
        | Record::Tasks(_)
        | Record::Notes(_)
        | Record::People(_) => None,
    }
}

fn turn_record(record: &Record) -> Option<&crate::TurnRecord> {
    match record {
        Record::Core(CoreRecord::Turn(value)) => Some(value),
        Record::Core(
            CoreRecord::Projection(_)
            | CoreRecord::Deployment(_)
            | CoreRecord::Call(_)
            | CoreRecord::RunProof(_)
            | CoreRecord::Terminal(_)
            | CoreRecord::ProposalDecision(_)
            | CoreRecord::EscalationDecision(_),
        )
        | Record::Tasks(_)
        | Record::Notes(_)
        | Record::People(_) => None,
    }
}

fn note(record: &Record) -> Option<&notes::Entry> {
    match record {
        Record::Notes(notes::Record::Entry(value)) => Some(value),
        Record::Tasks(_) | Record::Core(_) | Record::Notes(notes::Record::Line(_)) | Record::People(_) => None,
    }
}

fn ledger(record: &Record) -> Option<&tasks::FundingRecord> {
    match record {
        Record::Tasks(tasks::Stored::Ledger(value)) => Some(value),
        Record::Tasks(
            tasks::Stored::Activity(_)
            | tasks::Stored::PersonProposal(_)
            | tasks::Stored::History(_)
            | tasks::Stored::Milestone(_)
            | tasks::Stored::Live(_)
            | tasks::Stored::Ended(_)
            | tasks::Stored::Stub(_)
            | tasks::Stored::Writer(_)
            | tasks::Stored::Pool(_),
        )
        | Record::Core(_)
        | Record::Notes(_)
        | Record::People(_) => None,
    }
}

fn inbox_key_allowed(group: Group, task: u64, object: documents::InboxObject, key: &Key) -> bool {
    match group {
        Group::Updates | Group::Results | Group::Replies => match object {
            documents::InboxObject::Result { .. } => key_kind(key) == KeyKind::Ended(task),
            documents::InboxObject::Reply { .. } => key_kind(key) == KeyKind::Activity(task),
            documents::InboxObject::Question { .. }
            | documents::InboxObject::Proposal { .. }
            | documents::InboxObject::Escalation { .. }
            | documents::InboxObject::PersonTask => false,
        },
        Group::Needs | Group::Proposals | Group::Escalations | Group::Questions | Group::PersonTasks => match object {
            documents::InboxObject::Proposal { proposer: Proposer::Party(_), number } => {
                key_kind(key) == KeyKind::PersonProposal(number)
            }
            documents::InboxObject::Question { .. }
            | documents::InboxObject::Proposal { proposer: Proposer::Task(_), .. }
            | documents::InboxObject::Escalation { .. }
            | documents::InboxObject::PersonTask => key_kind(key) == KeyKind::Live(task),
            documents::InboxObject::Result { .. } | documents::InboxObject::Reply { .. } => false,
        },
    }
}

pub(super) fn position_allowed(scope: &Scope, position: &Position) -> bool {
    match position {
        Position::Inbox { task, object, key, .. } => match scope {
            Scope::Inbox { group, .. } => inbox_key_allowed(*group, *task, *object, key),
            Scope::Task { .. }
            | Scope::Proposal { .. }
            | Scope::Escalation { .. }
            | Scope::Question { .. }
            | Scope::PersonTask { .. }
            | Scope::Result { .. }
            | Scope::Reply { .. }
            | Scope::Turn { .. }
            | Scope::EndedDelegates { .. }
            | Scope::History { .. }
            | Scope::Activity { .. }
            | Scope::Conversations
            | Scope::Goals { .. }
            | Scope::Notes { .. }
            | Scope::ScopedNotes { .. }
            | Scope::Spend { .. } => false,
        },
        Position::Owner(key) => {
            let kind = key_kind(key);
            match scope {
                Scope::Task { task } | Scope::PersonTask { task } | Scope::Result { task } => {
                    kind == KeyKind::Ended(*task)
                }
                Scope::Proposal { proposer, proposal } => match proposer {
                    Proposer::Task(task) => kind == KeyKind::History(*task),
                    Proposer::Party(_) => kind == KeyKind::PersonProposal(*proposal),
                },
                Scope::Escalation { task } => kind == KeyKind::Escalation(*task),
                Scope::Question { task, .. } | Scope::Reply { task, .. } | Scope::Activity { task } => {
                    kind == KeyKind::Activity(*task)
                }
                Scope::Turn { task, attempt, turn } => {
                    kind == KeyKind::Turn { task: *task, attempt: *attempt, turn: *turn }
                }
                Scope::History { task, subtree } => match kind {
                    KeyKind::History(owner) | KeyKind::Milestone(owner) => *subtree || owner == *task,
                    KeyKind::Ended(_)
                    | KeyKind::Live(_)
                    | KeyKind::Activity(_)
                    | KeyKind::PersonProposal(_)
                    | KeyKind::Escalation(_)
                    | KeyKind::Turn { .. }
                    | KeyKind::Note
                    | KeyKind::Ledger(_)
                    | KeyKind::Unsupported => false,
                },
                Scope::EndedDelegates { .. } | Scope::Conversations | Scope::Goals { .. } => match kind {
                    KeyKind::Ended(_) => true,
                    KeyKind::Live(_)
                    | KeyKind::History(_)
                    | KeyKind::Milestone(_)
                    | KeyKind::Activity(_)
                    | KeyKind::PersonProposal(_)
                    | KeyKind::Escalation(_)
                    | KeyKind::Turn { .. }
                    | KeyKind::Note
                    | KeyKind::Ledger(_)
                    | KeyKind::Unsupported => false,
                },
                Scope::Notes { .. } | Scope::ScopedNotes { .. } => kind == KeyKind::Note,
                Scope::Spend { project } => kind == KeyKind::Ledger(*project),
                Scope::Inbox { .. } => false,
            }
        }
    }
}

#[derive(PartialEq, Eq)]
enum KeyKind {
    Ended(u64),
    Live(u64),
    History(u64),
    Milestone(u64),
    Activity(u64),
    PersonProposal(u64),
    Escalation(u64),
    Turn { task: u64, attempt: u64, turn: u32 },
    Note,
    Ledger(u32),
    Unsupported,
}
fn key_kind(key: &Key) -> KeyKind {
    match key {
        Key::Tasks(value) => match value {
            tasks::Key::Ended(task) => KeyKind::Ended(*task),
            tasks::Key::Live(task) => KeyKind::Live(*task),
            tasks::Key::History { task, .. } => KeyKind::History(*task),
            tasks::Key::Milestone { task, .. } => KeyKind::Milestone(*task),
            tasks::Key::Activity { task, .. } => KeyKind::Activity(*task),
            tasks::Key::PersonProposal(proposal) => KeyKind::PersonProposal(*proposal),
            tasks::Key::Ledger(tasks::Funder::Period { project, .. }) => KeyKind::Ledger(*project),
            tasks::Key::Ledger(
                tasks::Funder::Task(_) | tasks::Funder::Pool { .. } | tasks::Funder::Recurring { .. },
            )
            | tasks::Key::Stub(_)
            | tasks::Key::Writer(_)
            | tasks::Key::Pool(_) => KeyKind::Unsupported,
        },
        Key::Core(value) => match value {
            CoreKey::EscalationDecision { task, .. } => KeyKind::Escalation(*task),
            CoreKey::Turn { task, attempt, turn } => KeyKind::Turn { task: *task, attempt: *attempt, turn: *turn },
            CoreKey::Projection(_)
            | CoreKey::Deployment
            | CoreKey::RunProof(_)
            | CoreKey::Call(_)
            | CoreKey::Terminal { .. }
            | CoreKey::ProposalDecision { .. } => KeyKind::Unsupported,
        },
        Key::Notes(value) => match value {
            notes::Key::Entry { .. } => KeyKind::Note,
            notes::Key::Line { .. } => KeyKind::Unsupported,
        },
        Key::People(_) => KeyKind::Unsupported,
    }
}

#[expect(clippy::manual_find, reason = "the strict subset iterates without a closure")]
fn linked_row(row: &Row, number: u64) -> Option<&tasks::TaskRecord> {
    for value in &row.linked {
        if value.number == number {
            return Some(value);
        }
    }
    None
}

fn linked_below(row: &Row, candidate: &tasks::TaskRecord, root: u64, depth: u32) -> bool {
    let mut parent = candidate.requester;
    for _ in 0..depth {
        match parent {
            tasks::Party::Task(number) => {
                if number == root {
                    return true;
                }
                let Some(owner) = linked_row(row, number) else {
                    return false;
                };
                parent = owner.requester;
            }
            tasks::Party::Person(_) | tasks::Party::Deployment { .. } => return false,
        }
    }
    false
}

fn hydrate_task(
    core: &Core,
    limits: &Limits,
    party: u64,
    row: &Row,
    mut task: documents::Task,
) -> Result<documents::Task, Missing> {
    let owner = row_owner(row).ok_or(Missing::Unknown)?;
    let mut nearest = List::with_capacity(limits.documents.lines);
    let mut requester = owner.requester;
    for _ in 0..limits.tasks.depth {
        match requester {
            tasks::Party::Task(number) => {
                let parent = match linked_row(row, number) {
                    Some(parent) => parent,
                    None => match core.tasks.task(number) {
                        Some(parent) => parent,
                        None => {
                            task.cut = true;
                            break;
                        }
                    },
                };
                if core.people.role(party, parent.project).is_none() {
                    return Err(Missing::NotAllowed);
                }
                if parent.tracked.is_some() {
                    task.goal = Some(parent.number);
                }
                if nearest.room() == 0 {
                    task.cut = true;
                    break;
                }
                nearest.push(documents::chip_from_row_with_core(core, limits, parent)?).expect("bounded lineage");
                requester = parent.requester;
            }
            tasks::Party::Person(_) | tasks::Party::Deployment { .. } => break,
        }
    }
    let mut lineage = List::with_capacity(limits.documents.lines);
    for index in (0..nearest.len()).rev() {
        lineage.push(nearest.get(index).expect("lineage index").clone()).expect("same lineage cap");
    }
    task.lineage = lineage;
    hydrate_links(core, limits, party, row, &mut task.inputs)?;
    hydrate_links(core, limits, party, row, &mut task.dependencies)?;
    hydrate_links(core, limits, party, row, &mut task.references)?;
    Ok(task)
}

fn hydrate_links(
    core: &Core,
    limits: &Limits,
    party: u64,
    row: &Row,
    links: &mut List<documents::TaskLink>,
) -> Result<(), Missing> {
    for index in 0..links.len() {
        let link = links.get_mut(index).expect("bounded link index");
        let number = match link {
            documents::TaskLink::Live(chip) | documents::TaskLink::Retained { chip, .. } => chip.number,
            documents::TaskLink::Stored { number, .. } => *number,
        };
        if let Some(owner) = linked_row(row, number)
            && ended(owner)
        {
            if core.people.role(party, owner.project).is_none() {
                return Err(Missing::NotAllowed);
            }
            *link = documents::TaskLink::Retained {
                chip: documents::chip_from_row_with_core(core, limits, owner)?,
                at: owner.ended_at,
            };
        }
    }
    Ok(())
}

fn terminal_record(record: &Record) -> Option<&crate::TerminalRecord> {
    match record {
        Record::Core(CoreRecord::Terminal(value)) => Some(value),
        Record::Core(
            CoreRecord::Deployment(_)
            | CoreRecord::RunProof(_)
            | CoreRecord::Call(_)
            | CoreRecord::Projection(_)
            | CoreRecord::Turn(_)
            | CoreRecord::ProposalDecision(_)
            | CoreRecord::EscalationDecision(_),
        )
        | Record::Tasks(_)
        | Record::People(_)
        | Record::Notes(_) => None,
    }
}

fn hydrate_result(
    core: &Core,
    limits: &Limits,
    party: u64,
    row: &Row,
    mut result: documents::ResultDoc,
) -> Result<documents::ResultDoc, Missing> {
    hydrate_links(core, limits, party, row, &mut result.delegates)?;
    if let Some(followups) = &mut result.followups {
        hydrate_links(core, limits, party, row, followups)?;
    }
    if let Some(related) = &row.related
        && let Some(terminal) = terminal_record(related)
        && terminal.task == result.task.number
        && match row_owner(row) {
            Some(owner) => owner.attempt == terminal.attempt,
            None => false,
        }
    {
        match &terminal.end {
            tasks::End::FinishedWithFollowups { followups: tasks::ResultFollowups::Proposal(proposal), .. } => {
                result.proposal = Some(documents::ProposalRef {
                    proposer: Proposer::Task(proposal.proposer),
                    number: proposal.number,
                });
            }
            tasks::End::FinishedWithFollowups { followups: tasks::ResultFollowups::Delegates(_), .. }
            | tasks::End::Finished { .. }
            | tasks::End::Parked
            | tasks::End::Failed(_)
            | tasks::End::Refused => {}
        }
    }
    Ok(result)
}

fn hydrate_person(
    core: &Core,
    limits: &Limits,
    party: u64,
    row: &Row,
    mut person: documents::PersonTask,
) -> Result<documents::PersonTask, Missing> {
    for index in 0..person.results.len() {
        let shown = person.results.get_mut(index).expect("bounded shown results");
        let number = match &shown.task {
            documents::TaskLink::Live(chip) | documents::TaskLink::Retained { chip, .. } => chip.number,
            documents::TaskLink::Stored { number, .. } => *number,
        };
        if let Some(owner) = linked_row(row, number)
            && ended(owner)
        {
            shown.result = Some(documents::result_from_row(core, limits, party, owner)?);
            shown.task = documents::TaskLink::Retained {
                chip: documents::chip_from_row_with_core(core, limits, owner)?,
                at: owner.ended_at,
            };
        }
    }
    Ok(person)
}

fn linked_limit(limits: &Limits) -> Option<u32> {
    limits
        .tasks
        .tree_tasks
        .checked_add(limits.tasks.depth)?
        .checked_add(limits.tasks.inputs)?
        .checked_add(limits.tasks.dependencies)?
        .checked_add(limits.tasks.references)?
        .checked_add(limits.tasks.delegates)?
        .checked_add(limits.tasks.batch)
}
