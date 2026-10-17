//! Root-authorized changes to live tasks (domain/tasks.md, sections 5.5 and 7).
//! The child keeps only current state; root owns durable call decisions and
//! commits every emitted save before any worker sees a control outcome.
use crate::domain::{Domain, entrance, publish, record, refused, task_mut};
use crate::{
    Active, Authority, Limits, MessageKind, Party, Phase, Refusal, Request, Spec, Stored, Tries, WakePolicy, Was, Word,
};
use alloc::boxed::Box;
use skein_lib::{Duration, Env, Queue, ReplyTo};

/// One tool change to a live delegate, authenticated by root's current claim.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Control {
    /// Close a whole subtree with a common bounded reason.
    Cancel { reason: Box<[u8]>, grace: Option<Duration>, message: Option<u64> },
    /// Lift a hold and reset the failure counts it exhausted.
    Release { again: Option<bool>, message: Option<u64> },
}

/// Whether an amendment tracks a task at a priority, or untracks it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Tracking {
    /// A goal from this decision, at this priority.
    Tracked { priority: u32 },
    /// No goal from this decision.
    Untracked,
}

/// Whole optional task changes; root checks any widening against its holder.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Amendment {
    /// Change goal tracking independently of the specification.
    pub tracked: Option<Tracking>,
    /// Whole person address change, independent of spec and hold revisions.
    pub addressed: Option<AddressChange>,
    /// Replace the charter's instructions, delivered whole as this amendment's message.
    pub instructions: Option<Box<[u8]>>,
    /// Release this exact hold with the change; a later hold remains in place.
    pub release: Option<u64>,
    pub spec: Option<Spec>,
    pub wake: Option<WakePolicy>,
    pub dependencies: Option<Box<[u64]>>,
    pub authority: Option<Authority>,
    pub reason: Box<[u8]>,
}

/// A replacement address naming the person task revision it decides about.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AddressChange {
    pub revision: u64,
    pub address: crate::PersonAddress,
}

/// Shape accepted for a person task's address, as for its creation.
pub(crate) fn valid_address(address: crate::PersonAddress) -> bool {
    match address {
        crate::PersonAddress::Person(person) => person != 0,
        crate::PersonAddress::Role(_) => true,
    }
}

/// Check the owner kind and named revision before any part of the amendment changes.
pub fn address_admit(row: &crate::TaskRecord, addressed: &Option<AddressChange>) -> Result<(), Refusal> {
    match addressed {
        None => Ok(()),
        Some(change) => {
            match row.executor {
                crate::Executor::Person(_) => {}
                crate::Executor::Agent { .. } | crate::Executor::Procedure { .. } => return Err(Refusal::Executor),
            }
            if !valid_address(change.address) {
                return Err(Refusal::Executor);
            }
            if change.revision != row.revisions.person {
                return Err(Refusal::PersonRevision { revision: row.revisions.person });
            }
            Ok(())
        }
    }
}

/// Instruction replacements belong only to agents and must fit one whole amendment message.
pub fn instruction_admit(
    limits: &Limits,
    executor: crate::Executor,
    instructions: &Option<Box<[u8]>>,
) -> Result<(), Refusal> {
    match instructions {
        None => Ok(()),
        Some(instructions) => {
            if instructions.len() > usize::try_from(limits.message_bytes).expect("u32 fits usize") {
                return Err(Refusal::Read);
            }
            match executor {
                crate::Executor::Agent { .. } => Ok(()),
                crate::Executor::Procedure { .. } | crate::Executor::Person(_) => Err(Refusal::Executor),
            }
        }
    }
}

/// A committed plan revision, kept outside the live arena for later reads.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct History {
    pub task: u64,
    pub revision: u64,
    pub by: Party,
    pub change: Change,
    pub reason: Box<[u8]>,
    pub proposal: Option<Box<crate::Proposal>>,
    pub at: skein_lib::Wall,
    pub amendment: Option<Box<AmendmentHistory>>,
}

/// Immutable settings before and after an admitted amendment.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Settings {
    pub tracked: Option<u32>,
    /// Exact person address and holder before or after the amendment.
    pub addressed: Option<crate::PersonAddress>,
    pub taken_by: Option<u64>,
    /// Exact task-owned override; absent means the configured charter default.
    pub instructions: Option<Box<[u8]>>,
    pub spec: Spec,
    pub wake: WakePolicy,
    pub dependencies: Box<[u64]>,
    pub authority: Authority,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AmendmentHistory {
    pub before: Settings,
    pub after: Settings,
}

fn settings(task: &crate::TaskRecord) -> Settings {
    Settings {
        tracked: task.tracked,
        addressed: match task.executor {
            crate::Executor::Person(address) => Some(address),
            crate::Executor::Agent { .. } | crate::Executor::Procedure { .. } => None,
        },
        taken_by: task.taken_by,
        instructions: task.instructions.clone(),
        spec: task.spec.clone(),
        wake: task.wake,
        dependencies: task.dependencies.clone(),
        authority: task.authority.clone(),
    }
}

/// One immutable lifecycle milestone, identified independently of semantic revisions.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Milestone {
    pub task: u64,
    pub position: u64,
    pub phase: Phase,
    pub at: skein_lib::Wall,
}

/// One kind of durable task-tree change.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Change {
    Cancelled,
    Released,
    Amended,
    Prioritised { priority: u32 },
    Moved,
    Proposed,
    ProposalPassed,
    ProposalAccepted,
    ProposalRejected,
    ProposalWithdrawn,
}

pub(crate) fn history(
    domain: &mut Domain,
    number: u64,
    at: skein_lib::Wall,
    by: Party,
    change: Change,
    reason: &[u8],
    out: &mut Queue<Request>,
) {
    save_history(
        domain,
        History { task: number, revision: 0, by, change, reason: reason.into(), proposal: None, at, amendment: None },
        out,
    );
}

fn save_history(domain: &mut Domain, mut row: History, out: &mut Queue<Request>) {
    let task = task_mut(domain, row.task).expect("history task live");
    task.record.revision = task.record.revision.checked_add(1).expect("revision preflighted");
    row.revision = task.record.revision;
    crate::activity::append(&mut task.record, row.at, crate::Line::Decided { revision: row.revision }, out);
    out.push(Request::Save { record: Stored::History(row) });
}

/// An authenticated amendment and its root-issued message.
pub(crate) struct Amend {
    pub funder: Option<crate::Funder>,
    pub number: u64,
    pub message: u64,
    pub stop_run: bool,
    pub amendment: Amendment,
}

pub(crate) struct AmendPlan {
    pub(crate) funding: Option<(crate::Funder, u64)>,
    pub(crate) number: u64,
    pub(crate) message: u64,
    pub(crate) amendment: Amendment,
    pub(crate) changed: bool,
    pub(crate) person_changed: bool,
    pub(crate) attempt: Option<u64>,
    pub(crate) before: Settings,
    pub(crate) prepared: crate::inbox::Prepared,
    pub(crate) release: Option<ReleasePlan>,
}

pub(crate) fn amend(
    domain: &mut Domain,
    env: &Env<Limits>,
    to: ReplyTo,
    by: Party,
    change: Amend,
    out: &mut Queue<Request>,
) {
    let number = change.number;
    let to = match entrance(domain, to, number) {
        Ok(to) => to,
        Err((to, why)) => return refused(to, Some(number), why, out),
    };
    match check_amend(domain, &crate::compound::Staged::new(0), env, by, change) {
        Ok(plan) => {
            apply_amend(domain, env, by, plan, out);
            out.push(Request::Done { reply_to: to });
        }
        Err(why) => refused(to, Some(number), why, out),
    }
}

#[expect(clippy::too_many_lines, reason = "whole amendment preflight remains together")]
pub(crate) fn check_amend(
    domain: &Domain,
    staged: &crate::compound::Staged,
    env: &Env<Limits>,
    by: Party,
    amendment: Amend,
) -> Result<AmendPlan, Refusal> {
    let Amend { number, message, stop_run, funder, amendment } = amendment;
    if !domain.ready() {
        return Err(Refusal::NotReady);
    }
    if record(domain, number).is_none() {
        return Err(Refusal::Unknown);
    }
    if !allowed(domain, number, by, env.limits.tasks) {
        return Err(Refusal::Reference);
    }
    let old = staged.record(domain, number).ok_or(Refusal::Unknown)?;
    if let Some(Tracking::Tracked { .. }) = amendment.tracked
        && old.tracked.is_none()
        && let Err(why) = staged.goals(domain, old.project, &env.limits)
    {
        return Err(why);
    }
    let releasing = match old.phase {
        Phase::Held { .. } => amendment.release == Some(old.revisions.hold),
        Phase::Waiting | Phase::Active(_) | Phase::Closing(_) | Phase::Ended(_) => false,
    };
    if releasing {
        match old.phase {
            Phase::Held { why: crate::Hold::Uncertain { .. }, .. } => {
                return Err(Refusal::AuthorityShape);
            }
            Phase::Held { .. } | Phase::Waiting | Phase::Active(_) | Phase::Closing(_) | Phase::Ended(_) => {}
        }
        if old.revision.checked_add(2).is_none() {
            return Err(Refusal::State);
        }
    }
    let mutable = match old.phase {
        Phase::Waiting | Phase::Active(_) | Phase::Held { was: Was::Waiting | Was::Active(_), .. } => true,
        Phase::Closing(_) | Phase::Held { was: Was::Closing(_), .. } | Phase::Ended(_) => false,
    };
    if !mutable {
        return Err(Refusal::State);
    }
    if old.revision == u64::MAX || message == 0 || message <= old.last_message {
        return Err(Refusal::Read);
    }
    if amendment.reason.len() > usize::try_from(env.limits.message_bytes).expect("u32 fits usize") {
        return Err(Refusal::Read);
    }
    crate::compound::Staged::address(&old, amendment.addressed.as_ref())?;
    instruction_admit(&env.limits, old.executor, &amendment.instructions)?;
    if let Some(spec) = &amendment.spec
        && (!crate::batch::valid_spec(&env.limits, spec) || !spec.inputs.is_empty())
    {
        return Err(Refusal::Spec);
    }
    if let Some(wake) = &amendment.wake
        && !crate::wake::valid(wake)
    {
        return Err(Refusal::Read);
    }
    if let Some(dependencies) = &amendment.dependencies {
        let waiting = match old.phase {
            Phase::Waiting | Phase::Held { was: Was::Waiting, .. } => true,
            Phase::Active(_)
            | Phase::Closing(_)
            | Phase::Held { was: Was::Active(_) | Was::Closing(_), .. }
            | Phase::Ended(_) => false,
        };
        if !waiting || dependencies.len() > usize::try_from(env.limits.dependencies).expect("u32 fits usize") {
            return Err(Refusal::Dependencies);
        }
        for (at, dependency) in dependencies.iter().enumerate() {
            if !crate::batch::contains(old.dependencies, *dependency) {
                return Err(Refusal::Dependencies);
            }
            for earlier in dependencies.iter().take(at) {
                if earlier == dependency {
                    return Err(Refusal::Dependencies);
                }
            }
        }
    }
    if let Some(authority) = &amendment.authority {
        if !crate::batch::valid_authority(&env.limits, authority) {
            return Err(Refusal::AuthorityShape);
        }
        if funder.unwrap_or(old.funder) == old.funder && !staged.can_resize(domain, number, authority.budget.spend) {
            return Err(Refusal::Funding);
        }
    } else if stop_run {
        return Err(Refusal::State);
    }
    let funding = match funder {
        Some(destination) if destination != old.funder => {
            let budget = match &amendment.authority {
                Some(authority) => authority.budget.spend,
                None => old.numbers.budget,
            };
            crate::funders::refunding::check(domain, staged, number, destination, budget, None)?;
            Some((destination, budget))
        }
        Some(_) | None => None,
    };
    let spec_changed = match &amendment.spec {
        Some(spec) => spec != old.spec,
        None => false,
    };
    let dependencies_changed = match &amendment.dependencies {
        Some(dependencies) => dependencies.as_ref() != old.dependencies,
        None => false,
    };
    let changed = spec_changed
        || match &amendment.instructions {
            Some(instructions) => old.instructions.as_ref() != Some(instructions),
            None => false,
        }
        || dependencies_changed
        || match &amendment.wake {
            Some(wake) => *wake != old.wake,
            None => false,
        }
        || match &amendment.authority {
            Some(authority) => authority != old.authority,
            None => false,
        };
    let address_changed = match amendment.addressed {
        Some(change) => old.executor != crate::Executor::Person(change.address) || old.taken_by.is_some(),
        None => false,
    };
    let person_changed = match old.executor {
        crate::Executor::Person(_) => spec_changed || dependencies_changed || address_changed,
        crate::Executor::Agent { .. } | crate::Executor::Procedure { .. } => false,
    };
    if (changed && old.revisions.spec == u64::MAX) || (person_changed && old.revisions.person == u64::MAX) {
        return Err(Refusal::State);
    }
    let attempt = if stop_run { crate::run::run_attempt(&old.phase) } else { None };
    let next_revision = old.revision.checked_add(1).expect("revision preflighted");
    let word = Word {
        number: message,
        from: by,
        kind: MessageKind::Amendment { revision: next_revision },
        words: match &amendment.instructions {
            Some(instructions) => instructions.clone(),
            None => amendment.reason.clone(),
        },
        at: env.wall,
        hits: 1,
        eligible: true,
    };
    let before = old.settings();
    let release = if releasing { Some(check_release(domain, staged, env, number, by, None, None)?) } else { None };
    let prepared =
        crate::inbox::prepare_staged(domain, staged, &env.limits, number, word, crate::inbox::Admission::Merged)
            .expect("authenticated amendment has a merged inbox slot");
    Ok(AmendPlan { funding, number, message, amendment, changed, person_changed, attempt, before, prepared, release })
}

pub(crate) fn apply_amend(
    domain: &mut Domain,
    env: &Env<Limits>,
    by: Party,
    plan: AmendPlan,
    out: &mut Queue<Request>,
) {
    let AmendPlan {
        funding,
        number,
        message,
        mut amendment,
        changed,
        person_changed,
        attempt,
        before,
        prepared,
        release,
    } = plan;
    if let Some((destination, budget)) = funding {
        crate::funders::refunding::apply(domain, env, number, destination, budget, out);
        if let Some(authority) = &mut amendment.authority {
            authority.budget.spend = record(domain, number).expect("amendment live").numbers.budget;
        }
    }
    if let Some(authority) = &amendment.authority
        && authority.budget.spend != record(domain, number).expect("amendment task live").numbers.budget
    {
        crate::funders::resize(domain, env, number, authority.budget.spend, out);
    }
    let task = task_mut(domain, number).expect("amendment task live");
    if let Some(tracked) = amendment.tracked {
        task.record.tracked = match tracked {
            Tracking::Tracked { priority } => Some(priority),
            Tracking::Untracked => None,
        };
    }
    if let Some(change) = amendment.addressed {
        task.record.executor = crate::Executor::Person(change.address);
        task.record.taken_by = None;
    }
    if let Some(instructions) = amendment.instructions {
        task.record.instructions = Some(instructions);
    }
    if let Some(spec) = amendment.spec {
        task.record.spec = spec;
    }
    if let Some(wake) = amendment.wake {
        task.record.wake = wake;
    }
    if let Some(dependencies) = amendment.dependencies {
        let mut waiting = skein_lib::List::with_capacity(env.limits.dependencies);
        for dependency in &task.record.waiting_on {
            if crate::batch::contains(&dependencies, *dependency) {
                waiting.push(*dependency).expect("dependency subset bound");
            }
        }
        task.record.waiting_on = waiting.into_boxed();
        task.record.dependencies = dependencies;
    }
    if let Some(authority) = amendment.authority {
        task.record.authority = authority;
    }
    if changed {
        task.record.revisions.spec = task.record.revisions.spec.checked_add(1).expect("spec revision preflighted");
    }
    if person_changed {
        task.record.revisions.person =
            task.record.revisions.person.checked_add(1).expect("person revision preflighted");
    }
    task.record.last_message = message;
    task.record.inbox = prepared.inbox;
    crate::activity::message(&mut task.record, &prepared.activity_word, env.limits.inbox_messages, out);
    if attempt.is_some() {
        task.record.narrowing = true;
        task.record.due_reason = crate::Why::Narrowed;
    }
    let after = settings(&task.record);
    save_history(
        domain,
        History {
            task: number,
            revision: 0,
            by,
            change: Change::Amended,
            reason: amendment.reason,
            proposal: None,
            at: env.wall,
            amendment: Some(Box::new(AmendmentHistory { before, after })),
        },
        out,
    );
    publish(domain, env, number, out);
    if let Some(attempt) = attempt {
        out.push(Request::Stop { task: number, attempt });
    } else {
        crate::wake::after_message(domain, env, number, prepared.previous, prepared.word, out);
    }
    if let Some(release) = release {
        apply_release(domain, env, by, release, out);
    }
}

/// Apply one bounded whole-project priority edit after the root checked the person's role.
pub(crate) fn prioritise(
    domain: &mut Domain,
    env: &Env<Limits>,
    to: ReplyTo,
    project: u32,
    by: Party,
    goals: &[(u64, u32)],
    out: &mut Queue<Request>,
) {
    if !domain.ready() {
        return refused(to, None, Refusal::NotReady, out);
    }
    let person = match by {
        Party::Person(person) if person != 0 => person,
        Party::Person(_) | Party::Task(_) | Party::Deployment { .. } => {
            return refused(to, None, Refusal::Reference, out);
        }
    };
    if project == 0 || goals.is_empty() || goals.len() > usize::try_from(env.limits.tasks).expect("u32 fits usize") {
        return refused(to, None, Refusal::Read, out);
    }
    for (at, (number, _)) in goals.iter().enumerate() {
        let Some(task) = record(domain, *number) else { return refused(to, Some(*number), Refusal::Unknown, out) };
        let mutable = match task.phase {
            Phase::Waiting | Phase::Active(_) | Phase::Held { was: Was::Waiting | Was::Active(_), .. } => true,
            Phase::Closing(_) | Phase::Held { was: Was::Closing(_), .. } | Phase::Ended(_) => false,
        };
        if task.project != project || task.tracked.is_none() || task.revision == u64::MAX || !mutable {
            return refused(to, Some(*number), Refusal::State, out);
        }
        for (earlier, _) in goals.iter().take(at) {
            if earlier == number {
                return refused(to, Some(*number), Refusal::Read, out);
            }
        }
    }
    for (number, priority) in goals {
        let task = task_mut(domain, *number).expect("priority preflight found task");
        task.record.tracked = Some(*priority);
        history(
            domain,
            *number,
            env.wall,
            Party::Person(person),
            Change::Prioritised { priority: *priority },
            &[],
            out,
        );
        publish(domain, env, *number, out);
    }
    out.push(Request::Done { reply_to: to });
}

fn below(domain: &Domain, number: u64, ancestor: u64, bound: u32) -> bool {
    let mut at = Some(number);
    for _ in 0..bound {
        let Some(number) = at else { return false };
        if number == ancestor {
            return true;
        }
        at = match record(domain, number) {
            Some(task) => match task.requester {
                Party::Task(parent) => Some(parent),
                Party::Person(_) | Party::Deployment { .. } => None,
            },
            None => None,
        };
    }
    false
}

pub(crate) fn allowed(domain: &Domain, number: u64, by: Party, bound: u32) -> bool {
    match by {
        Party::Task(task) => task != number && below(domain, number, task, bound),
        Party::Person(person) => person != 0,
        Party::Deployment { .. } => false,
    }
}

pub(crate) fn apply(
    domain: &mut Domain,
    env: &Env<Limits>,
    to: ReplyTo,
    by: Party,
    number: u64,
    control: Control,
    out: &mut Queue<Request>,
) {
    let to = match entrance(domain, to, number) {
        Ok(to) => to,
        Err((to, why)) => return refused(to, Some(number), why, out),
    };
    if !allowed(domain, number, by, env.limits.tasks) {
        return refused(to, Some(number), Refusal::Reference, out);
    }
    match control {
        Control::Cancel { reason, grace, message } => {
            match check_cancel(
                domain,
                &crate::compound::Staged::new(0),
                env,
                by,
                Cancel { number, reason, grace, message },
            ) {
                Ok(plan) => apply_cancel(domain, env, by, plan, out),
                Err(problem) => return out.push(Request::Refused { reply_to: to, problem: *problem }),
            }
            out.push(Request::Done { reply_to: to });
        }
        Control::Release { again, message } => {
            if let Err(why) = release(domain, env, number, by, again, message, out) {
                return refused(to, Some(number), why, out);
            }
            out.push(Request::Done { reply_to: to });
        }
    }
}

/// Lift a fully preflighted hold in the current decision.
pub(crate) fn release(
    domain: &mut Domain,
    env: &Env<Limits>,
    number: u64,
    by: Party,
    again: Option<bool>,
    message: Option<u64>,
    out: &mut Queue<Request>,
) -> Result<(), Refusal> {
    match check_release(domain, &crate::compound::Staged::new(0), env, number, by, again, message) {
        Ok(plan) => {
            apply_release(domain, env, by, plan, out);
            Ok(())
        }
        Err(why) => {
            if why == Refusal::Busy {
                let project = record(domain, number).expect("release task live").project;
                domain.remember_capacity(
                    env.wall,
                    crate::Capacity {
                        project: Some(project),
                        by,
                        action: crate::CapacityAction::Release,
                        cause: crate::capacity::inbox(number, &env.limits),
                    },
                );
            }
            Err(why)
        }
    }
}

pub(crate) struct ReleasePlan {
    pub(crate) number: u64,
    pub(crate) entry: Option<u64>,
    pub(crate) again: Option<bool>,
    pub(crate) notice: Option<crate::inbox::Prepared>,
    pub(crate) next: Phase,
}

pub(crate) fn check_release(
    domain: &Domain,
    staged: &crate::compound::Staged,
    env: &Env<Limits>,
    number: u64,
    by: Party,
    again: Option<bool>,
    message: Option<u64>,
) -> Result<ReleasePlan, Refusal> {
    let task = staged.record(domain, number).ok_or(Refusal::Unknown)?;
    if task.revision == u64::MAX {
        return Err(Refusal::State);
    }
    let entry = match task.phase {
        Phase::Held { why: crate::Hold::Uncertain { entry }, .. } => Some(entry),
        Phase::Held { .. } | Phase::Waiting | Phase::Active(_) | Phase::Closing(_) | Phase::Ended(_) => None,
    };
    if entry.is_some() != again.is_some() {
        return Err(Refusal::AuthorityShape);
    }
    let notice = if again == Some(false) {
        let Some(message) = message else { return Err(Refusal::AuthorityShape) };
        if message == 0 || message <= task.last_message {
            return Err(Refusal::Read);
        }
        let word = Word {
            number: message,
            from: by,
            kind: MessageKind::EffectNotMade { entry: entry.expect("uncertain choice preflighted") },
            words: Box::new([]),
            at: env.wall,
            hits: 1,
            eligible: false,
        };
        let Some(prepared) =
            crate::inbox::prepare_staged(domain, staged, &env.limits, number, word, crate::inbox::Admission::Refusable)
        else {
            return Err(Refusal::Busy);
        };
        Some(prepared)
    } else {
        if message.is_some() {
            return Err(Refusal::AuthorityShape);
        }
        None
    };
    let next = match &task.phase {
        Phase::Held { was: Was::Waiting, .. } => Phase::Waiting,
        Phase::Held {
            was: Was::Active(Active::Idle | Active::Due | Active::Preparing | Active::BackingOff { .. }),
            ..
        } => Phase::Active(Active::Due),
        Phase::Held { was: Was::Closing(closing), .. } => Phase::Closing(closing.clone()),
        Phase::Held { was: Was::Active(active @ (Active::Claimed { .. } | Active::Running { .. })), .. } => {
            Phase::Active(*active)
        }
        Phase::Waiting | Phase::Active(_) | Phase::Closing(_) | Phase::Ended(_) => {
            return Err(Refusal::State);
        }
    };
    Ok(ReleasePlan { number, entry, again, notice, next })
}

pub(crate) fn apply_release(
    domain: &mut Domain,
    env: &Env<Limits>,
    by: Party,
    plan: ReleasePlan,
    out: &mut Queue<Request>,
) {
    let ReleasePlan { number, entry, again, notice, next } = plan;
    if let Some(entry) = entry {
        out.push(Request::ResolveUncertain {
            task: number,
            entry,
            again: again.expect("uncertain choice preflighted"),
        });
    }
    let task = task_mut(domain, number).expect("entrance found live task");
    if let Some(notice) = notice {
        task.record.last_message = notice.word.number;
        task.record.inbox = notice.inbox;
        crate::activity::message(&mut task.record, &notice.activity_word, env.limits.inbox_messages, out);
    }
    task.record.narrowing |= crate::run::run_attempt(&next).is_some();
    task.record.phase = next;
    task.record.due_reason = crate::Why::Released;
    if task.record.phase == Phase::Waiting {
        task.record.hold_wait_since = None;
    }
    task.record.tries = Tries::NONE;
    task.record.refusals = 0;
    history(domain, number, env.wall, by, Change::Released, &[], out);
    publish(domain, env, number, out);
    if record(domain, number).expect("released task live").phase == Phase::Active(Active::Due) {
        crate::domain::activate(domain, number, out);
    }
    match &record(domain, number).expect("released task live").phase {
        Phase::Closing(closing) => match closing.stage {
            crate::Stage::Effects => out.push(Request::Close { task: number, ending: closing.ending.clone() }),
            crate::Stage::Releases => {
                out.push(Request::Release { task: number, ending: closing.ending.clone() });
            }
            crate::Stage::Run { .. } | crate::Stage::Delegates | crate::Stage::Settled => {}
        },
        Phase::Waiting | Phase::Active(_) | Phase::Held { .. } | Phase::Ended(_) => {}
    }
}

/// A cancel's input and root-issued finish word.
pub(crate) struct Cancel {
    pub number: u64,
    pub reason: Box<[u8]>,
    pub grace: Option<Duration>,
    pub message: Option<u64>,
}

pub(crate) struct CancelPlan {
    pub(crate) number: u64,
    pub(crate) reason: Box<[u8]>,
    pub(crate) finish: Option<crate::grace::FinishPlan>,
    pub(crate) selected: Box<[u64]>,
}

pub(crate) fn check_cancel(
    domain: &Domain,
    staged: &crate::compound::Staged,
    env: &Env<Limits>,
    by: Party,
    cancel: Cancel,
) -> Result<CancelPlan, Box<crate::Problem>> {
    let Cancel { number, reason, grace, message } = cancel;
    let mut problem =
        Box::new(crate::Problem { task: Some(number), why: Refusal::Read, blocked_by: None, capacity: None });
    if grace.is_some() != message.is_some() {
        return Err(problem);
    }
    if let Some(span) = grace
        && span > env.limits.grace
    {
        problem.why = Refusal::Grace;
        return Err(problem);
    }
    if reason.len() > usize::try_from(env.limits.result_bytes).expect("u32 fits usize") {
        return Err(problem);
    }
    let mut selected = skein_lib::List::with_capacity(env.limits.tasks);
    for (child, _) in &domain.names {
        if staged.descendant(domain, *child, number, env.limits.tasks) {
            if staged.record(domain, *child).expect("live child").revision == u64::MAX {
                problem.task = Some(*child);
                problem.why = Refusal::State;
                return Err(problem);
            }
            selected.push(*child).expect("live subtree bound");
        }
    }
    let finish = if let Some(span) = grace {
        let finish = crate::grace::Finish {
            number,
            by,
            until: skein_lib::Wall::from_nanos(env.wall.as_nanos().saturating_add(span.as_nanos())),
            message: message.expect("grace message checked"),
        };
        match crate::grace::check_finish(domain, staged, env, finish, &reason, &reason) {
            Ok(plan) => Some(plan),
            Err(why) => {
                problem.why = why;
                return Err(problem);
            }
        }
    } else {
        None
    };
    Ok(CancelPlan { number, reason, finish, selected: selected.into_boxed() })
}

pub(crate) fn apply_cancel(
    domain: &mut Domain,
    env: &Env<Limits>,
    by: Party,
    plan: CancelPlan,
    out: &mut Queue<Request>,
) {
    let CancelPlan { number, reason, finish, selected } = plan;
    if let Some(finish) = finish {
        crate::grace::apply_finish(domain, env, finish, out);
    } else {
        crate::grace::cancel_selected(domain, env, number, by, &reason, selected, out);
    }
}

/// Project the admission fields used by the remaining parts of an atomic step.
pub(crate) fn preview_amend(
    domain: &mut Domain,
    env: &Env<Limits>,
    by: Party,
    part: &crate::ProcedureAmendment,
) -> Result<(), Refusal> {
    let checked = check_amend(
        domain,
        &crate::compound::Staged::new(0),
        env,
        by,
        Amend {
            number: part.task,
            message: part.message,
            stop_run: part.stop_run,
            funder: None,
            amendment: part.amendment.clone(),
        },
    )?;
    let old = record(domain, part.task).expect("amendment checked");
    if let Some(authority) = &part.amendment.authority {
        let budget = authority.budget.spend;
        let old_budget = old.numbers.budget;
        let funder = old.funder;
        let source = match funder {
            crate::Funder::Task(number) => &mut task_mut(domain, number).expect("live funder").record.numbers,
            crate::Funder::Pool { .. } | crate::Funder::Period { .. } | crate::Funder::Recurring { .. } => {
                &mut domain.funding.get_mut(&funder).expect("live source").numbers
            }
        };
        source.reserved = source
            .reserved
            .checked_sub(old_budget)
            .expect("reservation checked")
            .checked_add(budget)
            .expect("resize checked");
        task_mut(domain, part.task).expect("live task").record.numbers.budget = budget;
    }
    let row = &mut task_mut(domain, part.task).expect("amendment checked").record;
    let amendment = &part.amendment;
    if let Some(tracked) = amendment.tracked {
        row.tracked = match tracked {
            Tracking::Tracked { priority } => Some(priority),
            Tracking::Untracked => None,
        };
    }
    if let Some(change) = amendment.addressed {
        row.executor = crate::Executor::Person(change.address);
        row.taken_by = None;
    }
    if let Some(instructions) = &amendment.instructions {
        row.instructions = Some(instructions.clone());
    }
    if let Some(spec) = &amendment.spec {
        row.spec = spec.clone();
    }
    if let Some(wake) = &amendment.wake {
        row.wake = *wake;
    }
    if let Some(dependencies) = &amendment.dependencies {
        row.dependencies.clone_from(dependencies);
    }
    if let Some(authority) = &amendment.authority {
        row.authority = authority.clone();
    }
    row.revision = row.revision.checked_add(1).expect("revision checked");
    if checked.changed {
        row.revisions.spec = row.revisions.spec.checked_add(1).expect("spec revision checked");
    }
    if checked.person_changed {
        row.revisions.person = row.revisions.person.checked_add(1).expect("person revision checked");
    }
    row.last_message = part.message;
    row.inbox = checked.prepared.inbox;
    Ok(())
}
