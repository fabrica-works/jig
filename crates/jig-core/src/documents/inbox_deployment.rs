//! Party pages and deployment facts (`client/domain/pages.md`, sections 3, 4
//! and 7; `contract.md`, 5 and 7.1). These are bounded cache views. Exact
//! cross-project counts, reply read positions and stored inbox rows come later.
use super::{BudgetLine, Chip, Count, Cursor, Missing, Proposer, TaskLink, Window};
use crate::{Core, Limits};
use alloc::boxed::Box;
use jig_core_accounts as accounts;
use jig_core_fleet as fleet;
use jig_core_people as people;
use jig_core_tasks as tasks;
use skein_lib::{Duration, List, Map, Time, Wall};

/// The object one inbox row opens; its own document carries content and acts.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum InboxObject {
    Question { message: u64 },
    Proposal { proposer: Proposer, number: u64 },
    Escalation { revision: u64 },
    PersonTask,
    Reply { message: u64 },
    Result { position: u64 },
}
/// A result's cached reference has no project/time today; a store read fills it.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct InboxEntry {
    pub task: TaskLink,
    pub project: Option<u32>,
    pub object: InboxObject,
    pub at: Option<Wall>,
    pub unread: Option<bool>,
    pub cursor: Cursor,
}
/// A bounded page across both groups. Needs precede updates. Known-time needs
/// are oldest first and updates newest first; missing result times are explicit.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Inbox {
    pub party: u64,
    pub needs: List<InboxEntry>,
    pub updates: List<InboxEntry>,
    pub next: Option<Cursor>,
    pub cached_needs: u32,
    pub cached_updates: Count,
    pub updates_time_complete: bool,
}
/// One live directly requested chat, with cache-known replies only.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ConversationLine {
    pub chip: Chip,
    pub project: u32,
    pub replies_unread: Option<Count>,
    pub cached_replies: u32,
    pub delegates: u32,
    pub budget: BudgetLine,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Conversations {
    pub lines: List<ConversationLine>,
    pub next: Option<Cursor>,
}
/// Clock supplied by the decision caller; deployment timers are volatile
/// monotonic facts and become wall times only against this injected pair.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DocumentClock {
    pub now: Time,
    pub wall: Wall,
}
/// Placement facts do not retain a lost channel's identity, which fleet forgets.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RunLocation {
    None,
    Connected { channel: u64, kind: fleet::HostKind },
    Adrift { until: Option<Wall>, remaining: Option<Duration> },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct HostRun {
    pub task: TaskLink,
    pub attempt: u64,
    pub kept: u32,
    pub listed: bool,
    pub state: fleet::AttemptState,
    pub location: RunLocation,
    pub grace_until: Option<Wall>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Host {
    pub channel: u64,
    pub kind: fleet::HostKind,
    pub slots: u32,
    pub used: u32,
    pub free: u32,
    pub draining: bool,
    pub runs: List<HostRun>,
    pub cut: bool,
}
/// Secret-free account status, with runs using the configured account and tasks
/// currently waiting for it; no secret or protocol refresh token crosses here.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[expect(clippy::struct_excessive_bools, reason = "independent child-owned flags plus document truncation")]
pub struct Account {
    pub number: u32,
    pub generation: u64,
    pub state: accounts::State,
    pub usable: bool,
    pub unsaved: bool,
    pub attention: bool,
    pub spent_until: Option<Wall>,
    pub runs: List<Chip>,
    pub waiting: List<Chip>,
    pub cut: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Capacity {
    pub used: u32,
    pub limit: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ProjectCapacity {
    pub project: u32,
    pub tasks: Capacity,
    pub proposals: u32,
}
/// Connected hosts and all tracked attempts (including adrift), account status,
/// and authentic finite capacities. Refusal history and connector panels are
/// owned by their facts/store and application subjects, not a new cache here.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Deployment {
    pub hosts: List<Host>,
    pub attempts: List<HostRun>,
    pub accounts: List<Account>,
    pub tasks: Capacity,
    pub workers: Capacity,
    pub attempts_capacity: Capacity,
    pub waiting_attempts: u32,
    pub proposals: Capacity,
    pub projects: List<ProjectCapacity>,
    pub grace_bound: Duration,
    pub cut: bool,
}

fn line_bound(limits: &Limits) -> usize {
    usize::try_from(limits.documents.lines).expect("line bound fits")
}
fn object(task: u64, kind: people::EntryKind) -> InboxObject {
    match kind {
        people::EntryKind::Question { message } => InboxObject::Question { message },
        people::EntryKind::Proposal { number } => InboxObject::Proposal { proposer: Proposer::Task(task), number },
        people::EntryKind::Escalation { revision } => InboxObject::Escalation { revision },
        people::EntryKind::PersonTask => InboxObject::PersonTask,
        people::EntryKind::Reply { message } => InboxObject::Reply { message },
    }
}
fn updates(value: InboxObject) -> bool {
    match value {
        InboxObject::Reply { .. } | InboxObject::Result { .. } => true,
        InboxObject::Question { .. }
        | InboxObject::Proposal { .. }
        | InboxObject::Escalation { .. }
        | InboxObject::PersonTask => false,
    }
}
fn key(cursor: Cursor) -> Result<(bool, u64, u64, u64, InboxObject), Missing> {
    match cursor {
        Cursor::Inbox { task, object, at } => {
            let update = updates(object);
            let time = match at {
                Some(at) => at.as_nanos(),
                None => 0,
            };
            let order = match object {
                InboxObject::Result { position } => u64::MAX.checked_sub(position).expect("position fits"),
                InboxObject::Reply { message } => u64::MAX.checked_sub(message).expect("message fits"),
                InboxObject::Question { .. }
                | InboxObject::Proposal { .. }
                | InboxObject::Escalation { .. }
                | InboxObject::PersonTask => task,
            };
            Ok((
                update,
                if update { u64::MAX.checked_sub(time).expect("wall time fits") } else { time },
                order,
                task,
                object,
            ))
        }
        Cursor::Number { .. } => Err(Missing::Unknown),
    }
}
fn result_project(core: &Core, task: u64) -> Option<u32> {
    if let Some(row) = core.tasks.task(task) {
        return Some(row.project);
    }
    for (_, projection) in &core.projections {
        for row in &projection.plan {
            if row.number == task {
                return Some(projection.goal.project);
            }
        }
    }
    None
}
fn result_time(core: &Core, task: u64) -> Option<Wall> {
    if let Some(row) = core.tasks.task(task) {
        return row.ended_at;
    }
    for (_, projection) in &core.projections {
        for row in &projection.plan {
            if row.number == task {
                match &row.phase {
                    tasks::Phase::Ended(tasks::Ending::Cancelled { at, .. }) => return Some(*at),
                    tasks::Phase::Ended(tasks::Ending::Done(_) | tasks::Ending::Failed { .. })
                    | tasks::Phase::Waiting
                    | tasks::Phase::Active(_)
                    | tasks::Phase::Closing(_)
                    | tasks::Phase::Held { .. } => return None,
                }
            }
        }
    }
    None
}
pub(super) fn inbox(core: &Core, limits: &Limits, party: u64, window: &Window) -> Result<Inbox, Missing> {
    let entries = core.people.cached_waiting(&limits.people, party).ok_or(Missing::Unknown)?;
    let results = core.people.cached_results(party).ok_or(Missing::Unknown)?;
    let mut ordered = Map::with_capacity(limits.people.inbox_entries.checked_mul(2).ok_or(Missing::TooLarge)?);
    let mut cached_needs = 0_u32;
    let mut cached_updates = 0_u32;
    let mut times = true;
    for entry in &entries {
        if core.people.role(party, entry.project).is_none() {
            continue;
        }
        let value = object(entry.task, entry.kind);
        if updates(value) {
            cached_updates = cached_updates.checked_add(1).expect("bounded cache");
        } else {
            cached_needs = cached_needs.checked_add(1).expect("bounded cache");
        }
        let cursor = Cursor::Inbox { task: entry.task, object: value, at: Some(entry.at) };
        ordered
            .insert(key(cursor)?, (entry.task, Some(entry.project), value, Some(entry.at), None, cursor))
            .expect("two bounded caches");
    }
    for entry in results {
        let project = result_project(core, entry.task);
        if let Some(project) = project
            && core.people.role(party, project).is_none()
        {
            continue;
        }
        cached_updates = cached_updates.checked_add(1).expect("bounded cache");
        let at = result_time(core, entry.task);
        times &= at.is_some();
        let value = InboxObject::Result { position: entry.position };
        let cursor = Cursor::Inbox { task: entry.task, object: value, at };
        ordered.insert(key(cursor)?, (entry.task, project, value, at, Some(true), cursor)).expect("two bounded caches");
    }
    let after = match window.after {
        Some(cursor) => Some(key(cursor)?),
        None => None,
    };
    let mut needs = List::with_capacity(window.rows);
    let mut changes = List::with_capacity(window.rows);
    let mut used = 0_u32;
    let mut last = None;
    let mut more = false;
    for (position, (task, project, object, at, unread, cursor)) in &ordered {
        if let Some(after) = after
            && *position <= after
        {
            continue;
        }
        if used == window.rows {
            more = true;
            break;
        }
        let entry = InboxEntry {
            task: super::task_tree::link(core, limits, party, *task)?,
            project: *project,
            object: *object,
            at: *at,
            unread: *unread,
            cursor: *cursor,
        };
        if updates(*object) {
            changes.push(entry).expect("bounded updates");
        } else {
            needs.push(entry).expect("bounded needs");
        }
        used = used.checked_add(1).expect("bounded window");
        last = Some(*cursor);
    }
    Ok(Inbox {
        party,
        needs,
        updates: changes,
        next: if more { last } else { None },
        cached_needs,
        cached_updates: if cached_updates >= limits.people.inbox_entries {
            Count::AtLeast(cached_updates)
        } else {
            Count::Known(cached_updates)
        },
        updates_time_complete: times,
    })
}
pub(super) fn conversations(
    core: &Core,
    limits: &Limits,
    party: u64,
    window: &Window,
) -> Result<Conversations, Missing> {
    let after = match window.after {
        Some(Cursor::Number { number }) => number,
        Some(Cursor::Inbox { .. }) => return Err(Missing::Unknown),
        None => 0,
    };
    let waiting = core.people.cached_waiting(&limits.people, party).ok_or(Missing::Unknown)?;
    let mut lines = List::with_capacity(window.rows);
    let mut next = None;
    let mut more = false;
    for row in core.tasks.view_tasks().as_ref() {
        if row.number <= after {
            continue;
        }
        let row = core.tasks.task(row.number).expect("live task view");
        let chat = row.requester == tasks::Party::Person(party)
            && match row.executor {
                tasks::Executor::Agent { charter } => charter == core.settings.charter,
                tasks::Executor::Person(_) | tasks::Executor::Procedure { .. } => false,
            };
        if !chat || row.tracked.is_some() || row.recurring.is_some() || core.people.role(party, row.project).is_none() {
            continue;
        }
        if lines.room() == 0 {
            more = true;
            break;
        }
        let mut replies = 0_u32;
        for entry in &waiting {
            if entry.task == row.number {
                match entry.kind {
                    people::EntryKind::Reply { .. } => replies = replies.checked_add(1).expect("bounded replies"),
                    people::EntryKind::Question { .. }
                    | people::EntryKind::Proposal { .. }
                    | people::EntryKind::Escalation { .. }
                    | people::EntryKind::PersonTask => {}
                }
            }
        }
        lines
            .push(ConversationLine {
                chip: super::small::chip_from_row(limits, row),
                project: row.project,
                replies_unread: None,
                cached_replies: replies,
                delegates: u32::try_from(row.delegates.len()).expect("bounded delegates"),
                budget: super::task_tree::budget(row),
            })
            .expect("bounded conversation rows");
        next = Some(Cursor::Number { number: row.number });
    }
    Ok(Conversations { lines, next: if more { next } else { None } })
}
fn wall_deadline(clock: Option<DocumentClock>, at: Time) -> Option<Wall> {
    let clock = clock?;
    if at >= clock.now {
        Some(Wall::from_nanos(clock.wall.as_nanos().saturating_add(at.saturating_since(clock.now).as_nanos())))
    } else {
        Some(Wall::from_nanos(clock.wall.as_nanos().saturating_sub(clock.now.saturating_since(at).as_nanos())))
    }
}
#[expect(clippy::manual_map, reason = "step code has no closures (programming-model.md, section 10)")]
fn host_run(core: &Core, limits: &Limits, clock: Option<DocumentClock>, value: &fleet::AttemptView) -> HostRun {
    let location = match value.placement {
        fleet::Placement::None => RunLocation::None,
        fleet::Placement::Connected { channel, kind } => RunLocation::Connected { channel: channel.raw(), kind },
        fleet::Placement::Adrift { until } => RunLocation::Adrift {
            until: wall_deadline(clock, until),
            remaining: match clock {
                Some(clock) => Some(until.saturating_since(clock.now)),
                None => None,
            },
        },
    };
    HostRun {
        task: match core.tasks.task(value.run.raw()) {
            Some(row) => TaskLink::Live(super::small::chip_from_row(limits, row)),
            None => TaskLink::Stored { number: value.run.raw(), status: None },
        },
        attempt: value.attempt.raw(),
        kept: value.kept,
        listed: value.listed,
        state: value.state,
        location,
        grace_until: match value.until {
            Some(until) => wall_deadline(clock, until),
            None => None,
        },
    }
}
#[expect(
    clippy::too_many_lines,
    reason = "one deployment document gathers bounded child-owned host, account and capacity facts"
)]
pub(super) fn deployment(
    core: &Core,
    limits: &Limits,
    party: u64,
    clock: Option<DocumentClock>,
) -> Result<Deployment, Missing> {
    if core.people.role(party, 0).is_none() {
        return Err(Missing::NotAllowed);
    }
    let source_hosts = core.fleet.host_views();
    let source_runs = core.fleet.attempt_views();
    let mut hosts = List::with_capacity(limits.documents.lines);
    let mut cut = false;
    for value in source_hosts.iter().take(line_bound(limits)) {
        let mut runs = List::with_capacity(limits.documents.lines);
        let mut host_cut = false;
        for run in &source_runs {
            match run.placement {
                fleet::Placement::Connected { channel, .. } if channel == value.channel => {
                    if runs.room() > 0 {
                        runs.push(host_run(core, limits, clock, run)).expect("bounded host runs");
                    } else {
                        host_cut = true;
                    }
                }
                fleet::Placement::None | fleet::Placement::Connected { .. } | fleet::Placement::Adrift { .. } => {}
            }
        }
        hosts
            .push(Host {
                channel: value.channel.raw(),
                kind: value.kind,
                slots: value.slots,
                used: value.used,
                free: value.slots.checked_sub(value.used).expect("host occupants fit slots"),
                draining: value.draining,
                runs,
                cut: host_cut,
            })
            .expect("bounded hosts");
    }
    cut |= usize::try_from(source_hosts.len()).expect("host count fits") > line_bound(limits);
    let mut attempts = List::with_capacity(limits.documents.lines);
    for value in source_runs.iter().take(line_bound(limits)) {
        attempts.push(host_run(core, limits, clock, value)).expect("bounded attempt rows");
    }
    cut |= usize::try_from(source_runs.len()).expect("attempt count fits") > line_bound(limits);
    let rows = core.tasks.view_tasks();
    let mut accounts = List::with_capacity(limits.documents.lines);
    let source_accounts = core.accounts.account_views();
    for value in source_accounts.iter().take(line_bound(limits)) {
        let mut runs = List::with_capacity(limits.documents.lines);
        let mut waiting = List::with_capacity(limits.documents.lines);
        let mut account_cut = false;
        if value.account == core.settings.account {
            for row in &rows {
                let row = core.tasks.task(row.number).expect("live view task");
                let agent = match row.executor {
                    tasks::Executor::Agent { .. } => true,
                    tasks::Executor::Person(_) | tasks::Executor::Procedure { .. } => false,
                };
                if !agent {
                    continue;
                }
                let running = match core.proofs.get(&row.number) {
                    Some(proof) => proof.terminal.is_none(),
                    None => false,
                };
                let needs = !value.usable
                    && match row.phase {
                        tasks::Phase::Active(tasks::Active::Due | tasks::Active::Preparing) => true,
                        tasks::Phase::Waiting
                        | tasks::Phase::Active(_)
                        | tasks::Phase::Closing(_)
                        | tasks::Phase::Held { .. }
                        | tasks::Phase::Ended(_) => false,
                    };
                if running {
                    if runs.room() > 0 {
                        runs.push(super::small::chip_from_row(limits, row)).expect("bounded account runs");
                    } else {
                        account_cut = true;
                    }
                }
                if needs {
                    if waiting.room() > 0 {
                        waiting.push(super::small::chip_from_row(limits, row)).expect("bounded waiting agents");
                    } else {
                        account_cut = true;
                    }
                }
            }
        }
        accounts
            .push(Account {
                number: value.account,
                generation: value.generation,
                state: value.state,
                usable: value.usable,
                unsaved: value.unsaved,
                attention: value.attention,
                spent_until: match value.spent_until {
                    Some(at) => wall_deadline(clock, at),
                    None => None,
                },
                runs,
                waiting,
                cut: account_cut,
            })
            .expect("bounded accounts");
    }
    cut |= usize::try_from(source_accounts.len()).expect("account count fits") > line_bound(limits);
    let mut proposal_count = 0_u32;
    let mut projects = List::with_capacity(limits.documents.lines);
    for project in &core.projects {
        let mut live = 0_u32;
        let mut proposed = 0_u32;
        for row in &rows {
            if row.project == *project {
                live = live.checked_add(1).expect("bounded live tasks");
                let row = core.tasks.task(row.number).expect("live task view");
                if let Some(value) = &row.proposal {
                    match value.state {
                        tasks::ProposalState::Pending { .. } => {
                            proposed = proposed.checked_add(1).expect("one proposal per live task");
                        }
                        tasks::ProposalState::Accepted { .. }
                        | tasks::ProposalState::Rejected { .. }
                        | tasks::ProposalState::Withdrawn => {}
                    }
                }
            }
        }
        for (_, value) in core.tasks.person_proposals() {
            if value.project == *project {
                proposed = proposed.checked_add(1).expect("bounded person proposals");
            }
        }
        proposal_count = proposal_count.checked_add(proposed).expect("bounded proposal census");
        if projects.room() > 0 {
            projects
                .push(ProjectCapacity {
                    project: *project,
                    tasks: Capacity { used: live, limit: limits.tasks.project_tasks },
                    proposals: proposed,
                })
                .expect("bounded project capacities");
        } else {
            cut = true;
        }
    }
    Ok(Deployment {
        hosts,
        attempts,
        accounts,
        tasks: Capacity { used: u32::try_from(rows.len()).expect("bounded live rows"), limit: limits.tasks.tasks },
        workers: Capacity { used: core.fleet.workers(), limit: limits.fleet.workers },
        attempts_capacity: Capacity {
            used: core.fleet.attempts(),
            limit: limits
                .fleet
                .attempts
                .checked_add(limits.fleet.workers.checked_mul(limits.fleet.slots).ok_or(Missing::TooLarge)?)
                .ok_or(Missing::TooLarge)?,
        },
        waiting_attempts: core.fleet.waiting(),
        proposals: Capacity {
            used: proposal_count,
            limit: limits.tasks.tasks.checked_mul(2).ok_or(Missing::TooLarge)?,
        },
        projects,
        grace_bound: limits.fleet.grace,
        cut,
    })
}
/// Construct a document with the decision caller's injected clock. Other
/// subjects need no clock; deployment's grace/cooldown facts become wall times.
pub fn make_with_clock(
    core: &Core,
    limits: &Limits,
    party: u64,
    subject: &super::Subject,
    window: &Window,
    clock: DocumentClock,
) -> Result<super::Document, Missing> {
    match subject {
        super::Subject::Deployment => {
            core.people.identity(party).ok_or(Missing::Unknown)?;
            if super::document_worst_case(&limits.documents).is_none() || window.rows > limits.documents.rows {
                return Err(Missing::TooLarge);
            }
            Ok(super::Document::Deployment(Box::new(deployment(core, limits, party, Some(clock))?)))
        }
        super::Subject::Frame { .. }
        | super::Subject::Task { .. }
        | super::Subject::Tree { .. }
        | super::Subject::Chips { .. }
        | super::Subject::Inbox { .. }
        | super::Subject::Goals { .. }
        | super::Subject::Conversations { .. }
        | super::Subject::Run { .. }
        | super::Subject::Proposal { .. }
        | super::Subject::Escalation { .. }
        | super::Subject::Question { .. }
        | super::Subject::PersonTask { .. }
        | super::Subject::Result { .. }
        | super::Subject::Project { .. }
        | super::Subject::Note { .. } => super::make(core, limits, party, subject, window),
    }
}
pub(super) fn worst_case(limits: &super::DocumentLimits) -> Option<u64> {
    let carriers = u64::from(limits.lines)
        .checked_mul(u64::from(limits.lines))?
        .checked_add(u64::from(limits.rows).checked_mul(2)?)?
        .checked_add(u64::from(limits.lines).checked_mul(3)?)?
        .checked_add(4)?;
    // One nested host-run/task/account carrier is smaller than a task page,
    // whose bound prices all its chip/part/list children. Hosts nest run lists.
    super::task_tree::worst_case(limits)?
        .checked_mul(carriers)?
        .checked_add(u64::try_from(size_of::<Deployment>()).ok()?)
}

pub(super) fn scratch_worst_case(limits: &Limits) -> Option<u64> {
    let tracked = limits
        .fleet
        .workers
        .checked_mul(limits.fleet.slots)?
        .checked_add(limits.fleet.engine_slots)?
        .checked_add(limits.fleet.attempts)?;
    List::<fleet::HostView>::worst_case(limits.fleet.workers.checked_add(u32::from(limits.fleet.engine_slots > 0))?)?
        .checked_add(List::<fleet::AttemptView>::worst_case(tracked)?)?
        .checked_add(List::<accounts::Fact>::worst_case(limits.accounts.accounts)?)?
        .checked_add(Map::<
            (bool, u64, u64, u64, InboxObject),
            (u64, Option<u32>, InboxObject, Option<Wall>, Option<bool>, Cursor),
        >::worst_case(limits.people.inbox_entries.checked_mul(2)?)?)?
        .checked_add(List::<people::Entry>::worst_case(limits.people.inbox_entries.checked_mul(3)?)?)
}
