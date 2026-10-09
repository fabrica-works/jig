//! Root-authorized changes to live tasks (domain/tasks.md, sections 5.5 and 7).
//! The child keeps only current state; root owns durable call decisions and
//! commits every emitted save before any worker sees a control outcome.
use crate::domain::{Domain, entrance, publish, record, refused, task_mut};
use crate::{
    Active, Authority, Limits, MessageKind, Party, Phase, Refusal, Request, Spec, Stored, Tries, WakePolicy, Was, Word,
};
use alloc::boxed::Box;
use skein_lib::{Env, Queue, ReplyTo};

/// One tool change to a live delegate, authenticated by root's current claim.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Control {
    /// Close a whole subtree with a common bounded reason.
    Cancel { reason: Box<[u8]> },
    /// Lift a hold and reset the failure counts it exhausted.
    Release { again: Option<bool>, message: Option<u64> },
}

/// Whole optional task changes; root checks any widening against its holder.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Amendment {
    /// Release this exact hold with the change; a later hold remains in place.
    pub release: Option<u64>,
    pub spec: Option<Spec>,
    pub wake: Option<WakePolicy>,
    pub dependencies: Option<Box<[u64]>>,
    pub authority: Option<Authority>,
    pub reason: Box<[u8]>,
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

#[expect(
    clippy::too_many_arguments,
    reason = "one root-fenced amendment carries task, caller, message and stop decision"
)]
#[expect(clippy::too_many_lines, reason = "whole amendment preflight and mutation remain together")]
pub(crate) fn amend(
    domain: &mut Domain,
    env: &Env<Limits>,
    to: ReplyTo,
    by: Party,
    number: u64,
    message: u64,
    stop_run: bool,
    amendment: Amendment,
    out: &mut Queue<Request>,
) {
    let to = match entrance(domain, to, number) {
        Ok(to) => to,
        Err((to, why)) => return refused(to, Some(number), why, out),
    };
    if !allowed(domain, number, by, env.limits.tasks) {
        return refused(to, Some(number), Refusal::Reference, out);
    }
    let old = record(domain, number).expect("entrance found task");
    let releasing = match old.phase {
        Phase::Held { .. } => amendment.release == Some(old.revisions.hold),
        Phase::Waiting | Phase::Active(_) | Phase::Closing(_) | Phase::Ended(_) => false,
    };
    if releasing {
        match old.phase {
            Phase::Held { why: crate::Hold::Uncertain { .. }, .. } => {
                return refused(to, Some(number), Refusal::AuthorityShape, out);
            }
            Phase::Held { .. } | Phase::Waiting | Phase::Active(_) | Phase::Closing(_) | Phase::Ended(_) => {}
        }
        if old.revision.checked_add(2).is_none() {
            return refused(to, Some(number), Refusal::State, out);
        }
    }
    let mutable = match old.phase {
        Phase::Waiting | Phase::Active(_) | Phase::Held { was: Was::Waiting | Was::Active(_), .. } => true,
        Phase::Closing(_) | Phase::Held { was: Was::Closing(_), .. } | Phase::Ended(_) => false,
    };
    if !mutable {
        return refused(to, Some(number), Refusal::State, out);
    }
    if old.revision == u64::MAX || message == 0 || message <= old.last_message {
        return refused(to, Some(number), Refusal::Read, out);
    }
    if amendment.reason.len() > usize::try_from(env.limits.message_bytes).expect("u32 fits usize") {
        return refused(to, Some(number), Refusal::Read, out);
    }
    if let Some(spec) = &amendment.spec
        && (!crate::batch::valid_spec(&env.limits, spec) || !spec.inputs.is_empty())
    {
        return refused(to, Some(number), Refusal::Spec, out);
    }
    if let Some(wake) = &amendment.wake
        && !crate::wake::valid(wake)
    {
        return refused(to, Some(number), Refusal::Read, out);
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
            return refused(to, Some(number), Refusal::Dependencies, out);
        }
        for (at, dependency) in dependencies.iter().enumerate() {
            if !crate::batch::contains(&old.dependencies, *dependency) {
                return refused(to, Some(number), Refusal::Dependencies, out);
            }
            for earlier in dependencies.iter().take(at) {
                if earlier == dependency {
                    return refused(to, Some(number), Refusal::Dependencies, out);
                }
            }
        }
    }
    if let Some(authority) = &amendment.authority {
        if !crate::batch::valid_authority(&env.limits, authority) {
            return refused(to, Some(number), Refusal::AuthorityShape, out);
        }
        if !crate::funders::can_resize(domain, number, authority.budget.spend) {
            return refused(to, Some(number), Refusal::Funding, out);
        }
    } else if stop_run {
        return refused(to, Some(number), Refusal::State, out);
    }
    let spec_changed = match &amendment.spec {
        Some(spec) => *spec != old.spec,
        None => false,
    };
    let dependencies_changed = match &amendment.dependencies {
        Some(dependencies) => *dependencies != old.dependencies,
        None => false,
    };
    let changed = spec_changed
        || dependencies_changed
        || match &amendment.wake {
            Some(wake) => *wake != old.wake,
            None => false,
        }
        || match &amendment.authority {
            Some(authority) => *authority != old.authority,
            None => false,
        };
    let person_changed = match old.executor {
        crate::Executor::Person(_) => spec_changed || dependencies_changed,
        crate::Executor::Agent { .. } | crate::Executor::Procedure { .. } => false,
    };
    if (changed && old.revisions.spec == u64::MAX) || (person_changed && old.revisions.person == u64::MAX) {
        return refused(to, Some(number), Refusal::State, out);
    }
    let attempt = if stop_run { crate::run::run_attempt(&old.phase) } else { None };
    let next_revision = old.revision.checked_add(1).expect("revision preflighted");
    let word = Word {
        number: message,
        from: by,
        kind: MessageKind::Amendment { revision: next_revision },
        words: amendment.reason.clone(),
        at: env.wall,
        hits: 1,
        eligible: true,
    };
    let before = settings(old);
    let prepared = crate::inbox::prepare(domain, &env.limits, number, word, crate::inbox::Admission::Merged)
        .expect("authenticated amendment has a merged inbox slot");
    if let Some(authority) = &amendment.authority
        && authority.budget.spend != old.numbers.budget
    {
        crate::funders::resize(domain, env, number, authority.budget.spend, out);
    }
    let task = task_mut(domain, number).expect("amendment task live");
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
    crate::activity::message(&mut task.record, &prepared.activity_word, out);
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
    if releasing {
        release(domain, env, number, by, None, None, out).expect("amendment preflighted its exact hold release");
    }
    out.push(Request::Done { reply_to: to });
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

fn allowed(domain: &Domain, number: u64, by: Party, bound: u32) -> bool {
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
        Control::Cancel { reason } => {
            if reason.len() > usize::try_from(env.limits.result_bytes).expect("u32 fits usize") {
                return refused(to, Some(number), Refusal::Read, out);
            }
            for (child, _) in &domain.names {
                if below(domain, *child, number, env.limits.tasks)
                    && record(domain, *child).expect("live child").revision == u64::MAX
                {
                    return refused(to, Some(*child), Refusal::State, out);
                }
            }
            let mut selected = skein_lib::List::with_capacity(env.limits.tasks);
            for (child, _) in &domain.names {
                if below(domain, *child, number, env.limits.tasks) {
                    selected.push(*child).expect("live child bound");
                }
            }
            for child in selected.into_boxed() {
                history(domain, child, env.wall, by, Change::Cancelled, &reason, out);
            }
            crate::closing::cancel_tree(domain, env, number, by, &reason, out);
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
    let task = record(domain, number).expect("entrance found live task");
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
            crate::inbox::prepare(domain, &env.limits, number, word, crate::inbox::Admission::Refusable)
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
        crate::activity::message(&mut task.record, &notice.activity_word, out);
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
    Ok(())
}
