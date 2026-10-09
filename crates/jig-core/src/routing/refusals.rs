//! Child task and note reasons translated into party refusals
//! (`domain/people.md`, section 5.1; `domain/engine.md`, section 4.4).

use crate::Event;
use jig_core_notes as notes;
use jig_core_people as people;
use jig_core_tasks as tasks;
use skein_lib::{Queue, Token};

pub(super) fn person_task_refusal(why: tasks::Refusal) -> people::Refusal {
    match why {
        tasks::Refusal::Busy | tasks::Refusal::NotReady => people::Refusal::Busy,
        tasks::Refusal::Unknown => people::Refusal::Ended,
        tasks::Refusal::ResourceUnavailable => people::Refusal::ResourceUnavailable,
        tasks::Refusal::State | tasks::Refusal::Executor | tasks::Refusal::Reference => people::Refusal::Standing,
        tasks::Refusal::Funding | tasks::Refusal::AuthorityShape => people::Refusal::Authority,
        tasks::Refusal::Duplicate
        | tasks::Refusal::Empty
        | tasks::Refusal::Batch
        | tasks::Refusal::Live
        | tasks::Refusal::Project
        | tasks::Refusal::Tree
        | tasks::Refusal::Depth
        | tasks::Refusal::Delegates
        | tasks::Refusal::Subscription
        | tasks::Refusal::Dependencies
        | tasks::Refusal::Cycle
        | tasks::Refusal::Spec
        | tasks::Refusal::Contract
        | tasks::Refusal::Inputs
        | tasks::Refusal::Attempt
        | tasks::Refusal::LiveDelegates
        | tasks::Refusal::Restore
        | tasks::Refusal::Read
        | tasks::Refusal::Turn
        | tasks::Refusal::HoldKind
        | tasks::Refusal::HoldTaken
        | tasks::Refusal::Holds => people::Refusal::Limit,
    }
}

pub(super) fn pool_refused(work: &mut Queue<Event>, request: Token, why: people::Refusal) {
    work.push(Event::People(people::Event::Decided { request, outcome: people::Outcome::Refused(why) }));
}

pub(super) fn person_move_refusal(why: tasks::Refusal) -> people::Refusal {
    match why {
        tasks::Refusal::Busy | tasks::Refusal::NotReady => people::Refusal::Busy,
        tasks::Refusal::Unknown => people::Refusal::Ended,
        tasks::Refusal::ResourceUnavailable => people::Refusal::ResourceUnavailable,
        tasks::Refusal::Reference | tasks::Refusal::State => people::Refusal::Standing,
        tasks::Refusal::Funding => people::Refusal::Authority,
        tasks::Refusal::Duplicate
        | tasks::Refusal::Empty
        | tasks::Refusal::Batch
        | tasks::Refusal::Live
        | tasks::Refusal::Project
        | tasks::Refusal::Tree
        | tasks::Refusal::Depth
        | tasks::Refusal::Delegates
        | tasks::Refusal::Subscription
        | tasks::Refusal::Dependencies
        | tasks::Refusal::Cycle
        | tasks::Refusal::Executor
        | tasks::Refusal::Spec
        | tasks::Refusal::Contract
        | tasks::Refusal::AuthorityShape
        | tasks::Refusal::Inputs
        | tasks::Refusal::Attempt
        | tasks::Refusal::LiveDelegates
        | tasks::Refusal::Restore
        | tasks::Refusal::Read
        | tasks::Refusal::Turn
        | tasks::Refusal::HoldKind
        | tasks::Refusal::HoldTaken
        | tasks::Refusal::Holds => people::Refusal::Limit,
    }
}

pub(super) fn person_message_refusal(why: tasks::Refusal) -> people::Refusal {
    match why {
        tasks::Refusal::Busy | tasks::Refusal::NotReady => people::Refusal::Busy,
        tasks::Refusal::Unknown => people::Refusal::Unknown,
        tasks::Refusal::ResourceUnavailable => people::Refusal::ResourceUnavailable,
        tasks::Refusal::State => people::Refusal::Standing,
        tasks::Refusal::Duplicate
        | tasks::Refusal::Empty
        | tasks::Refusal::Batch
        | tasks::Refusal::Live
        | tasks::Refusal::Project
        | tasks::Refusal::Tree
        | tasks::Refusal::Depth
        | tasks::Refusal::Delegates
        | tasks::Refusal::Reference
        | tasks::Refusal::Subscription
        | tasks::Refusal::Dependencies
        | tasks::Refusal::Cycle
        | tasks::Refusal::Executor
        | tasks::Refusal::Spec
        | tasks::Refusal::Contract
        | tasks::Refusal::AuthorityShape
        | tasks::Refusal::Inputs
        | tasks::Refusal::Attempt
        | tasks::Refusal::LiveDelegates
        | tasks::Refusal::Restore
        | tasks::Refusal::Read
        | tasks::Refusal::Turn
        | tasks::Refusal::Funding
        | tasks::Refusal::HoldKind
        | tasks::Refusal::HoldTaken
        | tasks::Refusal::Holds => people::Refusal::Limit,
    }
}

pub(super) fn note_person_refusal(why: notes::Refusal) -> people::Refusal {
    match why {
        notes::Refusal::Busy => people::Refusal::Busy,
        notes::Refusal::Oversized | notes::Refusal::Full | notes::Refusal::RevisionExhausted => people::Refusal::Limit,
        notes::Refusal::Missing => people::Refusal::NoteMissing,
        notes::Refusal::Moved => people::Refusal::NoteMoved,
        notes::Refusal::Exists => people::Refusal::Unknown,
    }
}
