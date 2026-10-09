//! Small live documents and the shared bounded chip construction.
use super::{Chip, Count, ExecutorKind, Frame, Missing, OfferedCharter, PhaseLine, Run, Text};
use crate::{Core, Limits};
use jig_core_people as people;
use jig_core_tasks as tasks;
use skein_lib::List;
use skein_lib::bytes::copy_of;

pub(super) fn text(bytes: &[u8], maximum: u32) -> Text {
    let length = bytes.len().min(usize::try_from(maximum).expect("text bound fits usize"));
    Text { bytes: copy_of(bytes.get(..length).expect("bounded prefix")), cut: length < bytes.len() }
}

pub(super) fn visible(core: &Core, party: u64, number: u64) -> Result<&tasks::TaskRecord, Missing> {
    let row = core.tasks.task(number).ok_or(Missing::Unknown)?;
    if core.people.role(party, row.project).is_none() {
        return Err(Missing::NotAllowed);
    }
    match row.phase {
        tasks::Phase::Ended(_) => Err(Missing::Ended),
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
            Ok(row)
        }
    }
}

pub(super) fn chip(core: &Core, limits: &Limits, party: u64, number: u64) -> Result<Chip, Missing> {
    let row = visible(core, party, number)?;
    let first = skein_lib::bytes::find(&row.spec.words, b"\n").unwrap_or(row.spec.words.len());
    let title = text(row.spec.words.get(..first).expect("first line prefix"), limits.documents.text_bytes);
    let executor = match row.executor {
        tasks::Executor::Agent { .. } => ExecutorKind::Agent,
        tasks::Executor::Procedure { .. } => ExecutorKind::Procedure,
        tasks::Executor::Person(_) => ExecutorKind::Person,
    };
    let phase = match &row.phase {
        tasks::Phase::Waiting => {
            let mut dependencies = List::with_capacity(limits.documents.lines);
            for number in row.waiting_on.iter().take(usize::try_from(limits.documents.lines).expect("line count fits"))
            {
                dependencies.push(*number).expect("bounded dependency lines");
            }
            PhaseLine::Waiting {
                more: usize::try_from(dependencies.len()).expect("line count fits") < row.waiting_on.len(),
                dependencies,
                holds: !row.holds_taken && !row.holdings.is_empty(),
            }
        }
        tasks::Phase::Active(active) => match row.executor {
            tasks::Executor::Person(address) => {
                PhaseLine::Person { address, taken_by: row.taken_by, since: row.created_at }
            }
            tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. } => PhaseLine::Active(*active),
        },
        tasks::Phase::Closing(closing) => PhaseLine::Closing(closing.stage),
        tasks::Phase::Held { why, .. } => PhaseLine::Held { reason: *why, revision: row.revisions.hold },
        tasks::Phase::Ended(_) => PhaseLine::Ended,
    };
    Ok(Chip { number, title, executor, phase })
}

pub(super) fn frame(core: &Core, limits: &Limits, party: u64) -> Result<Frame, Missing> {
    let identity = core.people.identity(party).ok_or(Missing::Unknown)?;
    let roles = core.people.project_roles(party).ok_or(Missing::Unknown)?;
    let mut projects = List::with_capacity(limits.documents.lines);
    let mut charters = List::with_capacity(limits.documents.lines);
    for role in roles.iter().take(usize::try_from(limits.documents.lines).expect("line count fits")) {
        projects.push(*role).expect("bounded project list");
        let mut goals = List::with_capacity(limits.documents.lines);
        for charter in
            core.tasks.charters().iter().take(usize::try_from(limits.documents.lines).expect("line count fits"))
        {
            goals.push(*charter).expect("bounded charter list");
        }
        let cut = usize::try_from(goals.len()).expect("charter count fits") < core.tasks.charters().len();
        charters
            .push(OfferedCharter { project: role.project, chat: core.settings.charter, goals, cut })
            .expect("one charter line per project");
    }
    let waiting = core.people.cached_waiting(&limits.people, party).ok_or(Missing::Unknown)?;
    let mut needs = 0_u32;
    let mut updates =
        u32::try_from(core.people.cached_results(party).ok_or(Missing::Unknown)?.len()).expect("cache count fits");
    for entry in &waiting {
        match entry.kind {
            people::EntryKind::Reply { .. } => updates = updates.saturating_add(1),
            people::EntryKind::Proposal { .. }
            | people::EntryKind::Escalation { .. }
            | people::EntryKind::Question { .. }
            | people::EntryKind::PersonTask => needs = needs.saturating_add(1),
        }
    }
    let updates = if updates >= limits.people.inbox_entries { Count::AtLeast(updates) } else { Count::Known(updates) };
    Ok(Frame {
        party,
        kind: core.people.kind(party).ok_or(Missing::Unknown)?,
        login: text(&identity.login, limits.documents.text_bytes),
        name: text(&identity.name, limits.documents.text_bytes),
        cut: projects.len() < roles.len(),
        projects,
        needs,
        updates,
        charters,
        retention: limits.people.request_retention,
        versions: limits.documents.versions,
    })
}

pub(super) fn run(core: &Core, party: u64, number: u64) -> Result<Run, Missing> {
    visible(core, party, number)?;
    let proof = core.proofs.get(&number).ok_or(Missing::Ended)?;
    if proof.terminal.is_some() {
        return Err(Missing::Ended);
    }
    let committed_turn = match proof.turn {
        Some(turn) => turn.turn,
        None => 0,
    };
    Ok(Run { task: number, attempt: proof.attempt, committed_turn, writing: None })
}
