//! Indexed dependency, delegate and threshold queue waits (domain/tasks.md, 6.2, 6.6 and 6.7).
#![expect(clippy::result_large_err, reason = "bounded owner refusal carries exact capacity diagnostics inline")]

use crate::domain::{Domain, publish, record, task_mut};
use crate::{
    Circle, Hold, HoldKind, HoldSource, Holding, Limits, Name, New, Party, Phase, Problem, Refusal, Request, Was,
};
use alloc::boxed::Box;
use skein_lib::{Env, List, Map, Queue, Set};

/// A per-task comparison point: only changed membership is refreshed at publication.
#[derive(Debug)]
pub(crate) struct Indexed {
    holdings: Box<[Holding]>,
    pending: Box<[u64]>,
    taken: bool,
    queued: bool,
    held: bool,
}

/// Ordered resource ranges and reversed dependency pairs, bounded by the live edge limits.
/// The names belong to the index, independently of the durable records (domain/tasks.md, 6.7).
#[derive(Debug)]
pub(crate) struct Index {
    pub holders: Map<(Name, u64), ()>,
    pub waiters: Map<(Name, u64), ()>,
    pub dependents: Map<(u64, u64), ()>,
    pub rows: Map<u64, Indexed>,
}

impl Index {
    pub fn new(limits: &Limits) -> Self {
        let resources = resource_capacity(limits).expect("validated resource edges");
        Self {
            holders: Map::with_capacity(resources),
            waiters: Map::with_capacity(resources),
            dependents: Map::with_capacity(limits.tasks.checked_mul(limits.dependencies).expect("dependency edges")),
            rows: Map::with_capacity(limits.tasks),
        }
    }
}

/// A root may retain the full tree's holdings after children end (6.5); every
/// live row may therefore own up to tasks * holdings, including on restart.
fn resource_capacity(limits: &Limits) -> Option<u32> {
    limits.tasks.checked_mul(limits.tasks)?.checked_mul(limits.holdings)
}

/// Index and traversal heap, including owned names and the simultaneous batch/restore scratch.
pub(crate) fn worst_case(limits: &Limits) -> Option<u64> {
    let resources = resource_capacity(limits)?;
    let dependencies = limits.tasks.checked_mul(limits.dependencies)?;
    let name = u64::from(limits.hold_bytes).checked_add(List::<Box<[u8]>>::worst_case(limits.hold_segments)?)?;
    Map::<(Name, u64), ()>::worst_case(resources)?
        .checked_mul(2)?
        .checked_add(Map::<(u64, u64), ()>::worst_case(dependencies)?)?
        .checked_add(Map::<u64, Indexed>::worst_case(limits.tasks)?)?
        .checked_add(u64::from(resources).checked_mul(
            name.checked_mul(4)?.checked_add(u64::try_from(size_of::<Holding>()).ok()?.checked_mul(2)?)?,
        )?)?
        .checked_add(u64::from(dependencies).checked_mul(16)?)?
        .checked_add(Set::<u64>::worst_case(limits.tasks)?.checked_mul(2)?)?
        .checked_add(Map::<u64, u64>::worst_case(limits.tasks)?.checked_mul(3)?)?
        .checked_add(Map::<Name, u32>::worst_case(resources)?.checked_mul(2)?)?
        .checked_add(u64::from(resources).checked_mul(name)?.checked_mul(2)?)?
        .checked_add(List::<u64>::worst_case(limits.tasks)?.checked_mul(3)?)?
        .checked_add(Map::<(u64, u64), ()>::worst_case(dependencies)?)?
        .checked_add(Map::<(Name, u64), Gate>::worst_case(resources)?)?
        .checked_add(u64::from(resources).checked_mul(name)?)?
        .checked_add(List::<u64>::worst_case(limits.circle)?)
}

pub(crate) fn refresh(domain: &mut Domain, number: u64) {
    let row = record(domain, number).expect("published live row");
    let held = is_held(&row.phase);
    let queued = crate::holds::queued(row);
    if let Some(old) = domain.waits.rows.get(&number)
        && old.holdings == row.holdings
        && old.pending == row.waiting_on
        && old.taken == row.holds_taken
        && old.queued == queued
        && old.held == held
    {
        return;
    }
    let next = Indexed {
        holdings: row.holdings.clone(),
        pending: row.waiting_on.clone(),
        taken: row.holds_taken,
        queued,
        held,
    };
    let old = domain.waits.rows.remove(&number);
    if let Some(old) = &old {
        let mut freed = next.held && !old.held;
        for dependency in &old.pending {
            if !crate::batch::contains(&next.pending, *dependency) {
                domain.waits.dependents.remove(&(*dependency, number));
                freed = true;
            }
        }
        if freed {
            domain.circle_changes.insert(number).expect("one change per live task");
        }
        for holding in &old.holdings {
            let key = (crate::holds::name(holding).clone(), number);
            if old.taken && (!next.taken || !has(&next.holdings, holding)) {
                domain.waits.holders.remove(&key);
            }
            if old.queued && (!next.queued || !has(&next.holdings, holding)) {
                domain.waits.waiters.remove(&key);
            }
        }
    }
    for dependency in &next.pending {
        domain.waits.dependents.insert((*dependency, number), ()).expect("bounded dependency edges");
    }
    for holding in &next.holdings {
        let key = (crate::holds::name(holding).clone(), number);
        if next.taken {
            domain.waits.holders.insert(key.clone(), ()).expect("bounded held edges");
        }
        if next.queued {
            domain.waits.waiters.insert(key, ()).expect("bounded queued edges");
        }
    }
    domain.waits.rows.insert(number, next).expect("one indexed row per live task");
}

fn has(holdings: &[Holding], holding: &Holding) -> bool {
    for known in holdings {
        if crate::holds::name(known) == crate::holds::name(holding) {
            return true;
        }
    }
    false
}

pub(crate) fn remove(domain: &mut Domain, number: u64) {
    if let Some(old) = domain.waits.rows.remove(&number) {
        for holding in &old.holdings {
            let key = (crate::holds::name(holding).clone(), number);
            domain.waits.holders.remove(&key);
            domain.waits.waiters.remove(&key);
        }
        for dependency in &old.pending {
            domain.waits.dependents.remove(&(*dependency, number));
        }
    }
    // The dependent publications that follow the end remove their own comparison entries.
    for _ in 0..domain.names.capacity() {
        let pair = match domain.waits.dependents.range((number, 0)..=(number, u64::MAX)).next() {
            Some((pair, ())) => *pair,
            None => break,
        };
        domain.waits.dependents.remove(&pair);
    }
    domain.circle_changes.insert(number).expect("changes drained before another lifecycle cascade");
}

fn threshold(domain: &Domain, holding: &Holding) -> Option<u32> {
    match crate::holds::rule(domain, holding) {
        Some(HoldKind::Exclusive { .. }) => Some(1),
        Some(HoldKind::Pooled { .. }) => Some(match domain.pools.get(crate::holds::name(holding)) {
            Some(pool) => pool.slots.max(1),
            None => 1,
        }),
        Some(HoldKind::Shared) | None => None,
    }
}

struct Back {
    markers: Map<u64, u64>,
    counts: Map<Name, u32>,
    stack: List<u64>,
}

impl Back {
    fn new(limits: &Limits) -> Self {
        Self {
            markers: Map::with_capacity(limits.tasks),
            counts: Map::with_capacity(resource_capacity(limits).expect("resource count")),
            stack: List::with_capacity(limits.tasks),
        }
    }

    fn mark(&mut self, task: u64, marker: u64) {
        if self.markers.contains_key(&task) {
            return;
        }
        self.markers.insert(task, marker).expect("one mark per live task");
        self.stack.push(task).expect("one stack entry per live task");
    }

    /// A task creator's transferred holds cease to be waits on it in this batch.
    fn walk(&mut self, domain: &Domain, task: u64, batch: &[New]) {
        self.mark(task, task);
        for at in 0..self.stack.capacity() {
            let Some(&number) = self.stack.get(at) else { break };
            for ((_, dependent), ()) in domain.waits.dependents.range((number, 0)..=(number, u64::MAX)) {
                if let Some(row) = record(domain, *dependent)
                    && dependencies_wait(&row.phase)
                {
                    self.mark(*dependent, number);
                }
            }
            let Some(row) = record(domain, number) else { continue };
            if let Party::Task(parent) = row.requester
                && let Some(requester) = record(domain, parent)
                && delegates_wait(&requester.phase)
                && crate::batch::contains(&requester.delegates, number)
            {
                self.mark(parent, number);
            }
            if !row.holds_taken {
                continue;
            }
            for holding in &row.holdings {
                if number == task && handed(batch, holding) {
                    continue;
                }
                let Some(threshold) = threshold(domain, holding) else { continue };
                let name = crate::holds::name(holding);
                let count = self.counts.get(name).copied().unwrap_or(0).saturating_add(1);
                self.counts.insert(name.clone(), count).expect("one count per indexed resource");
                if count != threshold {
                    continue;
                }
                for ((_, waiter), ()) in domain.waits.waiters.range((name.clone(), 0)..=(name.clone(), u64::MAX)) {
                    self.mark(*waiter, number);
                }
            }
        }
    }

    fn problem(&self, limits: &Limits, task: u64, first: u64) -> Problem {
        let mut tasks = List::with_capacity(limits.circle);
        let mut next = task;
        let mut cut = false;
        for at in 0..=limits.tasks {
            if tasks.push(next).is_err() {
                cut = true;
                break;
            }
            next = if at == 0 { first } else { *self.markers.get(&next).expect("marked circle member") };
            if next == task {
                break;
            }
        }
        Problem::naming(Some(task), Refusal::Circle { cut }, tasks.as_slice())
    }
}

fn handed(batch: &[New], holding: &Holding) -> bool {
    for member in batch {
        if has(&member.holdings, holding) {
            return true;
        }
    }
    false
}

/// Judge new dependencies by walking back from their waiter (domain/tasks.md, 6.7).
pub(crate) fn check_added(domain: &Domain, limits: &Limits, task: u64, added: &[u64]) -> Result<(), Problem> {
    let mut back = Back::new(limits);
    back.walk(domain, task, &[]);
    for dependency in added {
        if back.markers.contains_key(dependency) {
            return Err(back.problem(limits, task, *dependency));
        }
    }
    Ok(())
}

/// Judge the queue edges a waiting or released task would give back (domain/tasks.md, 6.2 and 6.7).
pub(crate) fn check_queue(domain: &Domain, limits: &Limits, task: u64) -> Result<(), Problem> {
    let Some(row) = record(domain, task) else { return Ok(()) };
    let mut back = Back::new(limits);
    back.walk(domain, task, &[]);
    for holding in &row.holdings {
        let Some(threshold) = threshold(domain, holding) else { continue };
        let name = crate::holds::name(holding);
        if back.counts.get(name).copied().unwrap_or(0) >= threshold {
            for ((_, holder), ()) in domain.waits.holders.range((name.clone(), 0)..=(name.clone(), u64::MAX)) {
                if back.markers.contains_key(holder) {
                    return Err(back.problem(limits, task, *holder));
                }
            }
        }
    }
    Ok(())
}

/// Leave a circling queue at its saved place, publishing the ordinary held notice (5.5 and 6.7).
pub(crate) fn hold_circle(
    domain: &mut Domain,
    env: &Env<Limits>,
    number: u64,
    problem: Problem,
    out: &mut Queue<Request>,
) {
    let cut = circle_cut(&problem.why);
    let tasks = problem.blocked_by.expect("circle names its tasks");
    let row = &mut task_mut(domain, number).expect("circling live task").record;
    let was = match &row.phase {
        Phase::Held { was, .. } => was.clone(),
        Phase::Waiting | Phase::Active(_) | Phase::Closing(_) | Phase::Ended(_) => crate::run::was(row.phase.clone()),
    };
    row.phase =
        Phase::Held { was, why: Hold::Circle, source: Some(Box::new(HoldSource::Circle(Circle { tasks, cut }))) };
    crate::holds::leave(domain, number);
    publish(domain, env, number, out);
}

/// Rejudge affected circle holds outside publication; a cut circle is always reconsidered.
pub(crate) fn lift(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) -> bool {
    if domain.circle_changes.is_empty() {
        return false;
    }
    let changes = core::mem::replace(&mut domain.circle_changes, Set::with_capacity(env.limits.tasks));
    let numbers = crate::domain::snapshot(domain, env.limits.tasks);
    let mut lifted = false;
    for number in numbers.into_boxed() {
        let Some(row) = record(domain, number) else { continue };
        let Some(circle) = circle_held(&row.phase) else { continue };
        let mut affected = circle.cut;
        for task in &circle.tasks {
            affected |= changes.contains(task);
        }
        if !affected {
            continue;
        }
        let resume = match &row.phase {
            Phase::Held { was: Was::Waiting, .. } => Phase::Waiting,
            Phase::Held { was: Was::Active(active), .. } => Phase::Active(*active),
            Phase::Held { was: Was::Closing(closing), .. } => Phase::Closing(closing.clone()),
            Phase::Waiting | Phase::Active(_) | Phase::Closing(_) | Phase::Ended(_) => unreachable!("circle hold"),
        };
        let verdict = check_waits(domain, &env.limits, number, &resume);
        let verdict = match verdict {
            Ok(()) if resume == Phase::Waiting && row.waiting_on.is_empty() && !row.holds_taken => {
                check_queue(domain, &env.limits, number)
            }
            other => other,
        };
        match verdict {
            Ok(()) => {
                task_mut(domain, number).expect("circle held task").record.phase = resume;
                if crate::holds::queued(record(domain, number).expect("lifted task")) {
                    crate::holds::back(domain, env, number, out);
                } else {
                    publish(domain, env, number, out);
                    if record(domain, number).expect("lifted task").phase == Phase::Active(crate::Active::Due) {
                        crate::domain::activate(domain, number, out);
                    }
                }
                lifted = true;
            }
            Err(problem) => {
                let cut = circle_cut(&problem.why);
                if circle.cut != cut || problem.blocked_by.as_deref() != Some(circle.tasks.as_ref()) {
                    hold_circle(domain, env, number, problem, out);
                }
            }
        }
    }
    lifted
}

#[derive(Debug)]
struct Gate {
    count: u32,
    threshold: u32,
}

pub(crate) fn pool_changed(domain: &mut Domain, limits: &Limits) {
    for (number, _) in &domain.names {
        if let Some(row) = record(domain, *number)
            && circle_held(&row.phase).is_some()
        {
            domain.circle_changes.insert(*number).expect("bounded circle holds");
        }
    }
    // `limits` also documents that the pending set shares the live task bound.
    assert!(domain.circle_changes.len() <= limits.tasks, "bounded circle changes");
}

struct Eliminate {
    pending: Map<u64, u32>,
    counts: Map<Name, u32>,
    ready: List<u64>,
}

impl Eliminate {
    fn new(limits: &Limits) -> Self {
        Self {
            pending: Map::with_capacity(limits.tasks),
            counts: Map::with_capacity(resource_capacity(limits).expect("resource count")),
            ready: List::with_capacity(limits.tasks),
        }
    }

    fn met(&mut self, number: u64) {
        if let Some(count) = self.pending.get_mut(&number) {
            *count = count.checked_sub(1).expect("each wait removed once");
            if *count == 0 {
                self.ready.push(number).expect("one ready entry per task");
            }
        }
    }
}

/// Judge the restored set whole by ending the tasks that can end, with threshold counters.
/// A dependency/delegate remainder alone is corrupt; a queue circle becomes a hold (6.7).
pub(crate) fn restore(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) -> bool {
    for _ in 0..=env.limits.tasks {
        let remaining = eliminate(domain, &env.limits);
        if remaining.is_empty() {
            return true;
        }
        let mut earliest = None;
        for (number, _) in &remaining {
            let row = record(domain, *number).expect("remaining restored task");
            if !crate::holds::queued(row) {
                continue;
            }
            if let Err(problem) = check_queue(domain, &env.limits, *number) {
                match &earliest {
                    Some((old, _)) if crate::holds::earlier(domain, *old, *number, &env.limits) => {}
                    Some(_) | None => earliest = Some((*number, problem)),
                }
            }
        }
        let Some((number, problem)) = earliest else { return false };
        hold_circle(domain, env, number, problem, out);
    }
    false
}

fn eliminate(domain: &Domain, limits: &Limits) -> Map<u64, u32> {
    let mut judge = Eliminate::new(limits);
    for ((name, _), ()) in &domain.waits.holders {
        let count = judge.counts.get(name).copied().unwrap_or(0).saturating_add(1);
        judge.counts.insert(name.clone(), count).expect("bounded resources");
    }
    for (number, _) in &domain.names {
        let row = record(domain, *number).expect("restored task");
        let mut pending = 0_u32;
        if dependencies_wait(&row.phase) {
            pending = u32::try_from(row.waiting_on.len()).expect("bounded dependencies");
        }
        if delegates_wait(&row.phase) {
            pending = pending
                .checked_add(u32::try_from(row.delegates.len()).expect("bounded delegates"))
                .expect("validated wait counters");
        }
        if crate::holds::queued(row) {
            for holding in &row.holdings {
                if let Some(threshold) = threshold(domain, holding)
                    && judge.counts.get(crate::holds::name(holding)).copied().unwrap_or(0) >= threshold
                {
                    pending = pending.checked_add(1).expect("bounded waits");
                }
            }
        }
        judge.pending.insert(*number, pending).expect("bounded tasks");
        if pending == 0 {
            judge.ready.push(*number).expect("one ready entry per task");
        }
    }
    for at in 0..limits.tasks {
        let Some(&number) = judge.ready.get(at) else { break };
        judge.pending.remove(&number);
        for ((_, dependent), ()) in domain.waits.dependents.range((number, 0)..=(number, u64::MAX)) {
            if let Some(row) = record(domain, *dependent)
                && dependencies_wait(&row.phase)
            {
                judge.met(*dependent);
            }
        }
        let row = record(domain, number).expect("eliminated live row");
        if let Party::Task(parent) = row.requester
            && let Some(row) = record(domain, parent)
            && delegates_wait(&row.phase)
        {
            judge.met(parent);
        }
        if !row.holds_taken {
            continue;
        }
        for holding in &row.holdings {
            let Some(threshold) = threshold(domain, holding) else { continue };
            let name = crate::holds::name(holding);
            let count = judge.counts.get_mut(name).expect("indexed held resource");
            let before = *count;
            *count = count.checked_sub(1).expect("held resource counted once");
            if before != threshold {
                continue;
            }
            for ((_, waiter), ()) in domain.waits.waiters.range((name.clone(), 0)..=(name.clone(), u64::MAX)) {
                judge.met(*waiter);
            }
        }
    }
    judge.pending
}

/// Judge a creator's added delegation wait by a back walk, and new members by Kahn counters.
/// Earlier declarations and hand-downs count as in whole-set hold admission (4; 6.7).
pub(crate) fn check_batch(domain: &Domain, limits: &Limits, creator: Party, batch: &[New]) -> Result<(), Problem> {
    let mut back = Back::new(limits);
    let parent = match creator {
        Party::Task(number) if delegates_wait(&record(domain, number).expect("admitted creator").phase) => {
            back.walk(domain, number, batch);
            Some(number)
        }
        Party::Task(_) | Party::Person(_) | Party::Deployment { .. } => None,
    };
    let mut judge = Eliminate::new(limits);
    let mut dependencies = Map::with_capacity(limits.batch.checked_mul(limits.dependencies).expect("batch edges"));
    let mut gates = Map::with_capacity(limits.batch.checked_mul(limits.holdings).expect("batch gates"));
    for (at, member) in batch.iter().enumerate() {
        let mut pending = 0_u32;
        for dependency in &member.dependencies {
            if back.markers.contains_key(dependency) {
                back.mark(member.number, *dependency);
                return Err(back.problem(limits, parent.expect("creator's back walk"), member.number));
            }
            if member_named(batch, *dependency) {
                dependencies.insert((*dependency, member.number), ()).expect("bounded new dependencies");
                pending = pending.checked_add(1).expect("bounded waits");
            }
        }
        if member.dependencies.is_empty() {
            for holding in &member.holdings {
                let Some(threshold) = threshold(domain, holding) else { continue };
                let name = crate::holds::name(holding);
                let mut earlier = 0_u32;
                for prior in batch.iter().take(at) {
                    if has(&prior.holdings, holding) {
                        earlier = earlier.saturating_add(1);
                    }
                }
                let mut marked = 0_u32;
                let mut first = None;
                let mut transfer = false;
                for ((_, holder), ()) in domain.waits.holders.range((name.clone(), 0)..=(name.clone(), u64::MAX)) {
                    if creator == Party::Task(*holder) {
                        transfer = true;
                        continue;
                    }
                    if back.markers.contains_key(holder) {
                        marked = marked.saturating_add(1);
                        if first.is_none() {
                            first = Some(*holder);
                        }
                    }
                }
                if transfer && earlier == 0 {
                    continue;
                }
                if marked >= threshold {
                    back.mark(member.number, first.expect("marked resource holder"));
                    return Err(back.problem(limits, parent.expect("creator's back walk"), member.number));
                }
                let count = marked.saturating_add(earlier);
                if count >= threshold {
                    gates.insert((name.clone(), member.number), Gate { count, threshold }).expect("bounded gates");
                    pending = pending.checked_add(1).expect("bounded waits");
                }
            }
        }
        judge.pending.insert(member.number, pending).expect("bounded members");
        if pending == 0 {
            judge.ready.push(member.number).expect("one ready entry per member");
        }
    }
    for at in 0..limits.batch {
        let Some(&number) = judge.ready.get(at) else { break };
        judge.pending.remove(&number);
        for ((_, dependent), ()) in dependencies.range((number, 0)..=(number, u64::MAX)) {
            judge.met(*dependent);
        }
        let mut after = false;
        let member = member(batch, number);
        for later in batch {
            if later.number == number {
                after = true;
                continue;
            }
            if !after {
                continue;
            }
            for holding in &member.holdings {
                if let Some(gate) = gates.get_mut(&(crate::holds::name(holding).clone(), later.number)) {
                    let before = gate.count;
                    gate.count = gate.count.checked_sub(1).expect("earlier holder counted");
                    if before == gate.threshold {
                        judge.met(later.number);
                    }
                }
            }
        }
    }
    if judge.pending.is_empty() {
        return Ok(());
    }
    Err(batch_circle(limits, batch, &judge.pending, &gates))
}

fn batch_circle(limits: &Limits, batch: &[New], pending: &Map<u64, u32>, gates: &Map<(Name, u64), Gate>) -> Problem {
    // Pick one remaining wait per member, then follow until the path repeats.
    let mut circle = Back::new(limits);
    for (number, _) in pending {
        let member = member(batch, *number);
        let mut next = None;
        for dependency in &member.dependencies {
            if pending.contains_key(dependency) {
                next = Some(*dependency);
                break;
            }
        }
        if next.is_none() {
            for holding in &member.holdings {
                if let Some(gate) = gates.get(&(crate::holds::name(holding).clone(), *number))
                    && gate.count >= gate.threshold
                {
                    for prior in batch {
                        if prior.number == *number {
                            break;
                        }
                        if pending.contains_key(&prior.number) && has(&prior.holdings, holding) {
                            next = Some(prior.number);
                            break;
                        }
                    }
                }
                if next.is_some() {
                    break;
                }
            }
        }
        circle.markers.insert(*number, next.expect("remaining member has a remaining wait")).expect("bounded markers");
    }
    let mut next = *pending.first().expect("remaining circle").0;
    let mut seen = Set::with_capacity(limits.tasks);
    for _ in 0..=limits.tasks {
        if !seen.insert(next).expect("bounded visited members") {
            return circle.problem(limits, next, *circle.markers.get(&next).expect("remaining wait"));
        }
        next = *circle.markers.get(&next).expect("remaining wait");
    }
    unreachable!("finite remaining wait graph repeats")
}

fn member_named(batch: &[New], number: u64) -> bool {
    for member in batch {
        if member.number == number {
            return true;
        }
    }
    false
}

fn member(batch: &[New], number: u64) -> &New {
    for member in batch {
        if member.number == number {
            return member;
        }
    }
    unreachable!("admitted batch member")
}

fn dependencies_wait(phase: &Phase) -> bool {
    match phase {
        Phase::Waiting | Phase::Active(_) => true,
        Phase::Closing(_) | Phase::Held { .. } | Phase::Ended(_) => false,
    }
}

fn delegates_wait(phase: &Phase) -> bool {
    match phase {
        Phase::Waiting | Phase::Active(_) | Phase::Closing(_) => true,
        Phase::Held { .. } | Phase::Ended(_) => false,
    }
}

fn is_held(phase: &Phase) -> bool {
    match phase {
        Phase::Held { .. } => true,
        Phase::Waiting | Phase::Active(_) | Phase::Closing(_) | Phase::Ended(_) => false,
    }
}

fn circle_held(phase: &Phase) -> Option<&Circle> {
    match phase {
        Phase::Held { why: Hold::Circle, source: Some(source), .. } => match source.as_ref() {
            HoldSource::Circle(circle) => Some(circle),
            HoldSource::Resource(_) | HoldSource::Effect { .. } => None,
        },
        Phase::Waiting | Phase::Active(_) | Phase::Closing(_) | Phase::Held { .. } | Phase::Ended(_) => None,
    }
}

fn circle_cut(why: &Refusal) -> bool {
    match why {
        Refusal::Circle { cut } => *cut,
        Refusal::Account { .. }
        | Refusal::Grace
        | Refusal::Goals
        | Refusal::Unproposable
        | Refusal::PersonRevision { .. }
        | Refusal::ResourceUnavailable
        | Refusal::NotReady
        | Refusal::Unknown
        | Refusal::Busy
        | Refusal::Duplicate
        | Refusal::Empty
        | Refusal::Batch
        | Refusal::Live
        | Refusal::Project
        | Refusal::Tree
        | Refusal::Depth
        | Refusal::Delegates
        | Refusal::Reference
        | Refusal::Subscription
        | Refusal::Dependencies
        | Refusal::DependencyEnded { .. }
        | Refusal::Kept
        | Refusal::Executor
        | Refusal::Spec
        | Refusal::Contract
        | Refusal::AuthorityShape
        | Refusal::Inputs
        | Refusal::State
        | Refusal::Attempt
        | Refusal::LiveDelegates
        | Refusal::Restore
        | Refusal::Read
        | Refusal::Turn
        | Refusal::Funding
        | Refusal::HoldKind
        | Refusal::HoldTaken
        | Refusal::Holds
        | Refusal::Llm { .. } => unreachable!("circle judge"),
    }
}

/// A release also restores dependency and delegation waits before it may go on (6.7).
pub(crate) fn check_return(domain: &Domain, limits: &Limits, task: u64) -> Result<(), Problem> {
    let phase = &record(domain, task).expect("released live row").phase;
    check_waits(domain, limits, task, phase)
}

fn check_waits(domain: &Domain, limits: &Limits, task: u64, phase: &Phase) -> Result<(), Problem> {
    let row = record(domain, task).expect("released live row");
    if dependencies_wait(phase) {
        check_added(domain, limits, task, &row.waiting_on)?;
    }
    let mut back = Back::new(limits);
    back.walk(domain, task, &[]);
    if delegates_wait(phase) {
        for delegate in &row.delegates {
            if back.markers.contains_key(delegate) {
                return Err(back.problem(limits, task, *delegate));
            }
        }
    }
    Ok(())
}
