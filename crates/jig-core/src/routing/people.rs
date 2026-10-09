//! People-child outputs translated to durable replies and one exhaustive party
//! request dispatcher (`domain/people.md`, section 5.1). The former chain's
//! observable precedence remains in `party_decisions`: person-origin goals
//! before live task proposals before history; current escalation revisions
//! before history. Other families select disjoint variants.

use super::note_routes::edit_note_request;
use super::parties::{
    adopt_request, begin_chat_creation, begin_goal_creation, change_policy_request, make_service_request, pool_route,
    roles_route,
};
use super::party_controls::{
    amend_route, answer_question_request, cancel_request, move_request, person_task_request, prioritise_request,
    release_request, say_request, stop_request,
};
use super::party_decisions::{escalation_request, proposal_request};
use crate::{Core, Event, Held, Key, Limits, Now, Record, Request, Requests, Write};
use alloc::boxed::Box;
use jig_core_people as people;
use skein_lib::{Env, Queue, ReplyTo};

pub(super) fn route_people(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    event: people::Event,
) -> Requests {
    super::view_routes::touch_people_event(core, &env.limits, &event);
    let mut child = Queue::with_capacity(people::max_out(&env.limits.people));
    people::step(&mut core.people, &Env { now: env.now, wall: env.wall, limits: env.limits.people }, event, &mut child);
    tag_people(core, env, child, work)
}

pub(super) fn sign_in(
    core: &mut Core,
    work: &mut Queue<Event>,
    reply_to: ReplyTo,
    identity: people::Identity,
) -> Requests {
    let mut out = Queue::with_capacity(2);
    if core.counters.deployment().people == u64::MAX || core.counters.deployment().sign_ins == u64::MAX {
        out.push(Request::Now(Box::new(Now::SignInRefused { to: reply_to })));
    } else {
        let person = crate::fresh(&mut core.counters, crate::Family::Person).expect("person counter available");
        let sign_in = crate::fresh(&mut core.counters, crate::Family::SignIn).expect("sign-in counter available");
        core.signing_in = Some(sign_in);
        let kind = if identity.key.provider == core.settings.deployment_provider {
            people::Kind::Service
        } else {
            people::Kind::Person
        };
        work.push(Event::People(people::Event::SignedIn { reply_to, person, sign_in, identity, kind }));
    }
    out.push(Request::Decided);
    Requests::Out(out)
}
#[expect(clippy::too_many_lines, reason = "one exhaustive party vocabulary routes its bounded child decision")]
pub(super) fn tag_people(
    core: &mut Core,
    env: &Env<Limits>,
    mut child: Queue<people::Request>,
    work: &mut Queue<Event>,
) -> Requests {
    let room = people::max_out(&env.limits.people).checked_mul(4).expect("bounded party routes");
    let mut pending = Queue::with_capacity(room);
    for _ in 0..child.len() {
        pending.push(child.pop().expect("party output count"));
    }
    let mut out = Queue::with_capacity(room.checked_add(1).expect("party output room"));
    for _ in 0..room {
        let Some(next) = pending.pop() else { break };
        let request = match next {
            people::Request::ServiceMade { request, outcome } => {
                let mut follow = Queue::with_capacity(people::max_out(&env.limits.people));
                people::step(
                    &mut core.people,
                    &Env { now: env.now, wall: env.wall, limits: env.limits.people },
                    people::Event::Decided { request, outcome },
                    &mut follow,
                );
                for _ in 0..follow.len() {
                    pending.push(follow.pop().expect("service output count"));
                }
                continue;
            }
            people::Request::Save { record } => {
                super::view_routes::touch_people_row(core, &record);
                Request::Write(Write::Save(Record::People(record)))
            }
            people::Request::Erase { key } => Request::Write(Write::Erase(Key::People(key))),
            people::Request::Reply { to, reply } => {
                Request::Held(Box::new(Held::PeopleReply { to, sign_in: core.signing_in, reply }))
            }
            people::Request::RolesApplied { .. } => continue,
            people::Request::RolesRefused { .. } | people::Request::RestoreRefused { .. } => {
                Request::Now(Box::new(Now::RestoreRefused))
            }
            people::Request::Route { request, person, project, role, ask } => {
                match *ask {
                    people::Ask::Application { .. } => {
                        work.push(Event::People(people::Event::Decided {
                            request,
                            outcome: people::Outcome::Refused(people::Refusal::NotOffered),
                        }));
                    }
                    ask @ (people::Ask::TakePerson { .. }
                    | people::Ask::HandBackPerson { .. }
                    | people::Ask::AnswerPerson { .. }) => {
                        person_task_request(core, work, request, person, project, role, ask);
                    }
                    ask @ people::Ask::Stop { .. } => {
                        stop_request(core, env, work, request, person, project, role, ask);
                    }
                    ask @ people::Ask::Cancel { .. } => {
                        cancel_request(core, env, work, request, person, project, role, ask);
                    }
                    ask @ people::Ask::Release { .. } => {
                        release_request(core, env, work, request, person, project, role, ask);
                    }
                    people::Ask::MakeService { role: service_role, .. } => {
                        make_service_request(core, env, request, person, project, &mut pending, service_role);
                    }
                    people::Ask::EditNote { name, scope, change, .. } => {
                        edit_note_request(core, work, request, person, project, role, name, scope, *change);
                    }
                    people::Ask::Adopt { adoption, .. } => {
                        adopt_request(core, env, work, request, person, project, &mut out, adoption);
                    }
                    people::Ask::SetPool { person: beneficiary, budget, .. } => {
                        pool_route(core, env, work, request, person, project, beneficiary, budget);
                    }
                    people::Ask::DecideProposal { proposer, proposal, decision, .. } => proposal_request(
                        core, env, work, request, person, project, role, &mut out, proposer, proposal, decision,
                    ),
                    people::Ask::SetRoles { holdings, .. } => {
                        roles_route(core, env, work, &mut out, request, person, project, holdings);
                    }
                    people::Ask::Amend { task, revision, amendment, .. } => {
                        amend_route(core, env, work, request, person, role, project, task, revision, amendment);
                    }
                    people::Ask::ChangePolicy { change, .. } => {
                        change_policy_request(core, env, work, request, person, project, &mut out, change);
                    }
                    people::Ask::DecideEscalation { task, revision, decision, .. } => escalation_request(
                        core, env, work, request, person, project, role, &mut out, task, revision, decision,
                    ),
                    people::Ask::SetGoal { spec, charter, budget, priority, .. } => begin_goal_creation(
                        core,
                        env,
                        &mut out,
                        work,
                        request,
                        person,
                        project,
                        role.expect("goal membership admitted"),
                        spec,
                        charter,
                        budget,
                        priority,
                    ),
                    people::Ask::StartChat { words, .. } => begin_chat_creation(
                        core,
                        env,
                        &mut out,
                        work,
                        request,
                        person,
                        project,
                        role.expect("chat membership admitted"),
                        words,
                    ),
                    people::Ask::Say { task, words, .. } => {
                        say_request(core, env, work, request, person, project, task, words);
                    }
                    people::Ask::AnswerQuestion { task, question, words, .. } => {
                        answer_question_request(core, env, work, request, person, project, task, question, words);
                    }
                    people::Ask::Prioritise { goals, .. } => {
                        prioritise_request(core, env, work, request, person, project, role, goals);
                    }
                    people::Ask::Move { task, reason, .. } => {
                        move_request(core, env, work, request, person, project, role, task, reason);
                    }
                }
                continue;
            }
        };
        out.push(request);
    }
    assert!(pending.is_empty(), "finite party routes fit their bound");
    out.push(Request::Decided);
    Requests::Out(out)
}
