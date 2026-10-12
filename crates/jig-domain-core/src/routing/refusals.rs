//! Total sibling translations into party answer facts
//! (client/domain/contract.md, section 4; domain/people.md, section 5.1).
//! The core reads the current live record for an ending; no historical archive
//! is retained here. Unknown names disclose no object. Retry answers are unsaved.
use crate::{Core, Event};
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Queue, Token};

pub(crate) fn person_task_refusal(core: &Core, task: Option<u64>, why: tasks::Refusal) -> people::Outcome {
    task_answer(core, task, why)
}

pub(super) fn person_move_refusal(core: &Core, task: Option<u64>, why: tasks::Refusal) -> people::Outcome {
    task_answer(core, task, why)
}

pub(super) fn person_message_refusal(core: &Core, task: Option<u64>, why: tasks::Refusal) -> people::Outcome {
    task_answer(core, task, why)
}

fn task_answer(core: &Core, task: Option<u64>, why: tasks::Refusal) -> people::Outcome {
    let refusal = match why {
        tasks::Refusal::PersonRevision { revision } => {
            people::Refusal::Moved { now: people::Moved::Person { revision } }
        }
        tasks::Refusal::Busy => return people::Outcome::Again(people::Again::Busy),
        tasks::Refusal::NotReady => return people::Outcome::Again(people::Again::NotReady),
        tasks::Refusal::ResourceUnavailable => {
            people::Refusal::Authority { proposable: None, lacking: people::Lack::ResourceUnavailable }
        }
        tasks::Refusal::Unknown => people::Refusal::Unknown,
        tasks::Refusal::State => match current_task(core, task) {
            Some(row) => match &row.phase {
                tasks::Phase::Closing(closing) => ended(&closing.ending),
                tasks::Phase::Ended(ending) => ended(ending),
                tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } => {
                    people::Refusal::NotWaiting
                }
            },
            None => people::Refusal::Unknown,
        },
        tasks::Refusal::Executor => people::Refusal::Authority { proposable: None, lacking: people::Lack::TaskKind },
        tasks::Refusal::Reference => people::Refusal::Authority { proposable: None, lacking: people::Lack::Reference },
        tasks::Refusal::Funding => people::Refusal::Authority { proposable: None, lacking: people::Lack::Funding },
        tasks::Refusal::AuthorityShape => people::Refusal::Authority { proposable: None, lacking: people::Lack::Shape },
        tasks::Refusal::Duplicate => people::Refusal::Limit { which: people::LimitKind::Duplicate },
        tasks::Refusal::Empty => people::Refusal::Limit { which: people::LimitKind::Empty },
        tasks::Refusal::Batch => people::Refusal::Limit { which: people::LimitKind::Batch },
        tasks::Refusal::Live => people::Refusal::Limit { which: people::LimitKind::LiveTasks },
        tasks::Refusal::Project => people::Refusal::Limit { which: people::LimitKind::ProjectLiveTasks },
        tasks::Refusal::Tree => people::Refusal::Limit { which: people::LimitKind::Tree },
        tasks::Refusal::Depth => people::Refusal::Limit { which: people::LimitKind::Depth },
        tasks::Refusal::Delegates => people::Refusal::Limit { which: people::LimitKind::Delegates },
        tasks::Refusal::Subscription => people::Refusal::Limit { which: people::LimitKind::Subscription },
        tasks::Refusal::Dependencies => people::Refusal::Limit { which: people::LimitKind::Dependencies },
        tasks::Refusal::Cycle => people::Refusal::Limit { which: people::LimitKind::Cycle },
        tasks::Refusal::Spec => people::Refusal::Limit { which: people::LimitKind::Spec },
        tasks::Refusal::Contract => people::Refusal::Limit { which: people::LimitKind::Contract },
        tasks::Refusal::Inputs => people::Refusal::Limit { which: people::LimitKind::Inputs },
        tasks::Refusal::Attempt => people::Refusal::Limit { which: people::LimitKind::Attempts },
        tasks::Refusal::LiveDelegates => people::Refusal::Limit { which: people::LimitKind::LiveDelegates },
        tasks::Refusal::Restore => people::Refusal::Limit { which: people::LimitKind::Restore },
        tasks::Refusal::Read => people::Refusal::Limit { which: people::LimitKind::Read },
        tasks::Refusal::Turn => people::Refusal::Limit { which: people::LimitKind::Turn },
        tasks::Refusal::HoldKind => people::Refusal::Limit { which: people::LimitKind::HoldKind },
        tasks::Refusal::HoldTaken => people::Refusal::Limit { which: people::LimitKind::HoldTaken },
        tasks::Refusal::Holds => people::Refusal::Limit { which: people::LimitKind::Holds },
    };
    people::Outcome::Refused(refusal)
}

fn current_task(core: &Core, task: Option<u64>) -> Option<&tasks::TaskRecord> {
    match task {
        Some(task) => core.tasks.task(task),
        None => None,
    }
}

pub(crate) fn ended(ending: &tasks::Ending) -> people::Refusal {
    let how = match ending {
        tasks::Ending::Done(_) => people::Ending::Done,
        tasks::Ending::Failed { .. } => people::Ending::Failed,
        tasks::Ending::Cancelled { by, at, .. } => people::Ending::Cancelled { by: party(*by), at: *at },
    };
    people::Refusal::Ended { how }
}

fn party(by: tasks::Party) -> people::Party {
    match by {
        tasks::Party::Person(person) => people::Party::Person(person),
        tasks::Party::Task(task) => people::Party::Task(task),
        tasks::Party::Deployment { project } => people::Party::Deployment { project },
    }
}

pub(super) fn pool_refused(work: &mut Queue<Event>, request: Token, why: people::Refusal) {
    work.push(Event::People(people::Event::Decided { request, outcome: people::Outcome::Refused(why) }));
}

pub(super) fn pool_again(work: &mut Queue<Event>, request: Token, why: people::Again) {
    work.push(Event::People(people::Event::Decided { request, outcome: people::Outcome::Again(why) }));
}

pub(super) fn note_person_refusal(why: notes::Refusal) -> people::Outcome {
    let refusal = match why {
        notes::Refusal::Busy => return people::Outcome::Again(people::Again::Busy),
        notes::Refusal::Oversized => people::Refusal::Limit { which: people::LimitKind::Words },
        notes::Refusal::Full => people::Refusal::Limit { which: people::LimitKind::Notes },
        notes::Refusal::RevisionExhausted => people::Refusal::Limit { which: people::LimitKind::Revision },
        notes::Refusal::Missing => people::Refusal::Ended { how: people::Ending::Deleted },
        notes::Refusal::Moved { revision } => people::Refusal::Moved { now: people::Moved::Note { revision } },
        notes::Refusal::Exists => people::Refusal::Limit { which: people::LimitKind::Duplicate },
    };
    people::Outcome::Refused(refusal)
}
