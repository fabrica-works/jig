//! Party task controls, amendments, messages, answers and moves routed to tasks
//! under current authority (`domain/people.md`, section 5.1).

use super::refusals::pool_refused;
use crate::{Core, Event, Limits, PersonTaskRoute};
use alloc::boxed::Box;
use jig_core_people as people;
use jig_core_tasks as tasks;
use skein_lib::{Env, Queue, ReplyTo, Token};

#[expect(clippy::too_many_arguments, reason = "a keyed amendment carries its person, project and task")]
pub(super) fn amend_route(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    role: Option<people::Role>,
    project: u32,
    task: u64,
    revision: u64,
    amendment: people::Amendment,
) {
    let Some(amendment) = crate::amendments::translate(amendment, &env.limits.tasks) else {
        pool_refused(work, request, people::Refusal::Limit { which: people::LimitKind::Words });
        return;
    };
    let (stop_run, propose) =
        match core.amend_admit(person, role, project, task, &amendment, env.limits.tasks.depth, &env.limits.tasks) {
            Ok(admitted) => admitted,
            Err(why) => {
                pool_refused(work, request, why);
                return;
            }
        };
    if let Err(why) = core.spec_revision(task, revision) {
        pool_refused(work, request, why);
        return;
    }
    if propose {
        let Some(proposal) = crate::fresh(&mut core.counters, crate::Family::Message) else {
            pool_refused(work, request, people::Refusal::Limit { which: people::LimitKind::Arithmetic });
            return;
        };
        assert!(
            core.person_tasks.insert(request, PersonTaskRoute::AmendProposed { task, proposal }) == Ok(None),
            "one amendment proposal"
        );
        work.push(Event::Tasks(tasks::Event::Propose {
            reply_to: ReplyTo::new(request),
            proposal: tasks::Proposal {
                number: proposal,
                proposer: task,
                project,
                reason: amendment.reason.clone(),
                as_holder: false,
                action: tasks::ProposalAction::Amend { task, amendment },
                state: tasks::ProposalState::Pending {
                    holder: tasks::ProposalHolder::Policy { project, kind: tasks::ProposalKind::Amend },
                    since: env.wall,
                },
            },
        }));
        return;
    }
    let Some(message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
        pool_refused(work, request, people::Refusal::Limit { which: people::LimitKind::Arithmetic });
        return;
    };
    assert!(core.person_tasks.insert(request, PersonTaskRoute::Amended(task)) == Ok(None), "one person amendment");
    work.push(Event::Tasks(tasks::Event::Amend {
        reply_to: ReplyTo::new(request),
        by: tasks::Party::Person(person),
        task,
        message,
        stop_run,
        amendment,
    }));
}

pub(super) fn person_task_request(
    core: &mut Core,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    project: u32,
    role: Option<people::Role>,
    ask: people::Ask,
) {
    match core.person_task(request, person, project, role, ask) {
        Ok(event) => work.push(Event::Tasks(event)),
        Err(why) => {
            work.push(Event::People(people::Event::Decided { request, outcome: people::Outcome::Refused(why) }));
        }
    }
}

#[expect(clippy::too_many_arguments, reason = "the route retains the admitted party and request fields")]
pub(super) fn stop_request(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    project: u32,
    role: Option<people::Role>,
    ask: people::Ask,
) {
    match core.control_admit(&ask, person, project, role, env.limits.tasks.depth, env.limits.tasks.result_bytes) {
        Ok(task) => {
            work.push(Event::Tasks(tasks::Event::Hold { task, why: tasks::Hold::StoppedBy { party: person } }));
            work.push(Event::People(people::Event::Decided { request, outcome: people::Outcome::Stopped { task } }));
        }
        Err(why) => {
            work.push(Event::People(people::Event::Decided { request, outcome: people::Outcome::Refused(why) }));
        }
    }
}

#[expect(clippy::too_many_arguments, reason = "the route retains the admitted party and request fields")]
pub(super) fn cancel_request(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    project: u32,
    role: Option<people::Role>,
    ask: people::Ask,
) {
    let admitted =
        core.control_admit(&ask, person, project, role, env.limits.tasks.depth, env.limits.tasks.result_bytes);
    let reason = match ask {
        people::Ask::Cancel { reason, .. } => reason,
        people::Ask::Application { .. }
        | people::Ask::EditNote { .. }
        | people::Ask::MakeService { .. }
        | people::Ask::Adopt { .. }
        | people::Ask::SetGoal { .. }
        | people::Ask::Stop { .. }
        | people::Ask::Release { .. }
        | people::Ask::TakePerson { .. }
        | people::Ask::HandBackPerson { .. }
        | people::Ask::AnswerPerson { .. }
        | people::Ask::DecideProposal { .. }
        | people::Ask::Amend { .. }
        | people::Ask::SetRoles { .. }
        | people::Ask::ChangePolicy { .. }
        | people::Ask::SetPool { .. }
        | people::Ask::DecideEscalation { .. }
        | people::Ask::StartChat { .. }
        | people::Ask::Say { .. }
        | people::Ask::AnswerQuestion { .. }
        | people::Ask::Prioritise { .. }
        | people::Ask::Move { .. } => unreachable!("cancel route retains its ask"),
    };
    match admitted {
        Ok(task) => {
            assert!(core.person_tasks.insert(request, PersonTaskRoute::Cancel(task)) == Ok(None), "one cancel route");
            work.push(Event::Tasks(tasks::Event::Control {
                reply_to: ReplyTo::new(request),
                by: tasks::Party::Person(person),
                task,
                control: tasks::Control::Cancel { reason },
            }));
        }
        Err(why) => {
            work.push(Event::People(people::Event::Decided { request, outcome: people::Outcome::Refused(why) }));
        }
    }
}

#[expect(clippy::too_many_arguments, reason = "the route retains the admitted party and request fields")]
pub(super) fn release_request(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    project: u32,
    role: Option<people::Role>,
    ask: people::Ask,
) {
    match core.control_admit(&ask, person, project, role, env.limits.tasks.depth, env.limits.tasks.result_bytes) {
        Ok(task) => {
            let (revision, again) = match ask {
                people::Ask::Release { revision, again, .. } => (revision, again),
                people::Ask::Application { .. }
                | people::Ask::EditNote { .. }
                | people::Ask::MakeService { .. }
                | people::Ask::Adopt { .. }
                | people::Ask::SetGoal { .. }
                | people::Ask::Stop { .. }
                | people::Ask::Cancel { .. }
                | people::Ask::TakePerson { .. }
                | people::Ask::HandBackPerson { .. }
                | people::Ask::AnswerPerson { .. }
                | people::Ask::Move { .. }
                | people::Ask::DecideProposal { .. }
                | people::Ask::Say { .. }
                | people::Ask::AnswerQuestion { .. }
                | people::Ask::Prioritise { .. }
                | people::Ask::Amend { .. }
                | people::Ask::SetRoles { .. }
                | people::Ask::ChangePolicy { .. }
                | people::Ask::SetPool { .. }
                | people::Ask::DecideEscalation { .. }
                | people::Ask::StartChat { .. } => unreachable!("release route owns its ask"),
            };
            if let Err(why) = core.hold_revision(task, revision) {
                pool_refused(work, request, why);
                return;
            }
            let required = match core.tasks.task(task).expect("release task admitted").phase {
                tasks::Phase::Held { why: tasks::Hold::Uncertain { .. }, .. } => true,
                tasks::Phase::Held { .. }
                | tasks::Phase::Waiting
                | tasks::Phase::Active(_)
                | tasks::Phase::Closing(_)
                | tasks::Phase::Ended(_) => false,
            };
            if required != again.is_some() {
                pool_refused(work, request, people::Refusal::Authority { lacking: people::Lack::Again { required } });
                return;
            }
            let message = if again == Some(false) {
                let Some(message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                    pool_refused(work, request, people::Refusal::Limit { which: people::LimitKind::Arithmetic });
                    return;
                };
                Some(message)
            } else {
                None
            };
            assert!(core.person_tasks.insert(request, PersonTaskRoute::Release(task)) == Ok(None), "one release route");
            work.push(Event::Tasks(tasks::Event::Control {
                reply_to: ReplyTo::new(request),
                by: tasks::Party::Person(person),
                task,
                control: tasks::Control::Release { again, message },
            }));
        }
        Err(why) => {
            work.push(Event::People(people::Event::Decided { request, outcome: people::Outcome::Refused(why) }));
        }
    }
}

#[expect(clippy::too_many_arguments, reason = "the route retains the admitted party and request fields")]
pub(super) fn say_request(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    project: u32,
    task: u64,
    words: Box<[u8]>,
    release: bool,
) {
    match core.person_message(crate::PersonMessage {
        request,
        person,
        project,
        task,
        question: None,
        release,
        words,
        at: env.wall,
    }) {
        Ok(event) => work.push(Event::Tasks(event)),
        Err(why) => {
            work.push(Event::People(people::Event::Decided { request, outcome: people::Outcome::Refused(why) }));
        }
    }
}

#[expect(clippy::too_many_arguments, reason = "the route retains the admitted party and request fields")]
pub(super) fn answer_question_request(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    project: u32,
    task: u64,
    question: u64,
    words: Box<[u8]>,
) {
    match core.person_message(crate::PersonMessage {
        request,
        person,
        project,
        task,
        question: Some(question),
        release: false,
        words,
        at: env.wall,
    }) {
        Ok(event) => work.push(Event::Tasks(event)),
        Err(why) => {
            work.push(Event::People(people::Event::Decided { request, outcome: people::Outcome::Refused(why) }));
        }
    }
}

#[expect(clippy::too_many_arguments, reason = "the route retains the admitted party and request fields")]
pub(super) fn prioritise_request(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    project: u32,
    role: Option<people::Role>,
    goals: Box<[(u64, u32)]>,
) {
    match core.prioritise(request, person, project, role, goals, env.limits.tasks.tasks) {
        Ok(event) => work.push(Event::Tasks(event)),
        Err(why) => {
            work.push(Event::People(people::Event::Decided { request, outcome: people::Outcome::Refused(why) }));
        }
    }
}

#[expect(clippy::too_many_arguments, reason = "the route retains the admitted party and request fields")]
pub(super) fn move_request(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    project: u32,
    role: Option<people::Role>,
    task: u64,
    reason: Box<[u8]>,
) {
    match core.move_admit(person, role, project, task, env.limits.tasks.tree_tasks, env.limits.tasks.depth) {
        Ok(()) => {
            assert!(core.moving.insert(request, task) == Ok(None), "one move flight per keyed request");
            work.push(Event::Tasks(tasks::Event::Move {
                reply_to: ReplyTo::new(request),
                task,
                person,
                period: core.settings.period,
                pool_budget: core.settings.person_budget,
                period_budget: core.settings.period_budget,
                reason,
            }));
        }
        Err(why) => {
            work.push(Event::People(people::Event::Decided { request, outcome: people::Outcome::Refused(why) }));
        }
    }
}
