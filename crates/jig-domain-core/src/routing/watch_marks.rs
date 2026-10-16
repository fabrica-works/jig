//! Bounded subject marks gathered while a decision routes.
//! `client/domain/contract.md`, 7.2. Task saves gather the affected watched
//! subjects; party-entry updates gather frame and inbox subjects. Marks hold
//! no state snapshots and coalesce in a set. Once projection handoffs finish,
//! the views child asks for documents from the decision's final live state.

use crate::documents::{self, Proposer, Subject};
use crate::{Core, Limits, Request, Requests};
use jig_domain_people as people;
use jig_domain_people::ApplicationScope;
use jig_domain_tasks as tasks;
use jig_domain_views as views;
use skein_lib::{Env, List, Queue, Set};

impl Core {
    /// Connector settlement changes the chips nested in watched documents.
    pub(crate) fn touch_task_number(&mut self, limits: &Limits, task: u64) {
        touch_number(self, limits, task);
    }

    /// Gather this party's frame, inbox and conversations after owned facts change.
    pub fn touch_party(&mut self, party: u64) {
        for (_, binding) in &self.view_subjects {
            let subject = &self.view_names.get(binding.subject).expect("bound subject").subject;
            let touched = match subject {
                Subject::Frame { party: owner }
                | Subject::Inbox { party: owner }
                | Subject::Conversations { party: owner } => *owner == party,
                Subject::Task { .. }
                | Subject::Tree { .. }
                | Subject::Chips { .. }
                | Subject::Goals { .. }
                | Subject::Query { .. }
                | Subject::Tasks { .. }
                | Subject::Run { .. }
                | Subject::Deployment
                | Subject::Proposal { .. }
                | Subject::Escalation { .. }
                | Subject::Question { .. }
                | Subject::PersonTask { .. }
                | Subject::Result { .. }
                | Subject::Project { .. }
                | Subject::Note { .. }
                | Subject::Application { .. } => false,
            };
            if touched {
                let _new = self.changed_views.insert(binding.subject).expect("bounded watched marks");
            }
        }
    }

    /// The configured creation period governs every project's current pool join.
    pub(crate) fn touch_frames(&mut self) {
        for (_, binding) in &self.view_subjects {
            let subject = &self.view_names.get(binding.subject).expect("bound subject").subject;
            let touched = match subject {
                Subject::Frame { .. } => true,
                Subject::Deployment
                | Subject::Task { .. }
                | Subject::Tree { .. }
                | Subject::Chips { .. }
                | Subject::Inbox { .. }
                | Subject::Goals { .. }
                | Subject::Query { .. }
                | Subject::Tasks { .. }
                | Subject::Conversations { .. }
                | Subject::Run { .. }
                | Subject::Proposal { .. }
                | Subject::Escalation { .. }
                | Subject::Question { .. }
                | Subject::PersonTask { .. }
                | Subject::Result { .. }
                | Subject::Project { .. }
                | Subject::Note { .. }
                | Subject::Application { .. } => false,
            };
            if touched {
                let _new = self.changed_views.insert(binding.subject).expect("bounded watched marks");
            }
        }
    }

    /// Gather deployment facts after host, account or task capacity changes.
    pub fn touch_deployment(&mut self) {
        for (_, binding) in &self.view_subjects {
            let subject = &self.view_names.get(binding.subject).expect("bound subject").subject;
            let touched = match subject {
                Subject::Deployment => true,
                Subject::Frame { .. }
                | Subject::Task { .. }
                | Subject::Tree { .. }
                | Subject::Chips { .. }
                | Subject::Inbox { .. }
                | Subject::Goals { .. }
                | Subject::Query { .. }
                | Subject::Tasks { .. }
                | Subject::Conversations { .. }
                | Subject::Run { .. }
                | Subject::Proposal { .. }
                | Subject::Escalation { .. }
                | Subject::Question { .. }
                | Subject::PersonTask { .. }
                | Subject::Result { .. }
                | Subject::Project { .. }
                | Subject::Note { .. }
                | Subject::Application { .. } => false,
            };
            if touched {
                let _new = self.changed_views.insert(binding.subject).expect("bounded watched marks");
            }
        }
    }

    pub(crate) fn touch_project_parties(&mut self, project: u32) {
        self.touch_project_objects(project);
        let mut parties = List::with_capacity(self.view_subjects.len());
        for (_, binding) in &self.view_subjects {
            if self.people.role(binding.party, project).is_some() {
                parties.push(binding.party).expect("one party per watch");
            }
        }
        for party in parties.into_boxed() {
            self.touch_party(party);
        }
    }

    /// Role, policy and funding facts govern offers on every project object.
    pub(crate) fn touch_project_objects(&mut self, project: u32) {
        for (_, binding) in &self.view_subjects {
            let subject = &self.view_names.get(binding.subject).expect("bound subject").subject;
            let touched = match subject {
                Subject::Project { project: owned }
                | Subject::Goals { project: owned }
                | Subject::Tasks { project: owned, .. } => *owned == project,
                // Store-only note names do not retain their scope in this core.
                Subject::Query { query } => query.project == Some(project),
                Subject::Note { .. } => true,
                Subject::Task { task }
                | Subject::Tree { task }
                | Subject::Run { task }
                | Subject::Escalation { task }
                | Subject::Question { task, .. }
                | Subject::PersonTask { task }
                | Subject::Result { task }
                | Subject::Proposal { proposer: Proposer::Task(task), .. } => match self.tasks.task(*task) {
                    Some(row) => row.project == project,
                    None => false,
                },
                Subject::Proposal { proposer: Proposer::Party(party), number } => {
                    match self.tasks.person_proposal(*party, *number) {
                        Some(row) => row.project == project,
                        None => false,
                    }
                }
                Subject::Chips { tasks } => {
                    let mut touched = false;
                    for task in tasks {
                        touched |= match self.tasks.task(*task) {
                            Some(row) => row.project == project,
                            None => false,
                        };
                    }
                    touched
                }
                Subject::Frame { .. }
                | Subject::Inbox { .. }
                | Subject::Conversations { .. }
                | Subject::Deployment
                | Subject::Application { .. } => false,
            };
            if touched {
                let _new = self.changed_views.insert(binding.subject).expect("bounded watched marks");
            }
        }
    }
}

pub(super) fn touch_task(core: &mut Core, limits: &Limits, row: &tasks::TaskRecord) {
    if core.view_subjects.is_empty() {
        return;
    }
    core.touch_deployment();
    let mut trees = List::with_capacity(limits.tasks.depth.checked_add(1).expect("bounded lineage"));
    trees.push(row.number).expect("a task is in its own tree");
    let mut requester = row.requester;
    for _ in 0..limits.tasks.depth {
        requester = match requester {
            tasks::Party::Task(parent) => {
                trees.push(parent).expect("bounded task lineage");
                match core.tasks.delegation(parent) {
                    Some(parent) => parent.requester,
                    None => break,
                }
            }
            tasks::Party::Person(_) | tasks::Party::Deployment { .. } => break,
        };
    }
    let goal = core.goal_of(row).is_some();
    let root = match core.tasks.task(row.root) {
        Some(root) => root,
        None => row,
    };
    let owner = core.conversation_requester(root);
    for (_, binding) in &core.view_subjects {
        let subject = &core.view_names.get(binding.subject).expect("bound subject").subject;
        let touched = match subject {
            Subject::Task { task } | Subject::Tree { task } => trees.as_slice().contains(task),
            Subject::Run { task }
            | Subject::Escalation { task }
            | Subject::Question { task, .. }
            | Subject::PersonTask { task }
            | Subject::Result { task }
            | Subject::Proposal { proposer: Proposer::Task(task), .. } => *task == row.number,
            Subject::Project { project } | Subject::Tasks { project, .. } => *project == row.project,
            Subject::Query { query } => {
                let mut matches = documents::task_query::may_touch(query, row);
                for number in &trees {
                    if let Some(ancestor) = core.tasks.task(*number) {
                        matches |= documents::task_query::may_touch(query, ancestor);
                    }
                }
                matches
            }
            Subject::Chips { tasks } => tasks.as_slice().contains(&row.number),
            Subject::Goals { project } => *project == row.project && goal,
            Subject::Conversations { party } => owner == Some(*party),
            Subject::Inbox { party } => core.people.role(*party, row.project).is_some(),
            Subject::Frame { .. }
            | Subject::Deployment
            | Subject::Note { .. }
            | Subject::Application { .. }
            | Subject::Proposal { proposer: Proposer::Party(_), .. } => false,
        };
        if touched {
            let _new = core.changed_views.insert(binding.subject).expect("bounded watched marks");
        }
    }
}

pub(super) fn touch_number(core: &mut Core, limits: &Limits, task: u64) {
    if core.view_subjects.is_empty() {
        return;
    }
    // The row belongs to the hub; this temporary copy ends in this call.
    let row = match core.tasks.task(task) {
        Some(row) => row.clone(),
        None => return,
    };
    touch_task(core, limits, &row);
}

pub(super) fn touch_people_row(core: &mut Core, row: &people::Stored) {
    match row {
        people::Stored::Person { number, .. }
        | people::Stored::ReadResults { party: number, .. }
        | people::Stored::ReadChat { party: number, .. } => {
            core.touch_party(*number);
        }
        people::Stored::Roles { holdings, .. } => {
            for holding in holdings {
                core.touch_party(holding.person);
            }
        }
        people::Stored::Answer { .. } => core.touch_deployment(),
        people::Stored::SignIn { .. } | people::Stored::Policy { .. } => {}
    }
}

pub(super) fn touch_people_event(core: &mut Core, limits: &Limits, event: &people::Event) {
    match event {
        people::Event::Roles { project, .. } => core.touch_project_parties(*project),
        people::Event::ChatState { party, .. } => core.touch_party(*party),
        people::Event::ChatSeen { request, .. } => {
            if let Some((party, _)) = core.people.seen_context(*request) {
                core.touch_party(party);
            }
        }
        people::Event::Waiting { task, entries } => {
            let mut parties = List::with_capacity(core.view_subjects.len());
            for (_, binding) in &core.view_subjects {
                let old = core.people.cached_waiting(&limits.people, binding.party);
                let mut touched = core.people.task_needs(binding.party, *task)
                    || match old {
                        Some(old) => {
                            let mut present = false;
                            for entry in &old {
                                present |= entry.task == *task;
                            }
                            present
                        }
                        None => false,
                    };
                for entry in entries {
                    touched |= match entry.whom {
                        people::Whom::Person(party) => party == binding.party,
                        people::Whom::Role { project, role } => match core.people.role(binding.party, project) {
                            Some(held) => held.number() == role,
                            None => false,
                        },
                    };
                }
                if touched {
                    parties.push(binding.party).expect("bounded watched parties");
                }
            }
            for party in parties.into_boxed() {
                core.touch_party(party);
            }
        }
        people::Event::Forget { party, .. } => {
            if let Some(party) = party {
                core.touch_party(*party);
            }
        }
        people::Event::MakeService { .. }
        | people::Event::Seed { .. }
        | people::Event::ApplyRoles { .. }
        | people::Event::SignedIn { .. }
        | people::Event::SignOut { .. }
        | people::Event::Ask { .. }
        | people::Event::Decided { .. }
        | people::Event::Restore { .. }
        | people::Event::Restored => {}
    }
}

pub(crate) fn finish_watches(core: &mut Core, env: &Env<Limits>) -> Requests {
    let mut out = Queue::with_capacity(crate::room_max(&env.limits).expect("bounded watch finish").held);
    append(end_invalid(core, env), &mut out);
    // Retained runs are live independently of watches (QUESTIONS.md, item 41).
    // A committed turn stops being provisional; only an ended attempt frees
    // its retained-run admission. Both are fenced by the core's durable proof.
    for token in &core.views.retained_runs() {
        let position = views::position(&core.views, *token);
        let attempt = views::retained_attempt(&core.views, *token);
        let event = match core.proofs.get(&token.raw()) {
            Some(proof) if proof.terminal.is_none() && attempt != Some(proof.attempt) => {
                Some(views::Event::RunEnded { run: *token })
            }
            Some(proof) if proof.terminal.is_none() => match position {
                Some((writing, _)) if writing.attempt != proof.attempt => Some(views::Event::RunEnded { run: *token }),
                Some((writing, _)) => match proof.turn {
                    Some(kept) if writing.turn <= kept.turn => {
                        Some(views::Event::TurnCommitted { run: *token, attempt: proof.attempt, turn: kept.turn })
                    }
                    Some(_) | None => None,
                },
                None => None,
            },
            Some(_) | None => Some(views::Event::RunEnded { run: *token }),
        };
        if let Some(event) = event {
            let mut quiet = Queue::with_capacity(1);
            views::step(
                &mut core.views,
                &Env { now: env.now, wall: env.wall, limits: env.limits.views },
                event,
                &mut quiet,
            );
            assert!(quiet.is_empty(), "ending writing emits no output");
        }
    }
    let changed = core::mem::replace(&mut core.changed_views, Set::with_capacity(env.limits.views.watchers));
    for name in &changed {
        let mut subjects = List::with_capacity(1);
        subjects.push(name.token()).expect("one opaque subject mark");
        append(super::view_routes::route_view(core, env, views::Event::Changed { subjects }), &mut out);
    }
    out.push(Request::Decided);
    Requests::Out(out)
}

pub(super) fn end_invalid(core: &mut Core, env: &Env<Limits>) -> Requests {
    let mut out = Queue::with_capacity(crate::room_max(&env.limits).expect("bounded admission endings").held);
    let mut watches = List::with_capacity(core.view_subjects.len());
    for (watcher, binding) in &core.view_subjects {
        watches.push((*watcher, binding.clone())).expect("bounded watch snapshot");
    }
    for (watcher, binding) in watches.into_boxed() {
        let ending = if core.stopping {
            Some(views::End::Stopping)
        } else if core.people.person(binding.sign_in, env.now, env.wall) == Some(binding.party) {
            visibility_end(core, &binding)
        } else {
            Some(views::End::SignedOut)
        };
        if let Some(end) = ending {
            append(super::view_routes::route_view(core, env, views::Event::EndWatch { watcher, end }), &mut out);
        }
    }
    crate::stopping::drained(core, &mut out);
    out.push(Request::Decided);
    Requests::Out(out)
}

fn append(requests: Requests, out: &mut Queue<Request>) {
    let Requests::Out(mut requests) = requests;
    for _ in 0..requests.len() {
        match requests.pop().expect("watch route outputs") {
            Request::Decided => {}
            request @ (Request::Write(_) | Request::Held(_) | Request::Now(_) | Request::Ask { .. }) => {
                out.push(request);
            }
        }
    }
}

fn visibility_end(core: &Core, binding: &crate::WatchBinding) -> Option<views::End> {
    let subject = &core.view_names.get(binding.subject).expect("bound subject").subject;
    match subject {
        Subject::Application { scope, .. } => {
            if *scope == ApplicationScope::Own || core.people.role(binding.party, scope.project()).is_some() {
                None
            } else {
                Some(views::End::NotAllowed)
            }
        }
        Subject::Task { task } | Subject::Tree { task } | Subject::Run { task } => {
            let Some(row) = core.tasks.task(*task) else { return Some(views::End::Gone) };
            match row.phase {
                tasks::Phase::Ended(_) => Some(views::End::Gone),
                tasks::Phase::Waiting
                | tasks::Phase::Active(_)
                | tasks::Phase::Closing(_)
                | tasks::Phase::Held { .. } => {
                    if core.people.role(binding.party, row.project).is_some() {
                        None
                    } else {
                        Some(views::End::NotAllowed)
                    }
                }
            }
        }
        Subject::Chips { tasks: named } => {
            for task in named {
                if let Some(row) = core.tasks.task(*task)
                    && core.people.role(binding.party, row.project).is_none()
                {
                    return Some(views::End::NotAllowed);
                }
            }
            None
        }
        Subject::Goals { project } | Subject::Tasks { project, .. } => {
            if core.people.role(binding.party, *project).is_some() { None } else { Some(views::End::NotAllowed) }
        }
        Subject::Query { query } => match documents::task_query::admission(core, binding.party, query) {
            Ok(()) => None,
            Err(_) => Some(views::End::NotAllowed),
        },
        Subject::Deployment => {
            if core.people.role(binding.party, 0).is_some() {
                None
            } else {
                Some(views::End::NotAllowed)
            }
        }
        Subject::Frame { .. } | Subject::Inbox { .. } | Subject::Conversations { .. } => None,
        Subject::Proposal { .. }
        | Subject::Escalation { .. }
        | Subject::Question { .. }
        | Subject::PersonTask { .. }
        | Subject::Result { .. }
        | Subject::Project { .. }
        | Subject::Note { .. } => Some(views::End::Gone),
    }
}
