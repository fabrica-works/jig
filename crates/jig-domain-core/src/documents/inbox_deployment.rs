//! Party pages and deployment facts (`client/domain/pages.md`, sections 3, 4
//! and 7; `contract.md`, 5 and 7.1). Entries are bounded cache views;
//! needs count all live task projections across the party's projects. Reply
//! read positions and stored inbox rows are supplied by their owners.
use super::{BudgetLine, Chip, Count, Cursor, Missing, Proposer, TaskLink, Window};
use crate::{Core, Limits};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
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
/// One owner-supplied inbox reference, with a chat header when it is a reply.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct InboxEntry {
    pub reply: Option<Box<ConversationLine>>,
    pub task: TaskLink,
    pub project: Option<u32>,
    pub object: InboxObject,
    /// Exact child activity to load, separate from a chat ordinal or notice message number.
    pub activity: Option<u64>,
    pub at: Option<Wall>,
    pub unread: Option<bool>,
    pub cursor: Cursor,
}
/// A bounded page across both groups. Needs precede updates. Known-time needs
/// are oldest first and updates newest first.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Inbox {
    pub party: u64,
    pub needs: List<InboxEntry>,
    pub updates: List<InboxEntry>,
    pub next: Option<Cursor>,
    /// Exact needs across every project the party currently holds a role in.
    pub cached_needs: u32,
    pub cached_updates: Count,
    pub results_seen: u64,
    pub updates_time_complete: bool,
    /// None is unsupplied; Some carries complete roots for each emitted row.
    pub selection: Option<List<super::InboxSelection>>,
}
/// One directly requested chat, with committed reply positions.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ConversationLine {
    pub chip: Chip,
    pub project: u32,
    pub first_words: super::Text,
    pub seen: u64,
    pub last_reply: u64,
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
/// Lost identity is present when this engine observed the channel before loss.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RunLocation {
    None,
    Connected { channel: u64, kind: fleet::HostKind },
    Adrift { channel: Option<u64>, until: Option<Wall>, remaining: Option<Duration> },
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
    pub connected: bool,
    pub grace_until: Option<Wall>,
    pub grace_remaining: Option<Duration>,
    pub runs: List<HostRun>,
    pub cut: bool,
}
/// Secret-free account status, with runs using the configured account and tasks
/// currently waiting for it; no secret or protocol refresh token crosses here.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[expect(clippy::struct_excessive_bools, reason = "independent child-owned flags plus document truncation")]
pub struct Account {
    pub number: u32,
    pub generation: Option<u64>,
    pub state: accounts::State,
    pub usable: bool,
    pub expiring: bool,
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
/// Deployment facts and configured connector references; connector values remain
/// their owners' documents (`client/domain/pages.md`, 7; `client/domain/root.md`, 6).
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
    pub hello_refusals: List<HelloRefusalLine>,
    pub hosts_forgotten: u64,
    pub hello_refusals_forgotten: u64,
    /// Person-keyed room refusals; other task callers have their own owner facts.
    pub person_room_refusals: List<people::CapacityRefusal>,
    pub person_room_refusals_forgotten: u64,
    /// Permanent refusal answers follow this retention; unsaved entrance history is live-only.
    pub person_answer_retention: Duration,
    pub task_room_refusals: List<tasks::CapacityRefusal>,
    pub task_room_refusals_forgotten: u64,
    pub connectors: List<u16>,
    pub connectors_cut: bool,
    pub cut: bool,
}
/// Recent refused hello facts, without protocol rights or advertised workstream payloads.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct HelloRefusalLine {
    pub channel: u64,
    pub at: Wall,
    pub slots: u32,
    pub stop_bound: Duration,
    pub why: fleet::HelloRefusal,
}

fn line_bound(limits: &Limits) -> usize {
    usize::try_from(limits.documents.lines).expect("line bound fits")
}
fn object(core: &Core, task: u64, kind: people::EntryKind) -> InboxObject {
    match kind {
        people::EntryKind::Question { message } => InboxObject::Question { message },
        people::EntryKind::Proposal { number } => {
            let proposer = match core.tasks.person_proposals().get(&number) {
                Some(row) if row.goal.number == task => Proposer::Party(row.proposer),
                Some(_) | None => Proposer::Task(task),
            };
            InboxObject::Proposal { proposer, number }
        }
        people::EntryKind::Escalation { revision } => InboxObject::Escalation { revision },
        people::EntryKind::PersonTask => InboxObject::PersonTask,
        people::EntryKind::Reply { message } | people::EntryKind::ChatReply { message } => {
            InboxObject::Reply { message }
        }
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
fn reply_activity(core: &Core, task: u64, object: InboxObject) -> Option<u64> {
    match object {
        InboxObject::Reply { message } => {
            let row = core.tasks.task(task)?;
            if core.chat_requester(row).is_some() {
                for reply in &row.replies {
                    if reply.number == message {
                        return Some(reply.activity);
                    }
                }
            }
            let notice = row.person_wait?.notice?;
            if notice.message == message { Some(notice.activity) } else { None }
        }
        InboxObject::Question { .. }
        | InboxObject::Proposal { .. }
        | InboxObject::Escalation { .. }
        | InboxObject::PersonTask
        | InboxObject::Result { .. } => None,
    }
}

fn archived_conversation(core: &Core, limits: &Limits, party: u64, task: u64) -> Option<ConversationLine> {
    let archive = core.archived_chats.get(&task)?;
    if archive.party != party || core.people.role(party, archive.project).is_none() {
        return None;
    }
    let mut line = archive.line.as_ref()?.clone();
    line.seen = core.people.chat_seen(party, task)?;
    let unread = line.last_reply.saturating_sub(line.seen);
    let capped = u32::try_from(unread.min(u64::from(limits.people.inbox_entries))).expect("bounded unread");
    line.replies_unread = Some(if unread > u64::from(limits.people.inbox_entries) {
        Count::AtLeast(capped)
    } else {
        Count::Known(capped)
    });
    Some(line)
}

fn inbox_reply(
    core: &Core,
    limits: &Limits,
    party: u64,
    task: u64,
    chat: bool,
) -> Result<Option<Box<ConversationLine>>, Missing> {
    if !chat {
        return Ok(None);
    }
    match core.tasks.task(task) {
        Some(row) => Ok(Some(Box::new(conversation_from_row(core, limits, party, row)?))),
        None => match archived_conversation(core, limits, party, task) {
            Some(line) => Ok(Some(Box::new(line))),
            None => Ok(None),
        },
    }
}

#[expect(clippy::too_many_lines, reason = "one bounded projection preserves both groups and their parallel contexts")]
pub(super) fn inbox(core: &Core, limits: &Limits, party: u64, window: &Window) -> Result<Inbox, Missing> {
    super::inbox_selection_worst_case(window.rows, limits.tasks.depth).ok_or(Missing::TooLarge)?;
    let entries = core.people.inbox_needs(&limits.people, party).ok_or(Missing::Unknown)?;
    let recent = core.people.cached_updates(&limits.people, party).ok_or(Missing::Unknown)?;
    let mut ordered = Map::with_capacity(limits.people.inbox_entries.checked_mul(2).ok_or(Missing::TooLarge)?);
    let cached_needs = core.people.needs(party).ok_or(Missing::Unknown)?;
    let counts = core.people.counts(&limits.people, party).ok_or(Missing::Unknown)?;
    for entry in &entries {
        if core.people.role(party, entry.project).is_none() {
            continue;
        }
        let value = object(core, entry.task, entry.kind);
        let cursor = Cursor::Inbox { task: entry.task, object: value, at: Some(entry.at) };
        ordered
            .insert(key(cursor)?, (entry.task, entry.project, value, None, entry.at, false, cursor, false))
            .expect("bounded need prefix");
    }
    for update in &recent {
        let (task, project, value, activity, at, chat) = match *update {
            people::Update::Result(row) => {
                (row.task, row.project, InboxObject::Result { position: row.position }, None, row.at, false)
            }
            people::Update::Reply(row) => {
                let value = object(core, row.task, row.kind);
                (row.task, row.project, value, reply_activity(core, row.task, value), row.at, false)
            }
            people::Update::ChatReply(row) => (
                row.entry.task,
                row.entry.project,
                object(core, row.entry.task, row.entry.kind),
                Some(row.activity),
                row.entry.at,
                true,
            ),
        };
        if core.people.role(party, project).is_none() {
            continue;
        }
        let cursor = Cursor::Inbox { task, object: value, at: Some(at) };
        ordered
            .insert(key(cursor)?, (task, project, value, activity, at, true, cursor, chat))
            .expect("bounded shared update prefix");
    }
    let after = match window.after {
        Some(cursor) => Some(key(cursor)?),
        None => None,
    };
    let mut needs = List::with_capacity(window.rows);
    let mut changes = List::with_capacity(window.rows);
    let mut selection = List::with_capacity(window.rows);
    let mut used = 0_u32;
    let mut last = None;
    let mut more = false;
    for (position, (task, project, object, activity, at, unread, cursor, chat)) in &ordered {
        if let Some(after) = after
            && *position <= after
        {
            continue;
        }
        if used == window.rows {
            more = true;
            break;
        }
        let reply = inbox_reply(core, limits, party, *task, *chat)?;
        if *chat && reply.is_none() {
            more = true;
            break;
        }
        let link = match core.archived_chats.get(task) {
            Some(archive) if archive.party == party => {
                TaskLink::Retained { chip: archive.chip.clone(), at: archive.at }
            }
            Some(_) | None => super::task_tree::link(core, limits, party, *task)?,
        };
        let entry = InboxEntry {
            reply,
            task: link,
            project: Some(*project),
            object: *object,
            activity: *activity,
            at: Some(*at),
            unread: Some(*unread),
            cursor: *cursor,
        };
        let context = super::inbox_selection::live(core, limits, *task, *cursor)?;
        selection.push(context).expect("one context per bounded inbox row");
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
        cached_updates: if counts.more_unread { Count::AtLeast(counts.unread) } else { Count::Known(counts.unread) },
        results_seen: core.people.results_seen(party).ok_or(Missing::Unknown)?,
        updates_time_complete: true,
        selection: Some(selection),
    })
}
/// One authentic chat header, used by both the list and reply inbox entries.
pub fn conversation_from_row(
    core: &Core,
    limits: &Limits,
    party: u64,
    row: &tasks::TaskRecord,
) -> Result<ConversationLine, Missing> {
    if core.chat_requester(row) != Some(party) || core.people.role(party, row.project).is_none() {
        return Err(Missing::NotAllowed);
    }
    let seen = core.people.chat_seen(party, row.number).ok_or(Missing::Unknown)?;
    let unread = row.last_reply.saturating_sub(seen);
    let count = u32::try_from(unread.min(u64::from(limits.people.inbox_entries))).expect("capped replies");
    Ok(ConversationLine {
        chip: super::small::chip_with_core(core, limits, row)?,
        project: row.project,
        first_words: super::small::text(&row.spec.words, limits.documents.text_bytes),
        seen,
        last_reply: row.last_reply,
        replies_unread: Some(if unread > u64::from(limits.people.inbox_entries) {
            Count::AtLeast(count)
        } else {
            Count::Known(count)
        }),
        cached_replies: u32::try_from(row.replies.len()).expect("bounded reply hints"),
        delegates: u32::try_from(row.delegates.len()).expect("bounded delegates"),
        budget: super::task_tree::budget(row),
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
                tasks::Executor::Agent { charter } => charter == core.settings.chat,
                tasks::Executor::Person(_) | tasks::Executor::Procedure { .. } => false,
            };
        if !chat || row.tracked.is_some() || row.recurring.is_some() || core.people.role(party, row.project).is_none() {
            continue;
        }
        if lines.room() == 0 {
            more = true;
            break;
        }
        lines.push(conversation_from_row(core, limits, party, row)?).expect("bounded conversation rows");
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
fn host_run(
    core: &Core,
    limits: &Limits,
    clock: Option<DocumentClock>,
    value: &fleet::AttemptView,
) -> Result<HostRun, Missing> {
    let location = match value.placement {
        fleet::Placement::None => RunLocation::None,
        fleet::Placement::Connected { channel, kind } => RunLocation::Connected { channel: channel.raw(), kind },
        fleet::Placement::Adrift { channel, until } => RunLocation::Adrift {
            channel: match channel {
                Some(value) => Some(value.raw()),
                None => None,
            },
            until: wall_deadline(clock, until),
            remaining: match clock {
                Some(clock) => Some(until.saturating_since(clock.now)),
                None => None,
            },
        },
    };
    Ok(HostRun {
        task: match core.tasks.task(value.run.raw()) {
            Some(row) => TaskLink::Live(super::small::chip_with_core(core, limits, row)?),
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
    })
}
#[expect(
    clippy::too_many_lines,
    reason = "one deployment document gathers bounded child-owned host, account and capacity facts"
)]
#[expect(clippy::manual_map, reason = "step code has no closures (programming-model.md, section 10)")]
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
                fleet::Placement::Connected { channel, .. }
                | fleet::Placement::Adrift { channel: Some(channel), .. }
                    if channel == value.channel =>
                {
                    if runs.room() > 0 {
                        runs.push(host_run(core, limits, clock, run)?).expect("bounded host runs");
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
                connected: value.connected,
                grace_until: match value.grace_until {
                    Some(at) => wall_deadline(clock, at),
                    None => None,
                },
                grace_remaining: match value.grace_until {
                    Some(at) => match clock {
                        Some(value) => Some(at.saturating_since(value.now)),
                        None => None,
                    },
                    None => None,
                },
                runs,
                cut: host_cut,
            })
            .expect("bounded hosts");
    }
    cut |= usize::try_from(source_hosts.len()).expect("host count fits") > line_bound(limits);
    cut |= core.fleet.host_history_lost() != 0 || core.fleet.hello_history_lost() != 0;
    let mut hello_refusals = List::with_capacity(limits.documents.lines);
    let refused = core.fleet.hello_refusals();
    for value in refused.iter().take(line_bound(limits)) {
        hello_refusals
            .push(HelloRefusalLine {
                channel: value.channel.raw(),
                at: value.at,
                slots: value.slots,
                stop_bound: value.stop_bound,
                why: value.why,
            })
            .expect("bounded hello refusals");
    }
    cut |= usize::try_from(refused.len()).expect("hello count fits") > line_bound(limits);
    let mut attempts = List::with_capacity(limits.documents.lines);
    for value in source_runs.iter().take(line_bound(limits)) {
        attempts.push(host_run(core, limits, clock, value)?).expect("bounded attempt rows");
    }
    cut |= usize::try_from(source_runs.len()).expect("attempt count fits") > line_bound(limits);
    let rows = core.tasks.view_tasks();
    let mut accounts = List::with_capacity(limits.documents.lines);
    let source_accounts = core.accounts.account_views();
    for value in source_accounts.iter().take(line_bound(limits)) {
        let mut runs = List::with_capacity(limits.documents.lines);
        let mut waiting = List::with_capacity(limits.documents.lines);
        let mut account_cut = false;
        for row in &rows {
            let row = core.tasks.task(row.number).expect("live view task");
            let agent = match row.executor {
                tasks::Executor::Agent { charter } => {
                    core.charter(charter).expect("task charter").account() == value.account
                }
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
                    runs.push(super::small::chip_with_core(core, limits, row)?).expect("bounded account runs");
                } else {
                    account_cut = true;
                }
            }
            if needs {
                if waiting.room() > 0 {
                    waiting.push(super::small::chip_with_core(core, limits, row)?).expect("bounded waiting agents");
                } else {
                    account_cut = true;
                }
            }
        }
        accounts
            .push(Account {
                number: value.account,
                generation: value.generation,
                state: value.state,
                usable: value.usable,
                expiring: value.expiring,
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
    let refused = core.people.capacity_refusals();
    let mut person_room_refusals = List::with_capacity(limits.documents.lines);
    for value in refused.iter().take(line_bound(limits)) {
        person_room_refusals.push(*value).expect("bounded person room refusals");
    }
    cut |= usize::try_from(refused.len()).expect("refusal count fits") > line_bound(limits);
    cut |= core.people.capacity_refusals_lost() != 0;
    let refused_tasks = core.tasks.capacity_refusals();
    let mut task_room_refusals = List::with_capacity(limits.documents.lines);
    for value in refused_tasks.as_slice().iter().rev().take(line_bound(limits)) {
        task_room_refusals.push(*value).expect("bounded task room refusals");
    }
    cut |= usize::try_from(refused_tasks.len()).expect("refusal count fits") > line_bound(limits);
    cut |= core.tasks.capacity_refusals_lost() != 0;
    let mut connectors = List::with_capacity(limits.documents.lines);
    for connector in core.connectors.iter().take(line_bound(limits)) {
        connectors.push(*connector).expect("bounded configured connector references");
    }
    let connectors_cut = core.connectors.len() > line_bound(limits);
    cut |= connectors_cut;
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
        hello_refusals,
        hosts_forgotten: core.fleet.host_history_lost(),
        hello_refusals_forgotten: core.fleet.hello_history_lost(),
        person_room_refusals,
        person_room_refusals_forgotten: core.people.capacity_refusals_lost(),
        person_answer_retention: limits.people.request_retention,
        task_room_refusals,
        task_room_refusals_forgotten: core.tasks.capacity_refusals_lost(),
        connectors,
        connectors_cut,
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
        | super::Subject::Tasks { .. }
        | super::Subject::Conversations { .. }
        | super::Subject::Run { .. }
        | super::Subject::Proposal { .. }
        | super::Subject::Escalation { .. }
        | super::Subject::Question { .. }
        | super::Subject::PersonTask { .. }
        | super::Subject::Result { .. }
        | super::Subject::Project { .. }
        | super::Subject::Note { .. }
        | super::Subject::Application { .. } => super::make(core, limits, party, subject, window),
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
        .checked_add(List::<HelloRefusalLine>::worst_case(limits.lines)?)?
        .checked_add(List::<people::CapacityRefusal>::worst_case(limits.lines)?)?
        .checked_add(List::<tasks::CapacityRefusal>::worst_case(limits.lines)?)?
        .checked_add(List::<u16>::worst_case(limits.lines)?)?
        .checked_add(u64::try_from(size_of::<Deployment>()).ok()?)
}

pub(super) fn scratch_worst_case(limits: &Limits) -> Option<u64> {
    let tracked = limits
        .fleet
        .workers
        .checked_mul(limits.fleet.slots)?
        .checked_add(limits.fleet.engine_slots)?
        .checked_add(limits.fleet.attempts)?;
    List::<fleet::HostView>::worst_case(
        limits
            .fleet
            .workers
            .checked_add(u32::from(limits.fleet.engine_slots > 0))?
            .checked_add(limits.fleet.history)?,
    )?
    .checked_add(List::<fleet::HelloRefusalRecord>::worst_case(limits.fleet.history)?)?
    .checked_add(List::<people::CapacityRefusal>::worst_case(
        limits.people.requests.checked_add(limits.people.room_refusals)?,
    )?)?
    .checked_add(List::<tasks::CapacityRefusal>::worst_case(limits.tasks.room_refusals)?)?
    .checked_add(List::<fleet::AttemptView>::worst_case(tracked)?)?
    .checked_add(List::<accounts::Fact>::worst_case(limits.accounts.accounts)?)?
    .checked_add(Map::<
        (bool, u64, u64, u64, InboxObject),
        (u64, u32, InboxObject, Option<u64>, Wall, bool, Cursor, bool),
    >::worst_case(limits.people.inbox_entries.checked_mul(2)?)?)?
    .checked_add(List::<people::Entry>::worst_case(limits.people.inbox_entries.checked_mul(5)?)?)
}
