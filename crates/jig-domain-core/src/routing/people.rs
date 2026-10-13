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
use jig_domain_people as people;
use skein_lib::{Env, Queue, ReplyTo};

pub(super) fn route_people(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    event: people::Event,
) -> Requests {
    if event == people::Event::Restored {
        let stage = core.restart_step();
        if stage == Some(crate::RestartStep::LoadCore) || stage == Some(crate::RestartStep::LoadInbox) {
            core.counters.finish_restore();
            if core.counters.dirty() && !core.people.ready() {
                return super::first_start::seed(core, env, work);
            }
        }
        core.seed_owners = Box::new([]);
    }
    if let people::Event::ForgetChat { task, .. } = &event {
        drop(core.archived_chats.remove(task));
    }
    let event = match event {
        people::Event::Decided { request, outcome } => {
            if let Some(flight) = core.applications.get_mut(&request)
                && flight.part
            {
                flight.part = false;
                people::application_part_done(&mut core.people, request);
                return super::application_routes::answer(flight.connector, request, outcome);
            }
            people::Event::Decided { request, outcome }
        }
        other @ (people::Event::ChatState { .. }
        | people::Event::ChatSeen { .. }
        | people::Event::ForgetChat { .. }
        | people::Event::MakeService { .. }
        | people::Event::Seed { .. }
        | people::Event::Waiting { .. }
        | people::Event::ApplyRoles { .. }
        | people::Event::SignedIn { .. }
        | people::Event::SignOut { .. }
        | people::Event::Roles { .. }
        | people::Event::Ask { .. }
        | people::Event::Restore { .. }
        | people::Event::Restored) => other,
    };
    super::view_routes::touch_people_event(core, &env.limits, &event);
    let mut child = Queue::with_capacity(people::max_out(&env.limits.people));
    people::step(&mut core.people, &Env { now: env.now, wall: env.wall, limits: env.limits.people }, event, &mut child);
    tag_people(core, env, child, work)
}

pub(super) fn sign_in(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    reply_to: ReplyTo,
    identity: people::Identity,
) -> Requests {
    let mut out = Queue::with_capacity(2);
    if !core.people.ready() {
        out.push(Request::Held(Box::new(Held::PeopleReply {
            to: reply_to,
            reply: people::Reply::Again(people::Again::NotReady),
        })));
    } else if core.fill >= env.limits.fill_mark
        || core.counters.deployment().people == u64::MAX
        || core.counters.deployment().sign_ins == u64::MAX
    {
        out.push(Request::Now(Box::new(Now::SignInRefused { to: reply_to })));
    } else {
        let person = crate::fresh(&mut core.counters, crate::Family::Person).expect("person counter available");
        let sign_in = crate::fresh(&mut core.counters, crate::Family::SignIn).expect("sign-in counter available");
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
    let room = people::max_out(&env.limits.people)
        .checked_mul(4)
        .expect("bounded party routes")
        .checked_add(env.limits.people.inbox_tasks)
        .expect("bounded party routes");
    let mut pending = Queue::with_capacity(room);
    for _ in 0..child.len() {
        pending.push(child.pop().expect("party output count"));
    }
    let mut out = Queue::with_capacity(room.checked_add(1).expect("party output room"));
    for _ in 0..room {
        let Some(next) = pending.pop() else { break };
        let request = match next {
            people::Request::ServiceMade { request, outcome } => {
                work.push(Event::People(people::Event::Decided { request, outcome }));
                continue;
            }
            people::Request::Save { record } => {
                super::view_routes::touch_people_row(core, &record);
                Request::Write(Write::Save(Record::People(record)))
            }
            people::Request::Erase { key } => {
                match key {
                    people::Key::Answer(_) => core.touch_deployment(),
                    people::Key::Person(_)
                    | people::Key::ReadResults { .. }
                    | people::Key::ReadChat { .. }
                    | people::Key::SignIn(_)
                    | people::Key::Roles(_)
                    | people::Key::Policy(_) => {}
                }
                Request::Write(Write::Erase(Key::People(key)))
            }
            people::Request::Reply { to, reply } => match reply {
                reply @ (people::Reply::Again(people::Again::Busy)
                | people::Reply::Outcome(people::Outcome::Again(people::Again::Busy))) => {
                    core.touch_deployment();
                    Request::Held(Box::new(Held::PeopleReply { to, reply }))
                }
                people::Reply::Outcome(people::Outcome::Applied { connector, token }) => {
                    Request::Held(Box::new(Held::Applied { to, connector, token }))
                }
                reply @ (people::Reply::SignedIn { .. }
                | people::Reply::SignedOut
                | people::Reply::Again(..)
                | people::Reply::Refused(..)
                | people::Reply::Outcome(
                    people::Outcome::Again(..)
                    | people::Outcome::SignedOut
                    | people::Outcome::MarkedSeen { .. }
                    | people::Outcome::NoteEdited { .. }
                    | people::Outcome::ServiceMade { .. }
                    | people::Outcome::Adopted { .. }
                    | people::Outcome::GoalStarted { .. }
                    | people::Outcome::GoalProposed { .. }
                    | people::Outcome::Closing { .. }
                    | people::Outcome::Stopped { .. }
                    | people::Outcome::Cancelled { .. }
                    | people::Outcome::Released { .. }
                    | people::Outcome::PersonTaken { .. }
                    | people::Outcome::PersonHandedBack { .. }
                    | people::Outcome::PersonAnswered { .. }
                    | people::Outcome::Moved { .. }
                    | people::Outcome::ProposalDecided { .. }
                    | people::Outcome::Said { .. }
                    | people::Outcome::QuestionAnswered { .. }
                    | people::Outcome::Prioritised { .. }
                    | people::Outcome::Amended { .. }
                    | people::Outcome::AmendProposed { .. }
                    | people::Outcome::RolesSet { .. }
                    | people::Outcome::PolicyChanged { .. }
                    | people::Outcome::PoolSet { .. }
                    | people::Outcome::EscalationDecided { .. }
                    | people::Outcome::Started { .. }
                    | people::Outcome::Refused(..),
                )) => Request::Held(Box::new(Held::PeopleReply { to, reply })),
            },
            people::Request::RolesApplied { .. } => continue,
            people::Request::RolesRefused { .. } | people::Request::RestoreRefused { .. } => {
                Request::Now(Box::new(Now::RestoreRefused))
            }
            people::Request::Route { request, person, project, role, ask } => {
                if let Err(why) = crate::acts::request(core, &env.limits, person, project, role, &ask) {
                    work.push(Event::People(people::Event::Decided {
                        request,
                        outcome: people::Outcome::Refused(why),
                    }));
                    continue;
                }
                match *ask {
                    people::Ask::Application { connector, .. } => {
                        super::application_routes::admitted(
                            core,
                            env,
                            work,
                            &mut out,
                            request,
                            connector,
                            person,
                            project,
                            role.expect("application membership admitted"),
                        );
                    }
                    ask @ (people::Ask::TakePerson { .. }
                    | people::Ask::HandBackPerson { .. }
                    | people::Ask::AnswerPerson { .. }) => {
                        person_task_request(core, work, &mut out, request, person, project, role, ask);
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
                    people::Ask::Amend { task, revision, amendment, release, .. } => {
                        amend_route(
                            core, env, work, request, person, role, project, task, revision, amendment, release,
                        );
                    }
                    people::Ask::ChangePolicy { change, .. } => {
                        change_policy_request(core, env, work, request, person, project, &mut out, change);
                    }
                    people::Ask::DecideEscalation { task, revision, decision, .. } => escalation_request(
                        core, env, work, request, person, project, role, &mut out, task, revision, decision,
                    ),
                    people::Ask::SetGoal { spec, executor, budget, priority, deadline, .. } => begin_goal_creation(
                        core,
                        env,
                        &mut out,
                        work,
                        request,
                        person,
                        project,
                        role.expect("goal membership admitted"),
                        spec,
                        executor,
                        budget,
                        priority,
                        deadline,
                    ),
                    people::Ask::MarkSeen { seen } => match seen {
                        people::Seen::Results { .. } => {
                            work.push(Event::People(people::Event::Decided {
                                request,
                                outcome: people::Outcome::MarkedSeen { seen },
                            }));
                            for (task, _) in core.people.marked_chats(person).into_boxed() {
                                if core.tasks.task(task).is_none() {
                                    out.push(Request::Now(Box::new(Now::LoadEndedChat { task })));
                                }
                            }
                        }
                        people::Seen::Chat { task, .. } => match core.tasks.task(task) {
                            Some(row) => {
                                if core.chat_requester(row) == Some(person)
                                    && core.people.role(person, row.project).is_some()
                                {
                                    work.push(Event::People(people::Event::ChatSeen {
                                        project: row.project,
                                        request,
                                        last_reply: row.last_reply,
                                    }));
                                } else {
                                    work.push(Event::People(people::Event::Decided {
                                        request,
                                        outcome: people::Outcome::Refused(people::Refusal::Role {
                                            held: core.people.role(person, row.project),
                                        }),
                                    }));
                                }
                            }
                            None => out.push(Request::Now(Box::new(Now::SeenChat { request, task }))),
                        },
                    },
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
                    people::Ask::Say { task, words, release, .. } => {
                        say_request(core, env, work, request, person, project, task, words, release);
                    }
                    people::Ask::AnswerQuestion { task, question, words, .. } => {
                        answer_question_request(core, env, work, request, person, project, task, question, words);
                    }
                    people::Ask::Prioritise { goals, .. } => {
                        prioritise_request(core, env, work, request, person, project, role, goals);
                    }
                    people::Ask::Close { chat, disposals, .. } => {
                        crate::chats::route(core, env, work, request, person, project, role, chat, disposals);
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
