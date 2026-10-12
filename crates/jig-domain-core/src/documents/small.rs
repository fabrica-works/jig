//! Small live documents and the shared bounded chip construction.
use super::{Chip, Count, ExecutorKind, Frame, FrameProjectRole, Missing, OfferedCharter, PhaseLine, Run, Text};
use crate::{Core, Limits};
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
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
    chip_with_core(core, limits, row)
}

pub(super) fn chip_with_core(core: &Core, limits: &Limits, row: &tasks::TaskRecord) -> Result<Chip, Missing> {
    let mut chip = chip_from_row(limits, row)?;
    super::chip_phase::observe(core, limits, row, &mut chip.details)?;
    Ok(chip)
}

#[expect(clippy::manual_map, reason = "step code explicitly selects readiness without closures")]
pub(super) fn chip_from_row(limits: &Limits, row: &tasks::TaskRecord) -> Result<Chip, Missing> {
    let number = row.number;
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
            tasks::Executor::Person(address) => PhaseLine::Person {
                address,
                taken_by: row.taken_by,
                since: match row.person_wait {
                    Some(wait) => Some(wait.since),
                    None => None,
                },
            },
            tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. } => PhaseLine::Active(*active),
        },
        tasks::Phase::Closing(closing) => PhaseLine::Closing(closing.stage),
        tasks::Phase::Held { why, .. } => PhaseLine::Held { reason: *why, revision: row.revisions.hold },
        tasks::Phase::Ended(_) => PhaseLine::Ended,
    };
    Ok(Chip { number, title, executor, phase, details: Box::new(super::chip_phase::from_row(limits, row)?) })
}

#[expect(clippy::manual_map, reason = "step code explicitly matches optional owner ledgers without closures")]
pub(super) fn frame(core: &Core, limits: &Limits, party: u64) -> Result<Frame, Missing> {
    let identity = core.people.identity(party).ok_or(Missing::Unknown)?;
    let roles = core.people.project_roles(party).ok_or(Missing::Unknown)?;
    let mut projects = List::with_capacity(limits.documents.lines);
    let mut charters = List::with_capacity(limits.documents.lines);
    for role in roles.iter().take(usize::try_from(limits.documents.lines).expect("line count fits")) {
        let pool = match core.tasks.funding(tasks::Funder::Pool {
            project: role.project,
            person: party,
            period: core.period(role.project),
        }) {
            Some(record) => Some(super::project_note::ledger(record)),
            None => None,
        };
        projects
            .push(FrameProjectRole {
                project: role.project,
                role: role.role,
                name: None,
                pool,
                chat_budget: match core.authority.role(role.project, role.role.number()) {
                    Some(_) => {
                        core.chat_authority(role.project, party, role.role, limits.tasks.tree_tasks).budget.spend
                    }
                    None => 0,
                },
            })
            .expect("bounded project list");
        let mut goals = List::with_capacity(limits.documents.lines);
        let mut cut = false;
        for charter in &core.settings.charters {
            if crate::acts::bounds::goal_charter(core, limits, party, role.project, role.role, charter.number) {
                if goals.room() > 0 {
                    goals.push(charter.number).expect("bounded charter list");
                } else {
                    cut = true;
                }
            }
        }
        charters
            .push(OfferedCharter { project: role.project, chat: core.settings.chat, goals, cut })
            .expect("one charter line per project");
    }
    let counts = core.people.counts(&limits.people, party).ok_or(Missing::Unknown)?;
    let updates = if counts.more_unread { Count::AtLeast(counts.unread) } else { Count::Known(counts.unread) };
    Ok(Frame {
        party,
        kind: core.people.kind(party).ok_or(Missing::Unknown)?,
        login: text(&identity.login, limits.documents.text_bytes),
        name: text(&identity.name, limits.documents.text_bytes),
        deployment_role: core.people.role(party, 0),
        cut: projects.len() < roles.len(),
        projects,
        needs: counts.needs,
        need_counts: core.people.need_counts(party).ok_or(Missing::Unknown)?,
        updates,
        results_seen: core.people.results_seen(party).ok_or(Missing::Unknown)?,
        charters,
        retention: limits.people.request_retention,
    })
}

pub(super) fn run(core: &Core, limits: &Limits, party: u64, number: u64) -> Result<Run, Missing> {
    visible(core, party, number)?;
    let proof = match core.proofs.get(&number) {
        Some(proof) if proof.terminal.is_none() => proof,
        Some(_) | None => return Ok(Run { task: number, attempt: None, committed_turn: 0, writing: None, call: None }),
    };
    let committed_turn = match proof.turn {
        Some(turn) => turn.turn,
        None => 0,
    };
    let writing =
        match jig_domain_views::writing(&core.views, skein_lib::Token::new(number), limits.documents.text_bytes) {
            Some(writing) if writing.turn.attempt == proof.attempt && writing.turn.turn > committed_turn => {
                Some(writing)
            }
            Some(_) | None => None,
        };
    let call = jig_domain_views::current_call(
        &core.views,
        skein_lib::Token::new(number),
        proof.attempt,
        limits.documents.text_bytes,
    );
    Ok(Run { task: number, attempt: Some(proof.attempt), committed_turn, writing, call })
}
