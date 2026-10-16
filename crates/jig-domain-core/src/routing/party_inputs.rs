//! A party's historical inputs, loaded by the root and judged by the core.
use super::{
    CreationKind,
    parties::begin_creation,
    refusals::{pool_again, pool_refused},
};
use crate::{Core, Event, Limits, Now, Request};
use alloc::boxed::Box;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, Token};
#[derive(Debug)]
pub(crate) struct Inputs {
    pub person: u64,
    pub project: u32,
    pub role: people::Role,
    pub members: Box<[tasks::New]>,
    pub given: Option<people::Given>,
    pub ids: Box<[u64]>,
    pub at: usize,
    pub row: Option<Box<tasks::TaskRecord>>,
    pub stubs: List<tasks::Stub>,
}
pub(super) fn begin(
    core: &mut Core,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
    work: &mut Queue<Event>,
    mut flight: Inputs,
    request: Token,
) {
    let capacity = env.limits.people.batch.checked_mul(env.limits.tasks.inputs).expect("bounded party inputs");
    let mut ids = List::with_capacity(capacity);
    for member in &flight.members {
        for &task in &member.spec.inputs {
            if !ids.as_slice().contains(&task) {
                ids.push(task).expect("bounded member inputs");
            }
        }
    }
    if ids.is_empty() {
        begin_creation(
            core,
            env,
            out,
            work,
            request,
            CreationKind::Batch { person: flight.person, given: flight.given, stubs: Box::new([]) },
            flight.members,
        );
        return;
    }
    flight.ids = ids.into_boxed();
    flight.stubs = List::with_capacity(capacity);
    if core.creating_inputs.len() == core.creating_inputs.capacity() {
        pool_again(work, request, people::Again::Busy);
        return;
    }
    ask(out, request, &flight);
    if core.creating_inputs.insert(request, flight).is_err() {
        pool_again(work, request, people::Again::Busy);
    }
}
fn ask(out: &mut Queue<Request>, request: Token, flight: &Inputs) {
    let tasks: Box<[u64]> = match &flight.row {
        Some(row) => Box::new([row.root]),
        None => flight.ids.get(flight.at..).expect("remaining inputs").into(),
    };
    out.push(Request::Now(Box::new(Now::PartyInputs {
        request,
        person: flight.person,
        project: flight.project,
        tasks,
    })));
}
pub(super) struct Loaded {
    pub request: Token,
    pub row: Option<Box<tasks::TaskRecord>>,
    pub busy: bool,
}
pub(super) fn loaded(
    core: &mut Core,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
    work: &mut Queue<Event>,
    loaded: Loaded,
) {
    let Loaded { request, row, busy } = loaded;
    let Some(mut flight) = core.creating_inputs.remove(&request) else { return };
    if busy {
        pool_again(work, request, people::Again::Busy);
        return;
    }
    let Some(row) = row else {
        pool_refused(work, request, people::Refusal::Unknown);
        return;
    };
    let wanted = *flight.ids.get(flight.at).expect("pending input");
    let input = if let Some(input) = flight.row.take() {
        if row.number != input.root
            || row.project != flight.project
            || row.requester != tasks::Party::Person(flight.person)
        {
            pool_refused(work, request, people::Refusal::Unknown);
            return;
        }
        input
    } else {
        let ended = match row.phase {
            tasks::Phase::Ended(_) => true,
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
                false
            }
        };
        if row.number != wanted || row.project != flight.project || !ended {
            pool_refused(work, request, people::Refusal::Unknown);
            return;
        }
        if !core.reaches_project(flight.role, flight.project)
            && row.requester != tasks::Party::Person(flight.person)
            && !core.person_tree(flight.person, row.root, env.limits.tasks.depth)
        {
            if row.root == wanted || core.tasks.task(row.root).is_some() {
                pool_refused(work, request, people::Refusal::Unknown);
                return;
            }
            flight.row = Some(row);
            ask(out, request, &flight);
            assert!(core.creating_inputs.insert(request, flight).is_ok(), "continuing input load");
            return;
        }
        row
    };
    let phase = match input.phase {
        tasks::Phase::Ended(tasks::Ending::Done(_)) => tasks::Status::Done,
        tasks::Phase::Ended(tasks::Ending::Failed { .. }) => tasks::Status::Failed,
        tasks::Phase::Ended(tasks::Ending::Cancelled { .. }) => tasks::Status::Cancelled,
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
            unreachable!("ended input checked before its root")
        }
    };
    flight.stubs.push(tasks::Stub { task: wanted, phase, result: Token::new(wanted) }).expect("one stub per input");
    flight.at = flight.at.checked_add(1).expect("bounded input position");
    if flight.at < flight.ids.len() {
        ask(out, request, &flight);
        assert!(core.creating_inputs.insert(request, flight).is_ok(), "continuing input load");
    } else {
        begin_creation(
            core,
            env,
            out,
            work,
            request,
            CreationKind::Batch { person: flight.person, given: flight.given, stubs: flight.stubs.into_boxed() },
            flight.members,
        );
    }
}
