//! Connector-owned keyed requests and their synchronous party parts
//! (`domain/connectors.md`, section 5.5; `domain/core.md`, section 5).

use crate::{Ask, Core, Event, Family, Limits, Request, Requests, fresh};
use jig_domain_people as people;
use skein_lib::{Env, Queue, ReplyTo, Token};

#[derive(Debug)]
pub(crate) struct Flight {
    pub connector: u16,
    pub token: Token,
    pub part: bool,
}

#[expect(clippy::too_many_arguments, reason = "metadata envelope without an application value")]
pub(super) fn start(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    reply_to: ReplyTo,
    sign_in: u64,
    key: [u8; 16],
    scope: people::ApplicationScope,
    connector: u16,
    digest: [u8; 32],
    owner: Token,
) -> Requests {
    assert!(core.application_input.is_none(), "one admission at a time");
    core.application_input = Some((connector, owner));
    let Requests::Out(mut out) = super::people_routes::route_people(
        core,
        env,
        work,
        people::Event::Ask { reply_to, sign_in, key, ask: people::Ask::Application { scope, connector, digest } },
    );
    core.application_input = None;
    if core.connectors.contains(&connector) {
        out.push(Request::Ask { connector, ask: Ask::DropApplication { owner } });
    }
    Requests::Out(out)
}

#[expect(clippy::too_many_arguments, reason = "the admitted party envelope is passed unchanged")]
pub(super) fn admitted(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    out: &mut Queue<Request>,
    request: Token,
    connector: u16,
    person: u64,
    scope: people::ApplicationScope,
    role: Option<people::Role>,
) {
    let input = core.application_input.take();
    let owner = match input {
        Some((number, owner)) if number == connector && core.connectors.contains(&connector) => owner,
        Some(_) | None => {
            work.push(Event::People(people::Event::Decided {
                request,
                outcome: people::Outcome::Refused(people::Refusal::NotOffered),
            }));
            return;
        }
    };
    let Some(number) = fresh(&mut core.counters, Family::Application) else {
        work.push(Event::People(people::Event::Decided {
            request,
            outcome: people::Outcome::Refused(people::Refusal::Limit { which: people::LimitKind::Arithmetic }),
        }));
        return;
    };
    let token = Token::new(number);
    assert!(
        core.applications.insert(request, Flight { connector, token, part: false }).is_ok(),
        "one application per people flight"
    );
    out.push(Request::Ask {
        connector,
        ask: Ask::Application {
            request,
            token,
            owner,
            person,
            scope,
            role,
            retention: env.limits.people.request_retention,
        },
    });
}

pub(super) fn part(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    ask: people::Ask,
) -> Requests {
    let Some(flight) = core.applications.get_mut(&request) else { return empty() };
    assert!(!flight.part, "one synchronous core part at a time");
    let connector = flight.connector;
    let admitted = people::application_part(
        &mut core.people,
        &Env { now: env.now, wall: env.wall, limits: env.limits.people },
        request,
        ask,
    );
    match admitted {
        Ok(route) => {
            flight.part = true;
            let mut child = Queue::with_capacity(people::max_out(&env.limits.people));
            child.push(route);
            super::people_routes::tag_people(core, env, child, work)
        }
        Err(outcome) => answer(connector, request, outcome),
    }
}

pub(super) fn answer(connector: u16, request: Token, outcome: people::Outcome) -> Requests {
    let mut out = Queue::with_capacity(2);
    out.push(Request::Ask { connector, ask: Ask::ApplicationPartAnswered { request, outcome } });
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn finished(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    outcome: Result<Token, people::Again>,
) -> Requests {
    let Some(flight) = core.applications.remove(&request) else { return empty() };
    assert!(!flight.part, "core part settled before whole outcome");
    let outcome = match outcome {
        Ok(token) => {
            assert!(token == flight.token, "the owner kept the allocated token");
            people::Outcome::Applied { connector: flight.connector, token }
        }
        Err(why) => people::Outcome::Again(why),
    };
    super::people_routes::route_people(core, env, work, people::Event::Decided { request, outcome })
}

fn empty() -> Requests {
    let mut out = Queue::with_capacity(1);
    out.push(Request::Decided);
    Requests::Out(out)
}

impl Core {
    /// Validate retained application tokens against the restored deployment and its connectors.
    #[must_use]
    pub fn restore_people_row(&self, row: &people::Stored) -> bool {
        match row {
            people::Stored::Answer { outcome, .. } => match outcome {
                people::Outcome::Applied { connector, token } => {
                    token.raw() != 0
                        && token.raw() <= self.counters.deployment().applications
                        && self.connectors.contains(connector)
                }
                people::Outcome::Again(..)
                | people::Outcome::SignedOut
                | people::Outcome::MarkedSeen { .. }
                | people::Outcome::NoteEdited { .. }
                | people::Outcome::ServiceMade { .. }
                | people::Outcome::Adopted { .. }
                | people::Outcome::Created { .. }
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
                | people::Outcome::Refused(..) => true,
            },
            people::Stored::Person { .. }
            | people::Stored::ReadResults { .. }
            | people::Stored::ReadChat { .. }
            | people::Stored::SignIn { .. }
            | people::Stored::Roles { .. }
            | people::Stored::Policy { .. } => true,
        }
    }
}
