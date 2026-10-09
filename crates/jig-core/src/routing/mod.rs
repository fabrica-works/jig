//! Child entry points and their requests in the core's vocabulary
//! (`domain/engine.md`, sections 3 and 4). A root receives a whole child
//! request batch and routes it within the same decision.

#![expect(clippy::mod_module_files, reason = "the route split keeps dispatch in routing/mod.rs")]

use crate::{CallKey, CallPart, Core, GoalRoute, Key, PersonProposalRoute, PersonTaskRoute, Record, RoutedCall};
use children::{
    append_people, remember_due, route_account, route_brief, route_fleet, route_projection_settled, tag_account,
    tag_brief, tag_fleet,
};
use note_routes::route_notes;
use parties::route_holdings;
use people_routes::{route_people, sign_in};
use reads::{route_escalation_read, route_historical_escalation, route_historical_proposal};
use runs::{
    brief_assembled, call_decision, delegate_holdings, delegate_input_failed, delegate_validated, named_action,
    plan_brief, procedure_holdings, procedure_step, route_accepted_turn, route_activate, route_answer_payload,
    route_claim_prepared, route_claim_refused, route_named_answer, route_period, route_preparation_failed,
    route_prepared_agent, route_refused_payload, route_start_recurring, route_turn_payload, start_brief,
    workspace_prepared,
};
use skein_lib::{Env, JournalRoom, List, Queue, ReplyTo, Token};
use task_routes::{
    route_connector_repair, route_person_escalation, route_person_proposal, route_task_escalation, route_tasks,
    tag_tasks,
};
use view_routes::route_view;

mod children;
#[path = "notes.rs"]
mod note_routes;
mod parties;
#[path = "people.rs"]
mod people_routes;
mod reads;
mod runs;
#[path = "tasks.rs"]
mod task_routes;
#[path = "views.rs"]
mod view_routes;
mod vocabulary;

use alloc::boxed::Box;
use jig_core_accounts as accounts;
use jig_core_authority as authority;
use jig_core_brief as brief;
use jig_core_fleet as fleet;
use jig_core_notes as notes;
use jig_core_people as people;
use jig_core_tasks as tasks;
use jig_core_views as views;
pub use vocabulary::{
    Ask, CoreBriefBudgets, EscalationChoice, Event, Held, HoldingsNeed, Limits, NamedAction, Now, PayloadRefusal,
    ProposalChoice, Request, Requests, Timer, ToolKind, Write,
};

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
    fn begin_note_route(&mut self, route: NoteRoute) -> Token {
        let owner = Token::new(self.next_note_owner);
        self.next_note_owner = self.next_note_owner.checked_add(1).expect("transient note owner space");
        let previous = self.note_routes.insert(owner, route).expect("notes keeps one pending route plus a refusal");
        assert!(previous.is_none(), "transient note owners are unique");
        owner
    }
}

fn person_task_refusal(why: tasks::Refusal) -> people::Refusal {
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

fn pool_refused(work: &mut Queue<Event>, request: Token, why: people::Refusal) {
    work.push(Event::People(people::Event::Decided { request, outcome: people::Outcome::Refused(why) }));
}

#[expect(clippy::too_many_arguments, reason = "a keyed pool edit carries its owner, beneficiary and budget")]
fn pool_route(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    owner: u64,
    project: u32,
    person: u64,
    budget: u64,
) {
    if let Err(why) = core.role_allowed(owner, project, env.limits.tasks.tree_tasks) {
        pool_refused(work, request, why);
        return;
    }
    if core.person_tasks.len() >= env.limits.tasks.tasks || work.room() < 2 {
        pool_refused(work, request, people::Refusal::Busy);
        return;
    }
    let Some(role) = core.people.role(person, project) else {
        pool_refused(work, request, people::Refusal::Unknown);
        return;
    };
    let Some(allowed) = core.authority.role(project, role.number()) else {
        pool_refused(work, request, people::Refusal::Unknown);
        return;
    };
    if budget > allowed.period_spend {
        pool_refused(work, request, people::Refusal::Authority);
        return;
    }
    let Some(policy) = core.authority.policy(project) else {
        pool_refused(work, request, people::Refusal::Unknown);
        return;
    };
    if core.settings.period_budget > policy.period_spend {
        pool_refused(work, request, people::Refusal::Authority);
        return;
    }
    let period = tasks::Funder::Period { project, period: core.settings.period };
    let funder = tasks::Funder::Pool { project, person, period: core.settings.period };
    let needed =
        u32::from(core.tasks.funding(period).is_none()).saturating_add(u32::from(core.tasks.funding(funder).is_none()));
    if core.tasks.funding_room() < needed {
        pool_refused(work, request, people::Refusal::Busy);
        return;
    }
    if core.tasks.funding(period).is_none() && budget > core.settings.period_budget {
        pool_refused(work, request, people::Refusal::Authority);
        return;
    }
    if core.tasks.funding(period).is_none() {
        work.push(Event::Tasks(tasks::Event::OpenPeriod {
            reply_to: ReplyTo::new(Token::new(0)),
            project,
            period: core.settings.period,
            budget: core.settings.period_budget,
        }));
    }
    assert!(
        core.person_tasks.insert(request, PersonTaskRoute::PoolSet { project, person }) == Ok(None),
        "one admitted pool route"
    );
    if core.tasks.funding(funder).is_some() {
        work.push(Event::Tasks(tasks::Event::ResizePool {
            reply_to: ReplyTo::new(request),
            project,
            person,
            period: core.settings.period,
            budget,
        }));
    } else {
        work.push(Event::Tasks(tasks::Event::CarvePool {
            reply_to: ReplyTo::new(request),
            project,
            person,
            period: core.settings.period,
            budget,
        }));
    }
}

#[expect(clippy::too_many_arguments, reason = "an exact pending goal carries the keyed holder decision")]
fn goal_decide(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    role: Option<people::Role>,
    project: u32,
    proposer: u64,
    number: u64,
    choice: people::ProposalDecision,
) {
    let Some(proposal) = core.tasks.person_proposal(proposer, number) else {
        pool_refused(work, request, people::Refusal::Ended);
        return;
    };
    if let Err(why) = core.goal_standing(&proposal, project, role) {
        pool_refused(work, request, why);
        return;
    }
    let event = match choice {
        people::ProposalDecision::Accept => {
            if core.projections.len() >= env.limits.tasks.tasks {
                pool_refused(work, request, people::Refusal::Limit);
                return;
            }
            let source = match core.goal_accept_allowed(
                &proposal,
                project,
                person,
                role.expect("checked standing"),
                env.limits.tasks.tree_tasks,
            ) {
                Ok(source) => source,
                Err(why) => {
                    pool_refused(work, request, why);
                    return;
                }
            };
            let (period, pool) = core.goal_open_pool(project, person);
            if let Some(event) = period {
                work.push(Event::Tasks(event));
            }
            if let Some(event) = pool {
                work.push(Event::Tasks(event));
            }
            let mut goal = proposal.goal.clone();
            goal.funder = source;
            assert!(
                core.goal_routes.insert(
                    request,
                    GoalRoute::Accepting { proposer, proposal: number, by: person, task: goal.number },
                ) == Ok(None),
                "one goal acceptance"
            );
            tasks::Event::Make {
                reply_to: ReplyTo::new(request),
                creator: tasks::Party::Person(proposer),
                batch: Box::new([goal]),
            }
        }
        people::ProposalDecision::Reject { reason } => {
            let Some(message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                pool_refused(work, request, people::Refusal::Limit);
                return;
            };
            assert!(
                core.goal_routes.insert(request, GoalRoute::Deciding { proposer, proposal: number, by: person })
                    == Ok(None),
                "one goal decision"
            );
            tasks::Event::DecidePersonProposal {
                reply_to: ReplyTo::new(request),
                proposer,
                proposal: number,
                by: tasks::Party::Person(person),
                message: Some(message),
                decision: tasks::ProposalDecision::Reject { reason },
            }
        }
        people::ProposalDecision::Pass => {
            pool_refused(work, request, people::Refusal::NoFurther);
            return;
        }
    };
    work.push(Event::Tasks(event));
}

#[expect(
    clippy::too_many_arguments,
    reason = "one authenticated goal carries its task specification and funding request"
)]
fn begin_goal_creation(
    core: &mut Core,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    project: u32,
    role: people::Role,
    spec: Box<[u8]>,
    charter: u32,
    budget: u64,
    priority: u32,
) {
    if core.projections.len() >= env.limits.tasks.tasks {
        pool_refused(work, request, people::Refusal::Limit);
        return;
    }
    let admitted = match core.goal_start(project, person, role, charter, budget, env.limits.tasks.tree_tasks) {
        Ok(admitted) => admitted,
        Err(why) => {
            pool_refused(work, request, why);
            return;
        }
    };
    let Some(number) = crate::fresh(&mut core.counters, crate::Family::Task) else {
        pool_refused(work, request, people::Refusal::Limit);
        return;
    };
    let new = tasks::New {
        number,
        project,
        executor: tasks::Executor::Agent { charter },
        spec: tasks::Spec { words: spec, parameters: Box::new([]), inputs: Box::new([]) },
        contract: tasks::Contract::Report { words: env.limits.tasks.result_bytes },
        authority: crate::translate::task_authority(&admitted.authority),
        numbers: tasks::Numbers { budget, spent: 0, spent_below: 0, reserved: 0 },
        funder: admitted.source,
        dependencies: Box::new([]),
        holdings: Box::new([]),
        wake: tasks::WakePolicy::DEFAULT,
        recurring: None,
        tracked: Some(priority),
    };
    if !tasks::valid_spec(&env.limits.tasks, &new.spec) || !tasks::valid_authority(&env.limits.tasks, &new.authority) {
        pool_refused(work, request, people::Refusal::Limit);
        return;
    }
    begin_creation(core, env, out, work, request, CreationKind::Goal { person, direct: admitted.direct }, new);
}

#[expect(clippy::too_many_arguments, reason = "one authenticated chat carries its membership and opening words")]
fn begin_chat_creation(
    core: &mut Core,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    project: u32,
    role: people::Role,
    words: Box<[u8]>,
) {
    if let Err(why) = core.chat_admit(project, person, role, env.limits.tasks.tree_tasks) {
        pool_refused(work, request, why);
        return;
    }
    let (period, pool) = core.goal_open_pool(project, person);
    if let Some(event) = period {
        work.push(Event::Tasks(event));
    }
    if let Some(event) = pool {
        work.push(Event::Tasks(event));
    }
    let Some(number) = crate::fresh(&mut core.counters, crate::Family::Task) else {
        pool_refused(work, request, people::Refusal::Limit);
        return;
    };
    let new = tasks::New {
        number,
        project,
        executor: tasks::Executor::Agent { charter: core.settings.charter },
        spec: tasks::Spec { words, parameters: Box::new([]), inputs: Box::new([]) },
        contract: tasks::Contract::Report { words: env.limits.tasks.result_bytes },
        authority: crate::translate::task_authority(&core.settings.chat_authority),
        numbers: tasks::Numbers {
            budget: core.settings.chat_authority.budget.spend,
            spent: 0,
            spent_below: 0,
            reserved: 0,
        },
        funder: tasks::Funder::Pool { project, person, period: core.settings.period },
        dependencies: Box::new([]),
        holdings: Box::new([]),
        wake: tasks::WakePolicy::DEFAULT,
        recurring: None,
        tracked: None,
    };
    begin_creation(core, env, out, work, request, CreationKind::Chat { person }, new);
}

fn begin_creation(
    core: &mut Core,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
    work: &mut Queue<Event>,
    request: Token,
    kind: CreationKind,
    new: tasks::New,
) {
    if core.creating.len() == core.creating.capacity() {
        pool_refused(work, request, people::Refusal::Busy);
        return;
    }
    let remaining = u32::try_from(core.connectors.len()).expect("configured connector count fits u32");
    assert!(remaining > 0, "task creation has a numbered connector");
    for &connector in core.connectors.as_ref() {
        out.push(Request::Ask {
            connector,
            ask: Ask::TaskHoldings {
                request,
                project: new.project,
                root: new.number,
                number: new.number,
                executor: new.executor,
                spec: new.spec.clone(),
            },
        });
    }
    let inserted = core.creating.insert(
        request,
        Creation {
            kind,
            new,
            remaining,
            holdings: List::with_capacity(env.limits.tasks.holdings),
            answered: List::with_capacity(remaining),
            failed: false,
        },
    );
    match inserted {
        Ok(None) => {}
        Ok(Some(_)) | Err(_) => unreachable!("one keyed task creation"),
    }
}

fn created_holdings(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    connector: u16,
    holdings: Option<Box<[tasks::Holding]>>,
) {
    let Some(creating) = core.creating.get_mut(&request) else { return };
    assert!(creating.remaining > 0, "one holdings answer per connector");
    assert!(
        core.connectors.contains(&connector) && !creating.answered.as_slice().contains(&connector),
        "one answer from each numbered connector"
    );
    creating.answered.push(connector).expect("one answer per configured connector");
    creating.remaining = creating.remaining.checked_sub(1).expect("one holdings answer per connector");
    match holdings {
        Some(holdings) => {
            for holding in holdings {
                if creating.holdings.push(holding).is_err() {
                    creating.failed = true;
                }
            }
        }
        None => creating.failed = true,
    }
    if creating.remaining != 0 {
        return;
    }
    let creating = core.creating.remove(&request).expect("completed creation flight");
    if creating.failed {
        pool_refused(work, request, people::Refusal::Limit);
        return;
    }
    let mut new = creating.new;
    new.holdings = creating.holdings.into_boxed();
    match creating.kind {
        CreationKind::Goal { person, direct: true } => {
            let (period, pool) = core.goal_open_pool(new.project, person);
            if let Some(event) = period {
                work.push(Event::Tasks(event));
            }
            if let Some(event) = pool {
                work.push(Event::Tasks(event));
            }
            assert!(core.made.insert(request, (new.number, true)) == Ok(None), "one goal make flight");
            work.push(Event::Tasks(tasks::Event::Make {
                reply_to: ReplyTo::new(request),
                creator: tasks::Party::Person(person),
                batch: Box::new([new]),
            }));
        }
        CreationKind::Goal { person, direct: false } => {
            let Some(proposal) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                pool_refused(work, request, people::Refusal::Limit);
                return;
            };
            assert!(
                core.goal_routes.insert(request, GoalRoute::Proposing { proposal }) == Ok(None),
                "one goal proposal flight"
            );
            work.push(Event::Tasks(tasks::Event::ProposePerson {
                reply_to: ReplyTo::new(request),
                proposal: tasks::PersonProposal {
                    number: proposal,
                    proposer: person,
                    project: new.project,
                    goal: new,
                    state: tasks::PersonProposalState::Pending { since: env.wall },
                },
            }));
        }
        CreationKind::Chat { person } => {
            assert!(core.made.insert(request, (new.number, false)) == Ok(None), "one chat make flight");
            work.push(Event::Tasks(tasks::Event::Make {
                reply_to: ReplyTo::new(request),
                creator: tasks::Party::Person(person),
                batch: Box::new([new]),
            }));
        }
    }
}

#[expect(clippy::too_many_arguments, reason = "a keyed proposal decision carries its holder and exact live proposal")]
fn person_proposal_decide(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    role: Option<people::Role>,
    project: u32,
    proposer: u64,
    number: u64,
    choice: people::ProposalDecision,
) {
    let Some(proposal) = core.tasks.proposal(proposer, number) else {
        pool_refused(work, request, people::Refusal::Ended);
        return;
    };
    if proposal.project != project {
        pool_refused(work, request, people::Refusal::Standing);
        return;
    }
    let current = match proposal.state {
        tasks::ProposalState::Pending { holder, .. } => holder,
        tasks::ProposalState::Accepted { .. }
        | tasks::ProposalState::Rejected { .. }
        | tasks::ProposalState::Withdrawn => {
            pool_refused(work, request, people::Refusal::Ended);
            return;
        }
    };
    let (_, proposal_kind) = Core::proposal_kind(&proposal.action);
    let standing = match current {
        tasks::ProposalHolder::Person(holder) => holder == person,
        tasks::ProposalHolder::Policy { project: named, .. } => {
            named == project && core.proposal_policy_standing(person, project, proposal_kind)
        }
        tasks::ProposalHolder::Task(_) => false,
    };
    if !standing || role.is_none() {
        pool_refused(work, request, people::Refusal::Standing);
        return;
    }
    let next = match choice {
        people::ProposalDecision::Accept => {
            let Some(action) = Core::proposal_action_for_check(&proposal.action) else {
                pool_refused(work, request, people::Refusal::Authority);
                return;
            };
            if !core.proposal_covers_person(
                &env.limits,
                &authority::needs(&action).expect("admitted proposal needs"),
                person,
                project,
                proposal_kind,
            ) {
                pool_refused(work, request, people::Refusal::Authority);
                return;
            }
            person_proposal_accept(core, work, request, person, proposal);
            return;
        }
        people::ProposalDecision::Reject { reason } => tasks::ProposalDecision::Reject { reason },
        people::ProposalDecision::Pass => match current {
            tasks::ProposalHolder::Person(_) => tasks::ProposalDecision::Pass {
                holder: tasks::ProposalHolder::Policy { project, kind: Core::proposal_kind(&proposal.action).0 },
            },
            tasks::ProposalHolder::Policy { .. } => {
                pool_refused(work, request, people::Refusal::NoFurther);
                return;
            }
            tasks::ProposalHolder::Task(_) => unreachable!("person standing checked"),
        },
    };
    let message = match next {
        tasks::ProposalDecision::Reject { .. } => match crate::fresh(&mut core.counters, crate::Family::Message) {
            Some(message) => Some(message),
            None => {
                pool_refused(work, request, people::Refusal::Limit);
                return;
            }
        },
        tasks::ProposalDecision::Pass { .. } => None,
        tasks::ProposalDecision::Accept => unreachable!("accept routed separately"),
    };
    assert!(
        core.routing_people_proposals
            .insert(request, PersonProposalRoute::Deciding { request, proposer, proposal: number, by: person },)
            == Ok(None),
        "one person proposal route"
    );
    work.push(Event::PersonProposal(tasks::Event::DecideProposal {
        reply_to: ReplyTo::new(request),
        proposer,
        proposal: number,
        message,
        by: tasks::Party::Person(person),
        decision: next,
    }));
}

#[expect(clippy::too_many_lines, reason = "one exhaustive authenticated person proposal dispatcher")]
fn person_proposal_accept(
    core: &mut Core,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    proposal: tasks::Proposal,
) {
    let result_followups = core.tasks.result_proposal(proposal.proposer, proposal.number);
    match proposal.action {
        tasks::ProposalAction::Effect { connector, .. } => {
            work.push(Event::EffectStart {
                owner: request,
                connector,
                origin: crate::EffectOrigin::Accept {
                    to: ReplyTo::new(request),
                    key: None,
                    proposal: Box::new(proposal),
                    by: tasks::Party::Person(person),
                },
            });
            return;
        }
        tasks::ProposalAction::Batch(_)
        | tasks::ProposalAction::Amend { .. }
        | tasks::ProposalAction::Widen { .. }
        | tasks::ProposalAction::Release { .. } => {}
    }
    let event = match proposal.action {
        tasks::ProposalAction::Effect { .. } => unreachable!("effect accepted through its connector"),
        tasks::ProposalAction::Batch(mut batch) => {
            let creator =
                if proposal.as_holder { tasks::Party::Person(person) } else { tasks::Party::Task(proposal.proposer) };
            let (period, pool) = core.goal_open_pool(proposal.project, person);
            if let Some(event) = period {
                work.push(Event::Tasks(event));
            }
            if let Some(event) = pool {
                work.push(Event::Tasks(event));
            }
            for member in &mut batch {
                member.funder = tasks::Funder::Pool { project: proposal.project, person, period: core.settings.period };
            }
            if result_followups {
                tasks::Event::MakeResultFollowups {
                    reply_to: ReplyTo::new(request),
                    proposer: proposal.proposer,
                    proposal: proposal.number,
                    batch,
                }
            } else {
                tasks::Event::Make { reply_to: ReplyTo::new(request), creator, batch }
            }
        }
        tasks::ProposalAction::Amend { task, amendment } => {
            let Some(current) = core.tasks.delegation(task) else {
                pool_refused(work, request, people::Refusal::Ended);
                return;
            };
            let Some(amend_message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                pool_refused(work, request, people::Refusal::Limit);
                return;
            };
            let after = crate::translate::authority_value(amendment.authority.as_ref().expect("admitted authority"));
            let before = crate::translate::authority_value(&current.authority);
            let stop_run = !authority::at_most(&before, &after, &core.authority.rules().implies);
            tasks::Event::Amend {
                reply_to: ReplyTo::new(request),
                by: tasks::Party::Person(person),
                task,
                message: amend_message,
                stop_run,
                amendment,
            }
        }
        tasks::ProposalAction::Widen { task, authority } => {
            let Some(amend_message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                pool_refused(work, request, people::Refusal::Limit);
                return;
            };
            tasks::Event::Amend {
                reply_to: ReplyTo::new(request),
                by: tasks::Party::Person(person),
                task,
                message: amend_message,
                stop_run: false,
                amendment: tasks::Amendment {
                    spec: None,
                    wake: None,
                    dependencies: None,
                    authority: Some(authority),
                    reason: proposal.reason,
                },
            }
        }
        tasks::ProposalAction::Release { task } => tasks::Event::Control {
            reply_to: ReplyTo::new(request),
            by: tasks::Party::Person(person),
            task,
            control: tasks::Control::Release,
        },
    };
    let Some(message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
        pool_refused(work, request, people::Refusal::Limit);
        return;
    };
    assert!(
        core.routing_people_proposals.insert(
            request,
            PersonProposalRoute::Accepting {
                request,
                person,
                proposer: proposal.proposer,
                proposal: proposal.number,
                message,
            },
        ) == Ok(None),
        "one person acceptance route"
    );
    work.push(Event::PersonProposal(event));
}

#[expect(clippy::too_many_arguments, reason = "one keyed held choice carries its person and exact semantic revision")]
fn person_escalation_decide(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    role: Option<people::Role>,
    project: u32,
    task: u64,
    revision: u64,
    decision: people::EscalationDecision,
) {
    if let people::EscalationDecision::Reject { reason } = &decision
        && reason.len() > usize::try_from(env.limits.escalation_reason_bytes).expect("reason bound fits usize")
    {
        pool_refused(work, request, people::Refusal::Limit);
        return;
    }
    let Some(context) = core.tasks.escalation(task) else {
        pool_refused(work, request, people::Refusal::Unknown);
        return;
    };
    if context.project != project || !core.escalation_visible(&context, person, role) {
        pool_refused(work, request, people::Refusal::Standing);
        return;
    }
    let pending = match context.escalation {
        tasks::Escalation::Waiting { revision: current, .. } => current == revision,
        tasks::Escalation::Unheld { .. } | tasks::Escalation::Routing { .. } | tasks::Escalation::Rejected { .. } => {
            false
        }
    };
    if !pending {
        pool_refused(work, request, people::Refusal::Unknown);
        return;
    }
    if !Core::escalation_standing(&context, person, role) {
        pool_refused(work, request, people::Refusal::Standing);
        return;
    }
    let semantic = match &decision {
        people::EscalationDecision::Release => {
            if !core.escalation_covers(&env.limits, &context, person, role)
                || !core.escalation_release_allowed(&context, person, role, env.limits.tasks.tree_tasks)
            {
                pool_refused(work, request, people::Refusal::Authority);
                return;
            }
            tasks::EscalationDecision::Release
        }
        people::EscalationDecision::Reject { reason } => tasks::EscalationDecision::Reject { reason: reason.clone() },
        people::EscalationDecision::Pass => {
            let Some(holder) = core.escalation_fallback(project) else {
                pool_refused(work, request, people::Refusal::Authority);
                return;
            };
            tasks::EscalationDecision::Pass { holder }
        }
    };
    let entry = match semantic {
        tasks::EscalationDecision::Pass { .. } => match crate::fresh(&mut core.counters, crate::Family::Message) {
            Some(entry) => Some(entry),
            None => {
                pool_refused(work, request, people::Refusal::Limit);
                return;
            }
        },
        tasks::EscalationDecision::Release | tasks::EscalationDecision::Reject { .. } => None,
    };
    assert!(
        core.person_escalations.insert(
            request,
            PersonEscalation { request, requester: context.requester, person, project, task, revision, decision },
        ) == Ok(None),
        "one keyed escalation choice"
    );
    work.push(Event::PersonEscalation(tasks::Event::DecideEscalation {
        reply_to: ReplyTo::new(request),
        task,
        revision,
        by: person,
        entry,
        decision: semantic,
    }));
}

fn current_person_escalation(core: &Core, task: u64, revision: u64) -> bool {
    let Some(context) = core.tasks.escalation(task) else { return false };
    match context.escalation {
        tasks::Escalation::Waiting { revision: current, .. } => current == revision,
        tasks::Escalation::Unheld { .. } | tasks::Escalation::Routing { .. } | tasks::Escalation::Rejected { .. } => {
            false
        }
    }
}

#[expect(clippy::too_many_arguments, reason = "one keyed role edit carries the candidate roster and its owner")]
fn roles_route(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    out: &mut Queue<Request>,
    request: Token,
    person: u64,
    project: u32,
    holdings: Box<[people::Holding]>,
) {
    if let Err(why) = core.role_allowed(person, project, env.limits.tasks.tree_tasks) {
        pool_refused(work, request, why);
        return;
    }
    for holding in &holdings {
        match holding.role {
            people::Role::Policy { role } => {
                if core.authority.role(project, role).is_none() {
                    pool_refused(work, request, people::Refusal::Unknown);
                    return;
                }
            }
            people::Role::Owner | people::Role::Maintainer | people::Role::Member | people::Role::Observer => {}
        }
    }
    let contexts = match core.tasks.project_escalations(&env.limits.tasks, project) {
        Ok(contexts) => contexts,
        Err(refusal) => {
            assert!(refusal == tasks::Refusal::NotReady, "project inspection refusal is readiness");
            pool_refused(work, request, people::Refusal::NotReady);
            return;
        }
    };
    let changed = match core.roles_preflight(&env.limits, &contexts, &holdings) {
        Ok(changed) => changed,
        Err(why) => {
            pool_refused(work, request, why);
            return;
        }
    };
    let handoffs = u32::try_from(contexts.len()).expect("bounded Waiting count").checked_add(1).expect("recheck room");
    drop(contexts);
    drop(holdings);
    if work.room() < handoffs.checked_add(env.limits.tasks.tasks).expect("validated recheck bound")
        || out.room() < changed.checked_add(2).expect("validated role cohort bound")
    {
        pool_refused(work, request, people::Refusal::Busy);
        return;
    }
    if let Err(why) = core.role_allowed(person, project, env.limits.tasks.tree_tasks) {
        pool_refused(work, request, why);
        return;
    }
    let mut applied = Queue::with_capacity(people::max_out(&env.limits.people));
    people::step(
        &mut core.people,
        &Env { now: env.now, wall: env.wall, limits: env.limits.people },
        people::Event::ApplyRoles { reply_to: ReplyTo::new(request), request },
        &mut applied,
    );
    let mut terminal = None;
    for _ in 0..applied.len() {
        match applied.pop().expect("role application output count") {
            people::Request::Save { record } => out.push(Request::Write(Write::Save(Record::People(record)))),
            people::Request::RolesApplied { reply_to, request: actual, result } => {
                assert!(
                    reply_to.into_token() == request && actual == request && terminal.is_none(),
                    "one role terminal"
                );
                terminal = Some(result);
            }
            people::Request::Erase { .. }
            | people::Request::Route { .. }
            | people::Request::Reply { .. }
            | people::Request::RolesRefused { .. }
            | people::Request::ServiceMade { .. }
            | people::Request::RestoreRefused { .. } => unreachable!("role application has one save and terminal"),
        }
    }
    if let Err(why) = terminal.expect("role application terminal") {
        pool_refused(work, request, why);
        return;
    }
    work.push(Event::Tasks(tasks::Event::RecheckEscalations { reply_to: ReplyTo::new(request), project }));
    let mut reroutes = core.proposal_recheck_project(&env.limits, project);
    for _ in 0..reroutes.len() {
        work.push(Event::Tasks(reroutes.pop().expect("bounded proposal recheck")));
    }
    work.push(Event::People(people::Event::Decided { request, outcome: people::Outcome::RolesSet { project } }));
}

#[expect(clippy::too_many_arguments, reason = "a keyed amendment carries its person, project and task")]
fn amend_route(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    role: Option<people::Role>,
    project: u32,
    task: u64,
    amendment: people::Amendment,
) {
    let Some(amendment) = crate::amendments::translate(amendment, &env.limits.tasks) else {
        pool_refused(work, request, people::Refusal::Limit);
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
    if propose {
        let Some(proposal) = crate::fresh(&mut core.counters, crate::Family::Message) else {
            pool_refused(work, request, people::Refusal::Limit);
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
        pool_refused(work, request, people::Refusal::Limit);
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

fn person_move_refusal(why: tasks::Refusal) -> people::Refusal {
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

fn person_message_refusal(why: tasks::Refusal) -> people::Refusal {
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

fn result_words(result: tasks::TaskResult) -> Box<[u8]> {
    match result {
        tasks::TaskResult::Report { words }
        | tasks::TaskResult::Verdict { words, .. }
        | tasks::TaskResult::Change { words, .. } => words,
        tasks::TaskResult::Failure { reason } => reason,
    }
}

fn ending_words(ending: tasks::Ending) -> Box<[u8]> {
    match ending {
        tasks::Ending::Done(result) => result_words(result),
        tasks::Ending::Failed { reason } => reason,
        tasks::Ending::Cancelled { reason, result } => match result {
            Some(result) => result_words(result),
            None => reason,
        },
    }
}

fn result_notice(ending: tasks::Ending) -> (tasks::ResultKind, Box<[u8]>) {
    match ending {
        tasks::Ending::Done(result) => match result {
            tasks::TaskResult::Report { words } => (tasks::ResultKind::Report, words),
            tasks::TaskResult::Verdict { code, words } => (tasks::ResultKind::Verdict { code }, words),
            tasks::TaskResult::Change { connector, kind, resource, words } => {
                (tasks::ResultKind::Change { connector, kind, resource }, words)
            }
            tasks::TaskResult::Failure { reason } => (tasks::ResultKind::Failed, reason),
        },
        tasks::Ending::Failed { reason } => (tasks::ResultKind::Failed, reason),
        tasks::Ending::Cancelled { reason, .. } => (tasks::ResultKind::Cancelled, reason),
    }
}

fn task_saved_view(core: &mut Core, env: &Env<Limits>, task: &tasks::TaskRecord, out: &mut Queue<Request>) {
    let phase = tasks::view_phase(&task.phase);
    let current = (phase, task.tracked);
    let changed = match core.view_phases.get(&task.number) {
        Some(previous) => *previous != current,
        None => true,
    };
    if phase == 4 {
        core.view_phases.remove(&task.number);
    } else {
        core.view_phases.insert(task.number, current).expect("one live phase per task");
    }
    if !changed || core.watching.is_empty() {
        return;
    }
    let mut trees = List::with_capacity(env.limits.tasks.depth.saturating_add(1));
    trees.push(Token::new(task.number)).expect("self is in its own bounded tree");
    let mut requester = task.requester;
    for _ in 0..env.limits.tasks.depth {
        requester = match requester {
            tasks::Party::Task(parent) => {
                trees.push(Token::new(parent)).expect("bounded ancestor depth");
                match core.tasks.delegation(parent) {
                    Some(context) => context.requester,
                    None => break,
                }
            }
            tasks::Party::Person(_) | tasks::Party::Deployment { .. } => break,
        };
    }
    out.push(Request::Held(Box::new(Held::ViewTaskPhase {
        task: Token::new(task.number),
        trees: trees.into_boxed(),
        project: task.project,
        phase,
        priority: task.tracked,
    })));
}

#[expect(clippy::too_many_lines, reason = "one exhaustive party vocabulary routes its bounded child decision")]
fn tag_people(
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
            people::Request::Save { record } => Request::Write(Write::Save(Record::People(record))),
            people::Request::Erase { key } => Request::Write(Write::Erase(Key::People(key))),
            people::Request::Reply { to, reply } => {
                Request::Held(Box::new(Held::PeopleReply { to, sign_in: core.signing_in, reply }))
            }
            people::Request::RolesApplied { .. } => continue,
            people::Request::RolesRefused { .. } | people::Request::RestoreRefused { .. } => {
                Request::Now(Box::new(Now::RestoreRefused))
            }
            people::Request::Route { request, person, project, role, ask } => {
                if let people::Ask::TakePerson { .. }
                | people::Ask::HandBackPerson { .. }
                | people::Ask::AnswerPerson { .. } = &*ask
                {
                    match core.person_task(request, person, project, role, *ask) {
                        Ok(event) => work.push(Event::Tasks(event)),
                        Err(why) => work.push(Event::People(people::Event::Decided {
                            request,
                            outcome: people::Outcome::Refused(why),
                        })),
                    }
                    continue;
                }
                if let people::Ask::Stop { .. } = &*ask {
                    match core.control_admit(
                        &ask,
                        person,
                        project,
                        role,
                        env.limits.tasks.depth,
                        env.limits.tasks.result_bytes,
                    ) {
                        Ok(task) => {
                            work.push(Event::Tasks(tasks::Event::Hold {
                                task,
                                why: tasks::Hold::StoppedBy { party: person },
                            }));
                            work.push(Event::People(people::Event::Decided {
                                request,
                                outcome: people::Outcome::Stopped { task },
                            }));
                        }
                        Err(why) => work.push(Event::People(people::Event::Decided {
                            request,
                            outcome: people::Outcome::Refused(why),
                        })),
                    }
                    continue;
                }
                if let people::Ask::Cancel { .. } = &*ask {
                    let admitted = core.control_admit(
                        &ask,
                        person,
                        project,
                        role,
                        env.limits.tasks.depth,
                        env.limits.tasks.result_bytes,
                    );
                    let people::Ask::Cancel { reason, .. } = *ask else { unreachable!("cancel route retains its ask") };
                    match admitted {
                        Ok(task) => {
                            assert!(
                                core.person_tasks.insert(request, PersonTaskRoute::Cancel(task)) == Ok(None),
                                "one cancel route"
                            );
                            work.push(Event::Tasks(tasks::Event::Control {
                                reply_to: ReplyTo::new(request),
                                by: tasks::Party::Person(person),
                                task,
                                control: tasks::Control::Cancel { reason },
                            }));
                        }
                        Err(why) => work.push(Event::People(people::Event::Decided {
                            request,
                            outcome: people::Outcome::Refused(why),
                        })),
                    }
                    continue;
                }
                if let people::Ask::Release { .. } = &*ask {
                    match core.control_admit(
                        &ask,
                        person,
                        project,
                        role,
                        env.limits.tasks.depth,
                        env.limits.tasks.result_bytes,
                    ) {
                        Ok(task) => {
                            assert!(
                                core.person_tasks.insert(request, PersonTaskRoute::Release(task)) == Ok(None),
                                "one release route"
                            );
                            work.push(Event::Tasks(tasks::Event::Control {
                                reply_to: ReplyTo::new(request),
                                by: tasks::Party::Person(person),
                                task,
                                control: tasks::Control::Release,
                            }));
                        }
                        Err(why) => work.push(Event::People(people::Event::Decided {
                            request,
                            outcome: people::Outcome::Refused(why),
                        })),
                    }
                    continue;
                }
                if let people::Ask::MakeService { role: service_role, .. } = &*ask {
                    let outcome = match core.role_allowed(person, project, env.limits.tasks.tree_tasks) {
                        Ok(()) if core.authority.role(project, service_role.number()).is_some() => {
                            match crate::fresh(&mut core.counters, crate::Family::Person) {
                                Some(candidate) => {
                                    let mut follow = Queue::with_capacity(people::max_out(&env.limits.people));
                                    people::step(
                                        &mut core.people,
                                        &Env { now: env.now, wall: env.wall, limits: env.limits.people },
                                        people::Event::MakeService { request, person: candidate },
                                        &mut follow,
                                    );
                                    for _ in 0..follow.len() {
                                        pending.push(follow.pop().expect("service output count"));
                                    }
                                    None
                                }
                                None => Some(people::Outcome::Refused(people::Refusal::Limit)),
                            }
                        }
                        Ok(()) => Some(people::Outcome::Refused(people::Refusal::Unknown)),
                        Err(refusal) => Some(people::Outcome::Refused(refusal)),
                    };
                    if let Some(outcome) = outcome {
                        let mut follow = Queue::with_capacity(people::max_out(&env.limits.people));
                        people::step(
                            &mut core.people,
                            &Env { now: env.now, wall: env.wall, limits: env.limits.people },
                            people::Event::Decided { request, outcome },
                            &mut follow,
                        );
                        for _ in 0..follow.len() {
                            pending.push(follow.pop().expect("service decision count"));
                        }
                    }
                    continue;
                }
                let Some(ask) = route_person_note(core, work, request, person, project, role, ask) else {
                    continue;
                };
                if let people::Ask::Adopt { .. } = &*ask {
                    match core.role_allowed(person, project, env.limits.tasks.tree_tasks) {
                        Ok(()) => {
                            let people::Ask::Adopt { adoption, .. } = *ask else {
                                unreachable!("adoption route retains its ask")
                            };
                            out.push(Request::Ask {
                                connector: adoption.resource.connector,
                                ask: Ask::Adopt { request, project, adoption },
                            });
                        }
                        Err(why) => work.push(Event::People(people::Event::Decided {
                            request,
                            outcome: people::Outcome::Refused(why),
                        })),
                    }
                    continue;
                }
                if let people::Ask::SetPool { person: beneficiary, budget, .. } = &*ask {
                    pool_route(core, env, work, request, person, project, *beneficiary, *budget);
                    continue;
                }
                if let people::Ask::DecideProposal { proposer, proposal, .. } = &*ask
                    && core.tasks.person_proposal(*proposer, *proposal).is_some()
                {
                    let people::Ask::DecideProposal { proposer, proposal, decision, .. } = *ask else {
                        unreachable!("goal route retains its decision")
                    };
                    goal_decide(core, env, work, request, person, role, project, proposer, proposal, decision);
                    continue;
                }
                if let people::Ask::DecideProposal { proposer, proposal, .. } = &*ask
                    && core.tasks.proposal(*proposer, *proposal).is_some()
                {
                    let people::Ask::DecideProposal { proposer, proposal, decision, .. } = *ask else {
                        unreachable!("proposal route retains its choice")
                    };
                    person_proposal_decide(
                        core, env, work, request, person, role, project, proposer, proposal, decision,
                    );
                    continue;
                }
                if let people::Ask::DecideProposal { proposer, proposal, .. } = &*ask {
                    out.push(Request::Now(Box::new(Now::HistoricalProposal {
                        request,
                        person,
                        project,
                        proposer: *proposer,
                        proposal: *proposal,
                    })));
                    continue;
                }
                if let people::Ask::SetRoles { .. } = &*ask {
                    let people::Ask::SetRoles { holdings, .. } = *ask else {
                        unreachable!("role route retains its roster")
                    };
                    roles_route(core, env, work, &mut out, request, person, project, holdings);
                    continue;
                }
                if let people::Ask::Amend { .. } = &*ask {
                    let people::Ask::Amend { task, amendment, .. } = *ask else {
                        unreachable!("amendment route retains its payload")
                    };
                    amend_route(core, env, work, request, person, role, project, task, amendment);
                    continue;
                }
                if let people::Ask::ChangePolicy { .. } = &*ask {
                    let people::Ask::ChangePolicy { change, .. } = *ask else {
                        unreachable!("policy route retains its edit")
                    };
                    if work.room() < env.limits.tasks.tasks.checked_add(1).expect("task recheck bound")
                        || out.room() < 2
                    {
                        pool_refused(work, request, people::Refusal::Busy);
                        continue;
                    }
                    match core.change_policy(&env.limits, person, project, change) {
                        Ok(snapshot) => {
                            out.push(Request::Write(Write::Save(Record::People(people::Stored::Policy {
                                project,
                                value: snapshot,
                            }))));
                            let mut reroutes = core.proposal_recheck_project(&env.limits, project);
                            for _ in 0..reroutes.len() {
                                work.push(Event::Tasks(reroutes.pop().expect("policy holder recheck")));
                            }
                            work.push(Event::People(people::Event::Decided {
                                request,
                                outcome: people::Outcome::PolicyChanged { project },
                            }));
                        }
                        Err(why) => pool_refused(work, request, why),
                    }
                    continue;
                }
                if let people::Ask::DecideEscalation { task, revision, .. } = &*ask
                    && current_person_escalation(core, *task, *revision)
                {
                    let people::Ask::DecideEscalation { task, revision, decision, .. } = *ask else {
                        unreachable!("held choice route retains its exact revision")
                    };
                    person_escalation_decide(core, env, work, request, person, role, project, task, revision, decision);
                    continue;
                }
                if let people::Ask::DecideEscalation { .. } = &*ask {
                    let people::Ask::DecideEscalation { task, revision, decision, .. } = *ask else {
                        unreachable!("historical choice retains its exact revision")
                    };
                    if let people::EscalationDecision::Reject { reason } = &decision
                        && reason.len()
                            > usize::try_from(env.limits.escalation_reason_bytes).expect("reason bound fits usize")
                    {
                        pool_refused(work, request, people::Refusal::Limit);
                        continue;
                    }
                    if let Some(context) = core.tasks.escalation(task)
                        && (context.project != project || !core.escalation_visible(&context, person, role))
                    {
                        pool_refused(work, request, people::Refusal::Standing);
                        continue;
                    }
                    out.push(Request::Now(Box::new(Now::HistoricalEscalation {
                        request,
                        person,
                        project,
                        task,
                        revision,
                    })));
                    continue;
                }
                if let people::Ask::SetGoal { .. } = &*ask {
                    let people::Ask::SetGoal { spec, charter, budget, priority, .. } = *ask else {
                        unreachable!("goal route retains its task specification")
                    };
                    begin_goal_creation(
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
                    );
                    continue;
                }
                if let people::Ask::StartChat { .. } = &*ask {
                    let people::Ask::StartChat { words, .. } = *ask else {
                        unreachable!("chat route retains its opening words")
                    };
                    begin_chat_creation(
                        core,
                        env,
                        &mut out,
                        work,
                        request,
                        person,
                        project,
                        role.expect("chat membership admitted"),
                        words,
                    );
                    continue;
                }
                match *ask {
                    people::Ask::Say { task, words, .. } => {
                        match core.person_message(crate::PersonMessage {
                            request,
                            person,
                            project,
                            task,
                            question: None,
                            words,
                            at: env.wall,
                        }) {
                            Ok(event) => work.push(Event::Tasks(event)),
                            Err(why) => work.push(Event::People(people::Event::Decided {
                                request,
                                outcome: people::Outcome::Refused(why),
                            })),
                        }
                        continue;
                    }
                    people::Ask::AnswerQuestion { task, question, words, .. } => {
                        match core.person_message(crate::PersonMessage {
                            request,
                            person,
                            project,
                            task,
                            question: Some(question),
                            words,
                            at: env.wall,
                        }) {
                            Ok(event) => work.push(Event::Tasks(event)),
                            Err(why) => work.push(Event::People(people::Event::Decided {
                                request,
                                outcome: people::Outcome::Refused(why),
                            })),
                        }
                        continue;
                    }
                    people::Ask::Prioritise { goals, .. } => {
                        match core.prioritise(request, person, project, role, goals, env.limits.tasks.tasks) {
                            Ok(event) => work.push(Event::Tasks(event)),
                            Err(why) => work.push(Event::People(people::Event::Decided {
                                request,
                                outcome: people::Outcome::Refused(why),
                            })),
                        }
                        continue;
                    }
                    people::Ask::Move { task, reason, .. } => {
                        match core.move_admit(
                            person,
                            role,
                            project,
                            task,
                            env.limits.tasks.tree_tasks,
                            env.limits.tasks.depth,
                        ) {
                            Ok(()) => {
                                assert!(
                                    core.moving.insert(request, task) == Ok(None),
                                    "one move flight per keyed request"
                                );
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
                            Err(why) => work.push(Event::People(people::Event::Decided {
                                request,
                                outcome: people::Outcome::Refused(why),
                            })),
                        }
                        continue;
                    }
                    people::Ask::Watch { .. }
                    | people::Ask::EditNote { .. }
                    | people::Ask::MakeService { .. }
                    | people::Ask::Adopt { .. }
                    | people::Ask::SetGoal { .. }
                    | people::Ask::Stop { .. }
                    | people::Ask::Cancel { .. }
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
                    | people::Ask::StartChat { .. } => unreachable!("the core routes every party request"),
                }
            }
        };
        out.push(request);
    }
    assert!(pending.is_empty(), "finite party routes fit their bound");
    out.push(Request::Decided);
    Requests::Out(out)
}

fn route_person_note(
    core: &mut Core,
    work: &mut Queue<Event>,
    request: Token,
    person: u64,
    project: u32,
    role: Option<people::Role>,
    ask: Box<people::Ask>,
) -> Option<Box<people::Ask>> {
    match *ask {
        people::Ask::EditNote { name, scope, change, .. } => {
            if core.note_authorized(project, role, &scope) {
                let owner = core.begin_note_route(NoteRoute::Person { request });
                let scope = person_note_scope(project, *scope);
                let change = person_note_change(*change);
                work.push(Event::Notes(notes::Event::Edit { owner, party: person, name, scope, change }));
            } else {
                work.push(Event::People(people::Event::Decided {
                    request,
                    outcome: people::Outcome::Refused(people::Refusal::Authority),
                }));
            }
            None
        }
        other @ (people::Ask::Watch { .. }
        | people::Ask::MakeService { .. }
        | people::Ask::Adopt { .. }
        | people::Ask::SetGoal { .. }
        | people::Ask::Stop { .. }
        | people::Ask::Cancel { .. }
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
        | people::Ask::Move { .. }) => Some(Box::new(other)),
    }
}

fn person_note_scope(project: u32, scope: people::NoteScope) -> notes::Scope {
    match scope {
        people::NoteScope::Deployment => notes::Scope::Deployment,
        people::NoteScope::Project => notes::Scope::Project { project },
        people::NoteScope::Goal { goal } => notes::Scope::Goal { project, goal },
        people::NoteScope::Resources { connector, pattern } => {
            let mut segments = List::with_capacity(u32::try_from(pattern.segments.len()).expect("bounded pattern"));
            for segment in pattern.segments {
                segments.push(segment).expect("pattern segment room");
            }
            let last = match pattern.last {
                people::Last::Exact(value) => notes::Last::Exact(value),
                people::Last::Open(value) => notes::Last::Open(value),
            };
            notes::Scope::Resources {
                project,
                connector,
                pattern: notes::Pattern { segments: segments.into_boxed(), last },
            }
        }
    }
}

fn person_note_change(change: people::NoteChange) -> notes::Change {
    match change {
        people::NoteChange::Correct { description, body, references, recalled } => {
            let mut kept = List::with_capacity(u32::try_from(references.len()).expect("bounded references"));
            for reference in references {
                kept.push(reference).expect("reference room");
            }
            notes::Change::Correct { description, body, references: kept, recalled }
        }
        people::NoteChange::Delete { recalled } => notes::Change::Delete { recalled },
    }
}

fn note_person_refusal(why: notes::Refusal) -> people::Refusal {
    match why {
        notes::Refusal::Busy => people::Refusal::Busy,
        notes::Refusal::Oversized | notes::Refusal::Full | notes::Refusal::RevisionExhausted => people::Refusal::Limit,
        notes::Refusal::Missing => people::Refusal::NoteMissing,
        notes::Refusal::Moved => people::Refusal::NoteMoved,
        notes::Refusal::Exists => people::Refusal::Unknown,
    }
}

fn tag_notes(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    mut child: Queue<notes::Request>,
) -> Requests {
    let room = notes::max_out(&env.limits.notes).checked_add(3).expect("bounded note route");
    let mut out = Queue::with_capacity(room);
    for _ in 0..child.len() {
        match child.pop().expect("note output count") {
            notes::Request::Save { record } => out.push(Request::Write(Write::Save(Record::Notes(record)))),
            notes::Request::Erase { key } => out.push(Request::Write(Write::Erase(Key::Notes(key)))),
            notes::Request::Load { owner, range } => {
                out.push(Request::Held(Box::new(Held::NotesLoad { owner, range })));
            }
            notes::Request::Written { owner, name, revision } => match core.note_routes.remove(&owner) {
                Some(NoteRoute::Call { to, key }) => {
                    call_decision(core, &mut out, to, key, CallPart::NoteWritten { name, revision });
                }
                Some(NoteRoute::Person { request }) => work.push(Event::People(people::Event::Decided {
                    request,
                    outcome: people::Outcome::NoteEdited { name },
                })),
                Some(NoteRoute::Brief { .. }) => unreachable!("an index cannot write an entry"),
                None => unreachable!("a note write has a waiting caller"),
            },
            notes::Request::Deleted { owner, name } => match core.note_routes.remove(&owner) {
                Some(NoteRoute::Person { request }) => work.push(Event::People(people::Event::Decided {
                    request,
                    outcome: people::Outcome::NoteEdited { name },
                })),
                Some(NoteRoute::Call { .. } | NoteRoute::Brief { .. }) | None => {
                    unreachable!("only a party deletes a note")
                }
            },
            notes::Request::Recalled { owner, entries, more } => match core.note_routes.remove(&owner) {
                Some(NoteRoute::Call { to, key }) => {
                    let part = recalled_part(core, key.task, entries, more);
                    call_decision(core, &mut out, to, key, part);
                }
                Some(NoteRoute::Person { .. } | NoteRoute::Brief { .. }) | None => {
                    unreachable!("a run owns this recall")
                }
            },
            notes::Request::Indexed { owner, lines, more } => match core.note_routes.remove(&owner) {
                Some(NoteRoute::Brief { task }) => work.push(Event::BriefNotes { task, lines, more }),
                Some(NoteRoute::Call { .. } | NoteRoute::Person { .. }) | None => {
                    unreachable!("brief preparation owns note index")
                }
            },
            notes::Request::Refused { owner, why } => match core.note_routes.remove(&owner) {
                Some(NoteRoute::Call { to, key }) => match why {
                    notes::Refusal::Busy => {
                        let _pending = core.pending_calls.remove(&key);
                        out.push(Request::Now(Box::new(Now::NoteBusy { to, key })));
                    }
                    notes::Refusal::Oversized
                    | notes::Refusal::Full
                    | notes::Refusal::Exists
                    | notes::Refusal::Missing
                    | notes::Refusal::Moved
                    | notes::Refusal::RevisionExhausted => {
                        call_decision(core, &mut out, to, key, CallPart::NoteRefused(why));
                    }
                },
                Some(NoteRoute::Person { request }) => work.push(Event::People(people::Event::Decided {
                    request,
                    outcome: people::Outcome::Refused(note_person_refusal(why)),
                })),
                Some(NoteRoute::Brief { task }) => match why {
                    notes::Refusal::Busy => {
                        core.pending_note_briefs.insert(task, true).expect("brief count bounds waiting notes");
                    }
                    notes::Refusal::Oversized
                    | notes::Refusal::Full
                    | notes::Refusal::Exists
                    | notes::Refusal::Missing
                    | notes::Refusal::Moved
                    | notes::Refusal::RevisionExhausted => {
                        work.push(Event::Tasks(tasks::Event::PreparationFailed { task }));
                    }
                },
                None => out.push(Request::Now(Box::new(Now::NotesRefused { owner, why }))),
            },
        }
    }
    if !core.notes.busy()
        && let Some((&task, _)) = core.pending_note_briefs.iter().next()
    {
        core.pending_note_briefs.remove(&task);
        work.push(Event::StartBrief { task });
    }
    out.push(Request::Decided);
    Requests::Out(out)
}

fn recalled_part(core: &Core, task: u64, entries: List<notes::Entry>, more: bool) -> CallPart {
    for entry in &entries {
        if !core.note_readable(task, &entry.scope) {
            return CallPart::ToolDenied {
                answer: authority::Answer::Refuse,
                findings: Box::new([authority::Finding::Scope { source: authority::Source::Task }]),
            };
        }
    }
    CallPart::NoteRecalled { entries: entries.into_boxed(), more }
}

fn tag_view(mut child: Queue<views::Request>, room: u32) -> Requests {
    let mut out = Queue::with_capacity(room.checked_add(1).expect("view output room"));
    for _ in 0..child.len() {
        let request = match child.pop().expect("view output count") {
            request @ (views::Request::Watching { .. }
            | views::Request::Refused { .. }
            | views::Request::Deliver { .. }
            | views::Request::Ended { .. }) => Request::Now(Box::new(Now::View(request))),
        };
        out.push(request);
    }
    out.push(Request::Decided);
    Requests::Out(out)
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
            crate::watch::open(core, env, watcher, sign_in, key, project, subject, ready)
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

pub(crate) use runs::named_propose;
