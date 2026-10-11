//! Subject marks follow this owner's emitted state changes, never a root's guesses.
use crate::{ClientSubject as Subject, Domain, Limits, Record, RecordKey, Request};
use skein_lib::{Env, List, Queue};

fn panel(subject: &Subject, task: u64) -> bool {
    match subject {
        Subject::Panel { task: named, .. } => *named == task,
        Subject::Object { .. }
        | Subject::Result { .. }
        | Subject::Effect { .. }
        | Subject::Outbox { .. }
        | Subject::Hold { .. }
        | Subject::Project { .. }
        | Subject::Deployment => false,
    }
}

fn result(subject: &Subject, task: u64) -> bool {
    match subject {
        Subject::Panel { task: named, .. } | Subject::Result { task: named, .. } => *named == task,
        Subject::Object { .. }
        | Subject::Effect { .. }
        | Subject::Outbox { .. }
        | Subject::Hold { .. }
        | Subject::Project { .. }
        | Subject::Deployment => false,
    }
}

fn task(domain: &Domain, subject: &Subject, number: u64) -> bool {
    match subject {
        Subject::Panel { task, .. } | Subject::Result { task, .. } | Subject::Hold { task, .. } => *task == number,
        Subject::Effect { proposal, .. } => match domain.outbox.proposals.get(proposal) {
            Some((_, task, _)) => *task == number,
            None => false,
        },
        Subject::Object { .. } | Subject::Project { .. } | Subject::Deployment => false,
        Subject::Outbox { .. } => true,
    }
}

fn object(subject: &Subject, number: u64) -> bool {
    match subject {
        Subject::Object { object, .. } => *object == number,
        Subject::Panel { .. }
        | Subject::Result { .. }
        | Subject::Effect { .. }
        | Subject::Outbox { .. }
        | Subject::Hold { .. }
        | Subject::Project { .. }
        | Subject::Deployment => false,
    }
}

fn proposal(subject: &Subject, number: u64) -> bool {
    match subject {
        Subject::Effect { proposal, .. } => *proposal == number,
        Subject::Object { .. }
        | Subject::Panel { .. }
        | Subject::Result { .. }
        | Subject::Hold { .. }
        | Subject::Outbox { .. }
        | Subject::Project { .. }
        | Subject::Deployment => false,
    }
}

fn outbox(subject: &Subject, task: u64) -> bool {
    match subject {
        Subject::Panel { task: named, .. } | Subject::Hold { task: named, .. } => *named == task,
        Subject::Deployment | Subject::Outbox { .. } => true,
        Subject::Object { .. } | Subject::Result { .. } | Subject::Effect { .. } | Subject::Project { .. } => false,
    }
}

fn resource(domain: &Domain, subject: &Subject, owner: Option<u32>, path: &crate::Path) -> bool {
    match subject {
        Subject::Object { project, object } => {
            (owner.is_none() || owner == Some(*project))
                && match domain.specification_resource(*object) {
                    Some(specification) => specification.path == *path,
                    None => false,
                }
        }
        Subject::Panel { project, task } | Subject::Hold { project, task } => {
            if owner.is_some() && owner != Some(*project) {
                return false;
            }
            match domain.tasks.get(task) {
                Some(Record::Task { resources, .. }) => resources.contains(path),
                Some(
                    Record::ApplicationObject { .. }
                    | Record::ApplicationAnswer { .. }
                    | Record::Proposal { .. }
                    | Record::Procedure(_)
                    | Record::ProcedureDelegateOutcome { .. }
                    | Record::Result { .. }
                    | Record::Adoption { .. }
                    | Record::Subscription { .. }
                    | Record::Pool { .. }
                    | Record::Outbox(_)
                    | Record::Made { .. },
                )
                | None => false,
            }
        }
        Subject::Project { project } => match owner {
            Some(owner) => owner == *project,
            None => domain.resource_role(*project, path).is_some(),
        },
        Subject::Result { .. } | Subject::Effect { .. } | Subject::Outbox { .. } | Subject::Deployment => false,
    }
}

fn saved(domain: &Domain, subject: &Subject, record: &Record) -> bool {
    match record {
        Record::ApplicationObject { object: number, .. } => object(subject, *number),
        Record::Procedure(state) => panel(subject, state.task),
        Record::Result { task, .. } => result(subject, *task),
        Record::Task { task: number, .. } => task(domain, subject, *number),
        Record::Adoption { project, path, .. } => resource(domain, subject, Some(*project), path),
        Record::Proposal { number, .. } => proposal(subject, *number),
        Record::Outbox(entry) => outbox(subject, entry.task),
        Record::ProcedureDelegateOutcome { .. }
        | Record::ApplicationAnswer { .. }
        | Record::Subscription { .. }
        | Record::Pool { .. }
        | Record::Made { .. } => false,
    }
}

fn erased(domain: &Domain, subject: &Subject, key: &RecordKey) -> bool {
    match key {
        RecordKey::ApplicationObject(number) => object(subject, *number),
        RecordKey::Procedure(number) => panel(subject, *number),
        RecordKey::Result(number) => result(subject, *number),
        RecordKey::Task(number) => task(domain, subject, *number),
        RecordKey::Adoption { project, path } => resource(domain, subject, Some(*project), path),
        RecordKey::Proposal(number) => proposal(subject, *number),
        RecordKey::Outbox(key) => outbox(subject, key.task),
        RecordKey::ProcedureDelegateOutcome(_)
        | RecordKey::ApplicationAnswer(_)
        | RecordKey::Subscription { .. }
        | RecordKey::Pool(_)
        | RecordKey::Made(_) => false,
    }
}

pub(crate) fn collect(domain: &Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if domain.client_subjects.is_empty() {
        return;
    }
    let mut subjects = List::with_capacity(env.limits.values);
    // Rotate the bounded output once, preserving its exact ordering and values.
    for _ in 0..out.len() {
        let request = out.pop().expect("the initial output count is fixed");
        for (token, subject) in &domain.client_subjects {
            let touched = match &request {
                Request::Save { record } => saved(domain, subject, record),
                Request::Erase { key } => erased(domain, subject, key),
                Request::Changed { resource: path } => resource(domain, subject, None, path),
                Request::Document { .. }
                | Request::ReadAnswered { .. }
                | Request::Marked { .. }
                | Request::ApplicationDone { .. }
                | Request::ApplicationAgain { .. }
                | Request::ApplicationRelease { .. }
                | Request::Resource { .. }
                | Request::Closed { .. }
                | Request::Verdict { .. }
                | Request::Step { .. }
                | Request::Answer { .. }
                | Request::Ready { .. }
                | Request::Section { .. }
                | Request::Workspace { .. }
                | Request::DriftResource { .. }
                | Request::RestartDone
                | Request::Described { .. }
                | Request::EffectRefused { .. }
                | Request::EffectBusy { .. }
                | Request::Make { .. }
                | Request::Outcome { .. }
                | Request::System(_)
                | Request::Named { .. }
                | Request::Unknown { .. }
                | Request::Refused { .. }
                | Request::Adopted { .. }
                | Request::Slots { .. }
                | Request::Drift { .. }
                | Request::News { .. } => false,
            };
            if touched && !subjects.as_slice().contains(token) {
                subjects.push(*token).expect("each bounded prepared subject is marked at most once");
            }
        }
        out.push(request);
    }
    if !subjects.is_empty() {
        out.push(Request::Marked { subjects });
    }
}
