//! Child entry points and their requests in the core's vocabulary
//! (`domain/engine.md`, sections 3 and 4). A root receives a whole child
//! request batch and routes it within the same decision.

#![expect(clippy::mod_module_files, reason = "the route split keeps dispatch in routing/mod.rs")]

use crate::{CallKey, CallPart, Core, RoutedCall};
use alloc::boxed::Box;
use children::{route_account, route_brief, route_fleet, route_projection_settled, tag_account, tag_brief, tag_fleet};
use jig_core_accounts as accounts;
use jig_core_authority as authority;
use jig_core_brief as brief;
use jig_core_fleet as fleet;
use jig_core_notes as notes;
use jig_core_people as people;
use jig_core_tasks as tasks;
use jig_core_views as views;
use note_routes::route_notes;
use parties::route_holdings;
use people_routes::{route_people, sign_in};
use reads::{route_escalation_read, route_historical_escalation, route_historical_proposal};
use run_delegation::{
    delegate_holdings, delegate_input_failed, delegate_validated, procedure_holdings, procedure_step, route_period,
    route_start_recurring,
};
use run_preparation::{
    brief_assembled, plan_brief, route_activate, route_claim_prepared, route_claim_refused, route_preparation_failed,
    route_prepared_agent, start_brief, workspace_prepared,
};
pub(crate) use runs::named_propose;
use runs::{
    named_action, route_accepted_turn, route_answer_payload, route_named_answer, route_refused_payload,
    route_turn_payload,
};
use skein_lib::{Env, JournalRoom, List, Queue, ReplyTo, Token};
use task_routes::{
    route_connector_repair, route_person_escalation, route_person_proposal, route_task_escalation, route_tasks,
    tag_tasks,
};
pub use view_routes::watch_project;
use view_routes::{open, route_view, tag_view};
pub use vocabulary::{
    Ask, CoreBriefBudgets, EscalationChoice, Event, Held, HoldingsNeed, Limits, NamedAction, Now, PayloadRefusal,
    ProposalChoice, Request, Requests, Timer, ToolKind, Write,
};

mod children;
#[path = "notes.rs"]
mod note_routes;
mod parties;
mod party_controls;
mod party_decisions;
#[path = "people.rs"]
mod people_routes;
mod reads;
mod refusals;
mod run_delegation;
mod run_preparation;
mod runs;
#[path = "tasks.rs"]
mod task_routes;
#[path = "views.rs"]
mod view_routes;
mod vocabulary;

impl Core {
    /// Retain a numbered connector call until its translated answer arrives.
    #[must_use]
    pub fn reserve_connector_call(&mut self, key: CallKey) -> bool {
        self.pending_calls.insert(key, true) == Ok(None)
    }

    /// Retain a connector subscription's task-hub continuation with its named call.
    #[must_use]
    pub fn reserve_connector_subscription(&mut self, token: Token, key: CallKey, subscription: u64) -> bool {
        if !self.reserve_connector_call(key) {
            return false;
        }
        self.routing_calls.insert(token, RoutedCall::Subscribe { key, subscription }) == Ok(None)
    }
}

/// Person choice retained only while the task child decides its exact revision.
#[derive(PartialEq, Eq, Debug)]
pub(crate) struct PersonEscalation {
    request: Token,
    requester: u64,
    person: u64,
    project: u32,
    task: u64,
    revision: u64,
    decision: people::EscalationDecision,
}

#[derive(Debug)]
enum CreationKind {
    Goal { person: u64, direct: bool },
    Chat { person: u64 },
}

/// One person task pending all numbered connector holdings in this decision.
#[derive(Debug)]
pub(crate) struct Creation {
    kind: CreationKind,
    new: tasks::New,
    remaining: u32,
    holdings: List<tasks::Holding>,
    answered: List<u16>,
    failed: bool,
}

#[derive(Debug)]
pub(crate) struct DelegateCreation {
    key: CallKey,
    kind: BatchKind,
    members: Box<[tasks::New]>,
    remaining: u32,
    answered: List<u16>,
    failed: bool,
}

#[derive(Debug)]
enum BatchKind {
    Delegation { stubs: Box<[tasks::Stub]> },
    Proposal { reason: Box<[u8]>, as_holder: bool },
}

#[derive(Debug)]
pub(crate) struct ProcedureCreation {
    members: Box<[tasks::New]>,
    remaining: u32,
    answered: List<u16>,
    failed: bool,
}

impl Core {
    /// Check a named run call against its live task and policy before routing it.
    #[must_use]
    pub fn authorize_tool(&self, key: CallKey, kind: ToolKind) -> Option<CallPart> {
        let Some(task) = self.tasks.task(key.task) else {
            return Some(CallPart::ToolDenied {
                answer: authority::Answer::Refuse,
                findings: Box::new([authority::Finding::UnknownProject]),
            });
        };
        let family = match &kind {
            ToolKind::Delegate => self.settings.tools.delegate,
            ToolKind::Message { .. } => self.settings.tools.message,
            ToolKind::Control => self.settings.tools.control,
            ToolKind::Decide => self.settings.tools.decide,
            ToolKind::Propose => self.settings.tools.propose,
            ToolKind::Subscribe => self.settings.tools.subscribe,
            ToolKind::Effect => self.settings.tools.effect,
            ToolKind::Note { .. } => self.settings.tools.note,
            ToolKind::Recall { .. } => self.settings.tools.recall,
            ToolKind::Read => self.settings.tools.read,
        };
        let call = match kind {
            ToolKind::Message { target } => authority::Call::Message {
                referenced: task.requester == tasks::Party::Task(target)
                    || task.delegates.contains(&target)
                    || task.references.contains(&target),
            },
            ToolKind::Note { scope } => {
                let scope = match scope {
                    notes::Scope::Deployment => authority::NoteScope::Deployment,
                    notes::Scope::Project { project } if project == task.project => authority::NoteScope::Project,
                    notes::Scope::Goal { project, goal } if project == task.project && goal == task.root => {
                        authority::NoteScope::Goal
                    }
                    notes::Scope::Resources { project, connector, pattern } if project == task.project => {
                        let last = match pattern.last {
                            notes::Last::Exact(value) => authority::Last::Exact(value),
                            notes::Last::Open(value) => authority::Last::Open(value),
                        };
                        authority::NoteScope::Resources(authority::ResourceScope {
                            connector,
                            pattern: authority::Pattern { segments: pattern.segments, last },
                        })
                    }
                    notes::Scope::Project { .. } | notes::Scope::Goal { .. } | notes::Scope::Resources { .. } => {
                        return Some(CallPart::ToolDenied {
                            answer: authority::Answer::Refuse,
                            findings: Box::new([authority::Finding::Scope { source: authority::Source::Task }]),
                        });
                    }
                };
                authority::Call::Note(scope)
            }
            ToolKind::Recall { by } => {
                let readable = match by {
                    notes::Recall::Name { .. } => true,
                    notes::Recall::Search { scopes, .. } => {
                        let mut readable = true;
                        for scope in &scopes {
                            if !self.note_readable(key.task, scope) {
                                readable = false;
                            }
                        }
                        readable
                    }
                };
                if !readable {
                    return Some(CallPart::ToolDenied {
                        answer: authority::Answer::Refuse,
                        findings: Box::new([authority::Finding::Scope { source: authority::Source::Task }]),
                    });
                }
                authority::Call::Tool
            }
            ToolKind::Delegate
            | ToolKind::Control
            | ToolKind::Decide
            | ToolKind::Propose
            | ToolKind::Subscribe
            | ToolKind::Effect
            | ToolKind::Read => authority::Call::Tool,
        };
        let mut why = Queue::with_capacity(authority::max_out(self.authority.limits()).expect("bounded findings"));
        let answer = authority::check_call(
            &self.authority,
            &authority::CallAsk {
                project: task.project,
                authority: crate::translate::authority_value(&task.authority),
                family,
                call,
            },
            &mut why,
        );
        if answer == authority::Answer::Allow {
            return None;
        }
        let mut findings = List::with_capacity(why.capacity());
        for _ in 0..why.len() {
            findings.push(why.pop().expect("finding count")).expect("finding room");
        }
        Some(CallPart::ToolDenied { answer, findings: findings.into_boxed() })
    }

    /// A named recall may read only notes in this task's project and goal, or deployment notes.
    #[must_use]
    pub fn note_readable(&self, task: u64, scope: &notes::Scope) -> bool {
        let Some(row) = self.tasks.task(task) else { return false };
        match scope {
            notes::Scope::Deployment => true,
            notes::Scope::Project { project } | notes::Scope::Resources { project, .. } => *project == row.project,
            notes::Scope::Goal { project, goal } => *project == row.project && *goal == row.root,
        }
    }
}

/// The parent caller held while the notes child reads its store pages.
#[derive(Debug)]
pub(crate) enum NoteRoute {
    Call { to: ReplyTo, key: CallKey },
    Person { request: Token },
    Brief { task: u64 },
}

impl Core {
    pub(crate) fn begin_note_route(&mut self, route: NoteRoute) -> Token {
        let owner = Token::new(self.next_note_owner);
        self.next_note_owner = self.next_note_owner.checked_add(1).expect("transient note owner space");
        let previous = self.note_routes.insert(owner, route).expect("notes keeps one pending route plus a refusal");
        assert!(previous.is_none(), "transient note owners are unique");
        owner
    }
}

fn route_room_checked(limits: &Limits) -> Option<u32> {
    tasks::worst_case(&limits.tasks)?;
    let tasks = tasks::max_out(&limits.tasks).checked_mul(16)?;
    let people = people::max_out(&limits.people).checked_mul(8)?;
    let fleet = fleet::max_out(&limits.fleet).checked_mul(8)?;
    let brief = brief::gather_max_out(&limits.brief).checked_mul(4)?;
    tasks
        .checked_add(people)?
        .checked_add(fleet)?
        .checked_add(brief)?
        .checked_add(limits.people.pending.checked_mul(2)?)?
        .checked_add(
            limits.tasks.tasks.checked_mul(limits.tasks.tree_tasks.checked_add(limits.connectors)?.checked_add(4)?)?,
        )?
        .checked_add(8)
}

fn route_room(limits: &Limits) -> u32 {
    route_room_checked(limits).expect("validated core route room")
}

/// Largest core decision room, including a sibling callback reached after a
/// connector's own event. Roots use this when their input begins outside the
/// core but may synchronously hand work into it.
#[must_use]
pub fn room_max(limits: &Limits) -> Option<JournalRoom> {
    let bound = route_room_checked(limits)?;
    Some(JournalRoom { writes: bound, held: bound })
}

/// Pre-admission journal room for one core event, including all synchronous
/// sibling continuations and connector handoffs it can cause. The application
/// adds its connectors' writes and held outputs before taking the journal
/// decision (`domain/root.md`, section 10). The shared ceiling is deliberate:
/// a child callback may visit every other child before the decision ends.
#[must_use]
pub fn room(limits: &Limits, event: &Event) -> Option<JournalRoom> {
    let room = room_max(limits)?;
    match event {
        Event::EffectStart { .. }
        | Event::EffectConnector(_)
        | Event::EffectDeadline
        | Event::SignIn { .. }
        | Event::Watch { .. }
        | Event::Tasks(_)
        | Event::StartRecurring { .. }
        | Event::StartBrief { .. }
        | Event::BriefNotes { .. }
        | Event::BriefAssembled { .. }
        | Event::WorkspacePrepared { .. }
        | Event::ClaimPrepared { .. }
        | Event::ClaimRefused { .. }
        | Event::ConnectorRepair { .. }
        | Event::Period { .. }
        | Event::PreparationFailed { .. }
        | Event::PersonProposal(_)
        | Event::PersonEscalation(_)
        | Event::TaskEscalation(_)
        | Event::People(_)
        | Event::Fleet(_)
        | Event::TurnPayload { .. }
        | Event::AnswerPayload { .. }
        | Event::AcceptedTurn { .. }
        | Event::RefusedPayload { .. }
        | Event::Activate { .. }
        | Event::PreparedAgent { .. }
        | Event::HistoricalProposal { .. }
        | Event::HistoricalEscalation { .. }
        | Event::EscalationRead { .. }
        | Event::NamedAnswer { .. }
        | Event::NamedAction { .. }
        | Event::DelegateValidated { .. }
        | Event::DelegateInputFailed { .. }
        | Event::DelegateHoldings { .. }
        | Event::ProcedureHoldings { .. }
        | Event::ProcedureStep { .. }
        | Event::Brief(_)
        | Event::Holdings { .. }
        | Event::Account(_)
        | Event::View(_)
        | Event::Notes(_)
        | Event::SettledCall { .. }
        | Event::ProjectionSettled { .. } => Some(room),
    }
}

/// Route one child event and each sibling continuation within one core decision.
pub fn step(core: &mut Core, env: &Env<Limits>, event: Event) -> Requests {
    let room = route_room(&env.limits);
    let mut work = Queue::with_capacity(room);
    let mut out = Queue::with_capacity(room.checked_add(1).expect("core output mark"));
    work.push(event);
    for _ in 0..room {
        let Some(next) = work.pop() else { break };
        let Requests::Out(mut batch) = step_one(core, env, next, &mut work);
        for _ in 0..batch.len() {
            match batch.pop().expect("core route output count") {
                Request::Decided => {}
                request @ (Request::Write(_) | Request::Ask { .. } | Request::Held(_) | Request::Now(_)) => {
                    out.push(request);
                }
            }
        }
    }
    assert!(work.is_empty(), "finite sibling routes fit their bound");
    out.push(Request::Decided);
    Requests::Out(out)
}

/// Resume the fleet's next held placement or relay through the core's sibling routes.
pub fn resume_fleet(core: &mut Core, env: &Env<Limits>) -> Requests {
    let mut child = Queue::with_capacity(fleet::max_out(&env.limits.fleet));
    fleet::resume(&mut core.fleet, &Env { now: env.now, wall: env.wall, limits: env.limits.fleet }, &mut child);
    tag_fleet(core, env, child, fleet::max_out(&env.limits.fleet))
}

fn step_one(core: &mut Core, env: &Env<Limits>, event: Event, work: &mut Queue<Event>) -> Requests {
    match event {
        Event::ProjectionSettled { goal, connector } => route_projection_settled(core, env, goal, connector),
        Event::SettledCall { to, key, call } => crate::conversation::settle(core, &env.limits, to, key, call),
        Event::EffectStart { owner, connector, origin } => crate::effects::start(core, env, owner, connector, origin),
        Event::EffectConnector(event) => crate::effects::connector(core, env, work, event),
        Event::EffectDeadline => crate::effects::deadline(core, env),
        Event::SignIn { reply_to, identity } => sign_in(core, work, reply_to, identity),
        Event::Watch { watcher, sign_in, key, project, subject, ready } => {
            open(core, env, watcher, sign_in, key, project, subject, ready)
        }
        Event::Tasks(event) => route_tasks(core, env, work, event),
        Event::StartRecurring { project, authority, template } => {
            route_start_recurring(core, work, project, authority, template)
        }
        Event::StartBrief { task } => start_brief(core, env, work, task),
        Event::BriefNotes { task, lines, more } => plan_brief(core, env, work, task, lines, more),
        Event::BriefAssembled { task, ready } => brief_assembled(core, work, task, ready),
        Event::WorkspacePrepared { task, attempt, writes } => {
            workspace_prepared(core, env, work, task, attempt, writes)
        }
        Event::ClaimPrepared { task, attempt, budget, writes } => {
            route_claim_prepared(core, work, task, attempt, budget, writes)
        }
        Event::ClaimRefused { task } => route_claim_refused(core, work, task),
        Event::ConnectorRepair { seed, executor, spec, contract, holdings } => {
            route_connector_repair(work, seed, executor, spec, contract, holdings)
        }
        Event::Period { project, period, budget } => route_period(core, work, project, period, budget),
        Event::PreparationFailed { task } => route_preparation_failed(core, work, task),
        Event::PersonProposal(event) => route_person_proposal(core, env, event),
        Event::TaskEscalation(event) => route_task_escalation(core, env, event),
        Event::PersonEscalation(event) => route_person_escalation(core, env, event),
        Event::People(event) => route_people(core, env, work, event),
        Event::Holdings { request, connector, holdings } => {
            route_holdings(core, env, work, request, connector, holdings)
        }
        Event::Fleet(event) => route_fleet(core, env, event),
        Event::TurnPayload { run, attempt, turn, body, read, cumulative } => {
            route_turn_payload(core, work, run, attempt, turn, body, read, cumulative)
        }
        Event::AnswerPayload { run, attempt, payload, cumulative, end, saved, invalid_saved } => {
            route_answer_payload(core, work, run, attempt, payload, cumulative, end, saved, invalid_saved)
        }
        Event::AcceptedTurn { task, attempt, turn, accepted, cumulative, read, transcript } => {
            route_accepted_turn(core, env, task, attempt, turn, accepted, cumulative, read, transcript)
        }
        Event::RefusedPayload { request, problem, payload } => {
            route_refused_payload(core, work, request, problem, payload)
        }
        Event::Activate { context, ready } => route_activate(core, env, work, context, ready),
        Event::PreparedAgent { context, transcript_waiter, busy } => {
            route_prepared_agent(core, context, transcript_waiter, busy)
        }
        Event::HistoricalProposal { request, person, project, proposer, proposal, row, busy } => {
            route_historical_proposal(core, work, request, person, project, proposer, proposal, row, busy)
        }
        Event::HistoricalEscalation { request, person, project, task, revision, row, busy } => {
            route_historical_escalation(core, env, work, request, person, project, task, revision, row, busy)
        }
        Event::EscalationRead { to, person, task, context } => route_escalation_read(core, to, person, task, context),
        Event::NamedAnswer { to, key, part } => route_named_answer(core, to, key, part),
        Event::NamedAction { to, key, action } => named_action(core, env, work, to, key, action),
        Event::DelegateValidated { to, key, batch, stubs } => delegate_validated(core, env, to, key, batch, stubs),
        Event::DelegateInputFailed { to, key } => delegate_input_failed(work, to, key),
        Event::DelegateHoldings { request, connector, holdings } => {
            delegate_holdings(core, env, work, request, connector, holdings)
        }
        Event::ProcedureStep { task, step, connector, code, action } => {
            procedure_step(core, env, work, task, step, connector, code, action)
        }
        Event::ProcedureHoldings { task, step, connector, holdings } => {
            procedure_holdings(core, env, work, task, step, connector, holdings)
        }
        Event::Brief(event) => route_brief(core, env, event),
        Event::Account(event) => route_account(core, env, event),
        Event::View(event) => route_view(core, env, event),
        Event::Notes(event) => route_notes(core, env, work, event),
    }
}

/// Fire one due child timer, returning its whole bounded request batch.
pub fn fire(core: &mut Core, env: &Env<Limits>, timer: Timer) -> Requests {
    match timer {
        Timer::Tasks => {
            let mut out = Queue::with_capacity(tasks::max_out(&env.limits.tasks));
            tasks::fire(&mut core.tasks, &Env { now: env.now, wall: env.wall, limits: env.limits.tasks }, &mut out);
            tag_tasks(core, env, out, tasks::max_out(&env.limits.tasks), false, false, false)
        }
        Timer::Fleet => {
            let mut out = Queue::with_capacity(fleet::max_out(&env.limits.fleet));
            fleet::fire(&mut core.fleet, &Env { now: env.now, wall: env.wall, limits: env.limits.fleet }, &mut out);
            tag_fleet(core, env, out, fleet::max_out(&env.limits.fleet))
        }
        Timer::Brief => {
            let mut out = Queue::with_capacity(brief::gather_max_out(&env.limits.brief));
            brief::gather_fire(
                &mut core.brief,
                &Env { now: env.now, wall: env.wall, limits: env.limits.brief },
                &mut out,
            );
            tag_brief(core, env, out, brief::gather_max_out(&env.limits.brief))
        }
        Timer::Account => {
            let mut out = Queue::with_capacity(accounts::MAX_OUT);
            accounts::fire(
                &mut core.accounts,
                &Env { now: env.now, wall: env.wall, limits: env.limits.accounts },
                &mut out,
            );
            tag_account(out)
        }
        Timer::View => {
            let mut out = Queue::with_capacity(views::max_out(&env.limits.views));
            views::fire(&mut core.views, &Env { now: env.now, wall: env.wall, limits: env.limits.views }, &mut out);
            tag_view(out, views::max_out(&env.limits.views))
        }
    }
}
