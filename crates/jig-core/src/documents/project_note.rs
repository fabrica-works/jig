//! Project policy and notes (`client/domain/pages.md`, sections 6.1, 6.4, 6.5).
//! Notes' bodies are loaded on demand by their child; this module builds a
//! document from that loaded entry without adding a mutable cache.
//! Pool edits target the configured period independently of the observed
//! period. Permission mappings come from the core's retained policy entry;
//! an absent entry remains absent, and each new list states its own loss.
use super::{AuthorityLine, Chip, GrantLine, Missing, NoteScope, Part, PartKind, PatternEnd, ProposedTask, Text};
use crate::{Core, Limits};
use alloc::boxed::Box;
use jig_core_authority as authority;
use jig_core_notes as notes;
use jig_core_people as people;
use jig_core_tasks as tasks;
use skein_lib::{List, Map, Wall};

/// The role of a resource adopted into a project.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ResourceRole {
    Owned,
    Participant,
    Unavailable,
    Context,
}
/// One adopted connector resource and its project role.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProjectResource {
    pub part: Part,
    pub role: ResourceRole,
}
/// One policy role's permission and allotment ceilings.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PolicyRole {
    pub number: u32,
    pub authority: AuthorityLine,
    pub period_spend: u64,
    pub requests: authority::Requests,
    pub decides: authority::Proposals,
}
/// An effect requirement as policy records it, with an exact resource pattern.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Requirement {
    pub grant: GrantLine,
    pub judge: authority::Judge,
    pub guard: authority::Guard,
    pub must_be_guarded: bool,
}
/// Project policy in force, with truncated lists stated.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Policy {
    pub ceiling: AuthorityLine,
    pub period_spend: u64,
    pub escalation_role: Option<u32>,
    pub projections: List<GrantLine>,
    pub roles: List<PolicyRole>,
    pub requirements: List<Requirement>,
    pub permissions: Option<List<people::PermissionRole>>,
    pub permissions_cut: bool,
    pub cut: bool,
}
/// A retained party's membership; services carry the same role facts.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Member {
    pub party: u64,
    pub kind: people::Kind,
    pub role: people::Role,
    pub login: Text,
    pub name: Text,
}
/// Authentic project period or party pool accounting, including superseded rows.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Ledger {
    pub source: tasks::Funder,
    pub parent: Option<tasks::Funder>,
    pub numbers: tasks::Numbers,
    pub left: u64,
    pub closed: bool,
    pub began: Option<Wall>,
    pub next: Option<Wall>,
}
/// A live root's expense against its authentic allotment, largest first.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SpendLine {
    pub chip: Chip,
    pub funder: tasks::Funder,
    pub numbers: tasks::Numbers,
}
/// A recurring template with authentic retained period and supplied boundary facts.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Recurring {
    pub chip: Chip,
    pub key: u32,
    pub batch: List<ProposedTask>,
    pub budget: u64,
    pub overlap: tasks::RecurringOverlap,
    pub current_period: u64,
    pub began: Option<Wall>,
    pub next: Option<Wall>,
    pub last_period: u64,
    pub pending_period: Option<u64>,
    pub last_batch: List<u64>,
    pub cut: bool,
}
/// One project's adopted resources, current policy, membership and spend.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Project {
    pub acts: List<crate::acts::Offered>,
    pub number: u32,
    pub resources: List<ProjectResource>,
    pub policy: Policy,
    pub members: List<Member>,
    pub current_period: Option<Ledger>,
    pub pools: List<Ledger>,
    pub configured_period: u64,
    pub configured_pools: List<Ledger>,
    pub configured_pools_cut: bool,
    pub spend: List<SpendLine>,
    pub recurring: List<Recurring>,
    pub cut: bool,
}
/// A note scope keeps every connector key byte; oversized keys are refused.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum NoteLocation {
    Deployment,
    Project(u32),
    Goal { project: u32, task: u64 },
    Resources { project: u32, scope: NoteScope },
}
/// A note made from its child's store-loaded entry. The child currently retains
/// no changed wall time; its revision is carried unchanged.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Note {
    pub acts: List<crate::acts::Offered>,
    pub number: u64,
    pub scope: NoteLocation,
    pub description: Text,
    pub body: Text,
    pub writer: notes::Author,
    pub tasks: List<u64>,
    pub changed: Option<Wall>,
    pub revision: u32,
    pub cut: bool,
}

fn bound(limits: &Limits) -> usize {
    usize::try_from(limits.documents.lines).expect("line count fits")
}
fn path(limits: &Limits, values: &[Box<[u8]>]) -> Result<List<Box<[u8]>>, Missing> {
    let value = super::task_tree::part_path(limits, 0, PartKind::Resource, values)?;
    match value.key {
        super::PartKey::Path(path) => Ok(path),
        super::PartKey::Number(_) => unreachable!("path constructor"),
    }
}
fn last(limits: &Limits, value: &authority::Last) -> Result<PatternEnd, Missing> {
    let bytes = match value {
        authority::Last::Exact(bytes) | authority::Last::Open(bytes) => bytes,
    };
    if bytes.len() > usize::try_from(limits.documents.text_bytes).expect("text count fits") {
        return Err(Missing::TooLarge);
    }
    Ok(match value {
        authority::Last::Exact(bytes) => PatternEnd::Exact(skein_lib::bytes::copy_of(bytes)),
        authority::Last::Open(bytes) => PatternEnd::Open(skein_lib::bytes::copy_of(bytes)),
    })
}
fn grant(limits: &Limits, value: &authority::Grant) -> Result<GrantLine, Missing> {
    Ok(GrantLine {
        connector: value.connector,
        kind: value.kind,
        segments: path(limits, &value.pattern.segments)?,
        last: last(limits, &value.pattern.last)?,
    })
}
fn authority_line(limits: &Limits, value: &authority::Authority) -> Result<AuthorityLine, Missing> {
    let mut grants = List::with_capacity(limits.documents.lines);
    for value in value.grants.iter().take(bound(limits)) {
        grants.push(grant(limits, value)?).expect("bounded grants");
    }
    let mut executors = List::with_capacity(limits.documents.lines);
    for value in value.delegation.kinds.iter().take(bound(limits)) {
        executors
            .push(match value {
                authority::Executor::Charter(number) => tasks::AuthorityExecutor::Charter(*number),
                authority::Executor::Procedure(number) => tasks::AuthorityExecutor::Procedure(*number),
                authority::Executor::Role(number) => tasks::AuthorityExecutor::Role(*number),
            })
            .expect("bounded executors");
    }
    let mut note_resources = List::with_capacity(limits.documents.lines);
    for value in value.note_resources.iter().take(bound(limits)) {
        note_resources
            .push(NoteScope {
                connector: value.connector,
                segments: path(limits, &value.pattern.segments)?,
                last: last(limits, &value.pattern.last)?,
            })
            .expect("bounded note scopes");
    }
    Ok(AuthorityLine {
        tools: tasks::Tools(value.tools.0),
        grants,
        executors,
        tasks: value.delegation.tasks,
        depth: value.delegation.depth,
        ceiling: tasks::Budget { spend: value.budget.spend, deadline: value.budget.deadline },
        notes: tasks::Scopes(value.notes.0),
        note_resources,
        cut: value.grants.len() > bound(limits)
            || value.delegation.kinds.len() > bound(limits)
            || value.note_resources.len() > bound(limits),
    })
}
fn policy(
    limits: &Limits,
    value: &authority::Policy,
    mappings: Option<&[people::PermissionRole]>,
) -> Result<Policy, Missing> {
    let mut roles = List::with_capacity(limits.documents.lines);
    for value in value.roles.iter().take(bound(limits)) {
        roles
            .push(PolicyRole {
                number: value.number,
                authority: authority_line(limits, &value.authority)?,
                period_spend: value.period_spend,
                requests: value.requests,
                decides: value.decides,
            })
            .expect("bounded roles");
    }
    let mut requirements = List::with_capacity(limits.documents.lines);
    for value in value.requirements.iter().take(bound(limits)) {
        requirements
            .push(Requirement {
                grant: GrantLine {
                    connector: value.connector,
                    kind: value.kind,
                    segments: path(limits, &value.pattern.segments)?,
                    last: last(limits, &value.pattern.last)?,
                },
                judge: value.judge,
                guard: value.guard,
                must_be_guarded: value.must_be_guarded,
            })
            .expect("bounded requirements");
    }
    let mut projections = List::with_capacity(limits.documents.lines);
    for value in value.projections.iter().take(bound(limits)) {
        projections.push(grant(limits, value)?).expect("bounded projection grants");
    }
    let mut permissions_cut = false;
    let permissions = match mappings {
        Some(mappings) => {
            let mut permissions = List::with_capacity(limits.documents.lines);
            for mapping in mappings.iter().take(bound(limits)) {
                permissions.push(*mapping).expect("bounded permission mappings");
            }
            permissions_cut = mappings.len() > bound(limits);
            Some(permissions)
        }
        None => None,
    };
    Ok(Policy {
        ceiling: authority_line(limits, &value.ceiling)?,
        period_spend: value.period_spend,
        escalation_role: value.escalation_role,
        projections,
        roles,
        requirements,
        permissions,
        permissions_cut,
        cut: value.roles.len() > bound(limits)
            || value.requirements.len() > bound(limits)
            || value.projections.len() > bound(limits)
            || permissions_cut,
    })
}
#[expect(clippy::manual_map, reason = "step code matches options without closures")]
pub(super) fn ledger(value: &tasks::FundingRecord) -> Ledger {
    Ledger {
        source: value.funder,
        parent: value.parent,
        numbers: value.numbers,
        left: super::objects::left(value.numbers),
        closed: value.closed,
        began: match value.period {
            Some(times) => Some(times.began),
            None => None,
        },
        next: match value.period {
            Some(times) => times.next,
            None => None,
        },
    }
}
#[expect(
    clippy::too_many_lines,
    reason = "one project document gathers child-owned bounded policy, role and spending facts"
)]
pub(super) fn project(core: &Core, limits: &Limits, party: u64, number: u32) -> Result<Project, Missing> {
    if core.people.role(party, number).is_none() {
        return Err(Missing::NotAllowed);
    }
    let value = core.authority.policy(number).ok_or(Missing::Unknown)?;
    let holdings = core.people.project_members(number).ok_or(Missing::Unknown)?;
    let mut resources = List::with_capacity(limits.documents.lines);
    let mut cut = core.resource_roles_full;
    for (key, role) in &core.resource_roles {
        if key.project == number {
            if resources.room() == 0 {
                cut = true;
                continue;
            }
            resources
                .push(ProjectResource {
                    part: super::task_tree::part_path(limits, key.name.connector, PartKind::Resource, &key.name.path)?,
                    role: match role {
                        crate::connector::ResourceRole::Owned => ResourceRole::Owned,
                        crate::connector::ResourceRole::Participant => ResourceRole::Participant,
                        crate::connector::ResourceRole::Context => ResourceRole::Context,
                        crate::connector::ResourceRole::Unavailable => ResourceRole::Unavailable,
                    },
                })
                .expect("bounded resource roles");
        }
    }
    let mut members = List::with_capacity(limits.documents.lines);
    for holding in holdings.iter().take(bound(limits)) {
        let identity = core.people.identity(holding.person).ok_or(Missing::Unknown)?;
        members
            .push(Member {
                party: holding.person,
                kind: core.people.kind(holding.person).ok_or(Missing::Unknown)?,
                role: holding.role,
                login: super::small::text(&identity.login, limits.documents.text_bytes),
                name: super::small::text(&identity.name, limits.documents.text_bytes),
            })
            .expect("bounded memberships");
    }
    cut |= holdings.len() > bound(limits);
    let mut current_period = None;
    let mut current_number = 0;
    for (source, value) in core.tasks.funding_records() {
        match source {
            tasks::Funder::Period { project, period }
                if *project == number && *period > current_number && !value.closed =>
            {
                current_number = *period;
                current_period = Some(ledger(value));
            }
            tasks::Funder::Task(_)
            | tasks::Funder::Pool { .. }
            | tasks::Funder::Period { .. }
            | tasks::Funder::Recurring { .. } => {}
        }
    }
    let mut pools = List::with_capacity(limits.documents.lines);
    for (source, value) in core.tasks.funding_records() {
        match source {
            tasks::Funder::Pool { project, period, .. } if *project == number && *period == current_number => {
                if pools.room() > 0 {
                    pools.push(ledger(value)).expect("bounded pools");
                } else {
                    cut = true;
                }
            }
            tasks::Funder::Task(_)
            | tasks::Funder::Pool { .. }
            | tasks::Funder::Period { .. }
            | tasks::Funder::Recurring { .. } => {}
        }
    }
    let mut configured_pools = List::with_capacity(limits.documents.lines);
    let mut configured_pools_cut = false;
    for (source, value) in core.tasks.funding_records() {
        match source {
            tasks::Funder::Pool { project, period, .. } if *project == number && *period == core.settings.period => {
                if configured_pools.room() > 0 {
                    configured_pools.push(ledger(value)).expect("bounded configured pools");
                } else {
                    configured_pools_cut = true;
                }
            }
            tasks::Funder::Task(_)
            | tasks::Funder::Pool { .. }
            | tasks::Funder::Period { .. }
            | tasks::Funder::Recurring { .. } => {}
        }
    }
    cut |= configured_pools_cut;
    let rows = core.tasks.view_tasks();
    let mut ordered = Map::with_capacity(limits.tasks.tasks);
    let mut recurring = List::with_capacity(limits.documents.lines);
    for row in &rows {
        if row.project != number {
            continue;
        }
        let row = core.tasks.task(row.number).expect("live view row");
        if match row.requester {
            tasks::Party::Task(_) => false,
            tasks::Party::Person(_) | tasks::Party::Deployment { .. } => true,
        } {
            let spent = row.numbers.spent.checked_add(row.numbers.spent_below).expect("bounded settled spend");
            ordered
                .insert((u64::MAX.checked_sub(spent).expect("spent within u64"), row.number), row.number)
                .expect("one ordered root per live task");
        }
        if let Some(state) = &row.recurring {
            if recurring.room() == 0 {
                cut = true;
                continue;
            }
            let mut batch = List::with_capacity(limits.documents.lines);
            for member in state.template.batch.iter().take(bound(limits)) {
                batch.push(super::objects::proposed(limits, member)?).expect("bounded template");
            }
            if state.last_batch.len() > bound(limits) {
                return Err(Missing::TooLarge);
            }
            let mut last_batch = List::with_capacity(limits.documents.lines);
            for task in &state.last_batch {
                last_batch.push(*task).expect("bounded last batch");
            }
            recurring
                .push(Recurring {
                    chip: super::small::chip(core, limits, party, row.number)?,
                    key: state.template.key,
                    batch,
                    budget: row.authority.budget.spend,
                    overlap: state.template.overlap,
                    current_period: current_number,
                    began: match current_period {
                        Some(value) => value.began,
                        None => None,
                    },
                    next: match current_period {
                        Some(value) => value.next,
                        None => None,
                    },
                    last_period: state.last_period,
                    pending_period: state.pending_period,
                    last_batch,
                    cut: state.template.batch.len() > bound(limits),
                })
                .expect("bounded recurring tasks");
        }
    }
    let mut spend = List::with_capacity(limits.documents.lines);
    for (_, task) in &ordered {
        if spend.room() == 0 {
            cut = true;
            break;
        }
        let row = core.tasks.task(*task).expect("ordered live task");
        spend
            .push(SpendLine {
                chip: super::small::chip(core, limits, party, *task)?,
                funder: row.funder,
                numbers: row.numbers,
            })
            .expect("bounded spending rows");
    }
    Ok(Project {
        acts: super::offers::subject(core, limits, party, &super::Subject::Project { project: number })?,
        number,
        resources,
        policy: policy(
            limits,
            value,
            match core.permission_roles.get(&number) {
                Some(mappings) => Some(mappings.as_ref()),
                None => None,
            },
        )?,
        members,
        current_period,
        pools,
        configured_period: core.settings.period,
        configured_pools,
        configured_pools_cut,
        spend,
        recurring,
        cut,
    })
}
/// Make a note from the owning child's loaded record, checking scope visibility.
pub fn note_from_entry(core: &Core, limits: &Limits, party: u64, value: &notes::Entry) -> Result<Note, Missing> {
    let project = match value.scope {
        notes::Scope::Deployment => 0,
        notes::Scope::Project { project }
        | notes::Scope::Goal { project, .. }
        | notes::Scope::Resources { project, .. } => project,
    };
    if core.people.identity(party).is_none() {
        return Err(Missing::Unknown);
    }
    if core.people.role(party, project).is_none() {
        return Err(Missing::NotAllowed);
    }
    let scope = match &value.scope {
        notes::Scope::Deployment => NoteLocation::Deployment,
        notes::Scope::Project { project } => NoteLocation::Project(*project),
        notes::Scope::Goal { project, goal } => NoteLocation::Goal { project: *project, task: *goal },
        notes::Scope::Resources { project, connector, pattern } => {
            let bytes = match &pattern.last {
                notes::Last::Exact(bytes) | notes::Last::Open(bytes) => bytes,
            };
            if bytes.len() > usize::try_from(limits.documents.text_bytes).expect("text count fits") {
                return Err(Missing::TooLarge);
            }
            NoteLocation::Resources {
                project: *project,
                scope: NoteScope {
                    connector: *connector,
                    segments: path(limits, &pattern.segments)?,
                    last: match &pattern.last {
                        notes::Last::Exact(bytes) => PatternEnd::Exact(skein_lib::bytes::copy_of(bytes)),
                        notes::Last::Open(bytes) => PatternEnd::Open(skein_lib::bytes::copy_of(bytes)),
                    },
                },
            }
        }
    };
    let mut tasks = List::with_capacity(limits.documents.lines);
    for task in value.references.iter().take(bound(limits)) {
        tasks.push(*task).expect("bounded referenced tasks");
    }
    let edit_scope = match &value.scope {
        notes::Scope::Deployment => people::NoteScope::Deployment,
        notes::Scope::Project { .. } => people::NoteScope::Project,
        notes::Scope::Goal { goal, .. } => people::NoteScope::Goal { goal: *goal },
        notes::Scope::Resources { connector, pattern, .. } => people::NoteScope::Resources {
            connector: *connector,
            pattern: people::Pattern {
                segments: pattern.segments.clone(),
                last: match &pattern.last {
                    notes::Last::Exact(last) => people::Last::Exact(last.clone()),
                    notes::Last::Open(last) => people::Last::Open(last.clone()),
                },
            },
        },
    };
    Ok(Note {
        acts: super::offers::note(core, party, project, &edit_scope),
        number: value.name,
        scope,
        description: super::small::text(&value.description, limits.documents.text_bytes),
        body: super::small::text(&value.body, limits.documents.text_bytes),
        writer: value.author,
        tasks,
        changed: Some(value.changed),
        revision: value.revision,
        cut: usize::try_from(value.references.len()).expect("references fit") > bound(limits),
    })
}

pub(super) fn worst_case(limits: &super::DocumentLimits) -> Option<u64> {
    // Nested project carriers have no task steering offers of their own.
    let count = u64::from(limits.lines);
    let carriers = count.checked_mul(count)?.checked_add(count.checked_mul(8)?)?.checked_add(8)?;
    super::task_tree::overview_worst_case(limits)?
        .checked_mul(carriers)?
        .checked_add(super::offers::heap_worst_case(limits)?)?
        .checked_add(List::<people::PermissionRole>::worst_case(limits.lines)?)?
        .checked_add(List::<Ledger>::worst_case(limits.lines)?)?
        .checked_add(u64::try_from(size_of::<Project>()).ok()?)
}
