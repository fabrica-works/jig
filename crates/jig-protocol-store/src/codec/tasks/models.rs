//! Effective retained authorities follow configured names; original requests belong to people.
use super::{domain as d, wire as w};
use alloc::boxed::Box;
use skein_lib::List;

pub(super) fn encode(names: &[Box<[u8]>], limits: &w::Limits) -> Result<List<w::ModelName>, w::Problem> {
    let mut result = List::with_capacity(u32::try_from(names.len()).expect("configured models"));
    for name in names {
        result.push(w::ModelName::new(limits, w::ModelNameParts { value: name.clone() })?).expect("same count");
    }
    Ok(result)
}
pub(super) fn moves(kept: &List<w::ModelName>, names: &[Box<[u8]>]) -> [u64; 64] {
    let mut moved = [0; 64];
    for (old, name) in kept.iter().enumerate() {
        for (at, today) in names.iter().enumerate() {
            if name.value() == today.as_ref() {
                *moved.get_mut(old).expect("bounded names") =
                    1_u64.checked_shl(u32::try_from(at).expect("bounded model index")).unwrap_or(0);
                break;
            }
        }
    }
    moved
}
fn authority(value: &mut d::Authority, moved: &[u64; 64]) {
    if !value.models.is_all() {
        let mut bits = 0;
        for (at, today) in moved.iter().enumerate() {
            if value.models.bits().checked_shr(u32::try_from(at).expect("bounded bit")).unwrap_or(0) & 1 == 1 {
                bits |= today;
            }
        }
        value.models = d::Models::Named(bits);
    }
}
fn members(batch: &mut [d::New], moved: &[u64; 64]) {
    for member in batch {
        authority(&mut member.authority, moved);
        if let Some(recurring) = &mut member.recurring {
            for nested in &mut recurring.batch {
                authority(&mut nested.authority, moved);
            }
        }
    }
}
fn amendment(value: &mut d::Amendment, moved: &[u64; 64]) {
    if let Some(value) = &mut value.authority {
        authority(value, moved);
    }
}
fn proposal(value: &mut d::Proposal, moved: &[u64; 64]) {
    match &mut value.action {
        d::ProposalAction::Batch(batch) => members(batch, moved),
        d::ProposalAction::Amend { amendment: value, .. } => amendment(value, moved),
        d::ProposalAction::Widen { authority: value, .. }
        | d::ProposalAction::Effect { authority: value, .. }
        | d::ProposalAction::ProcedureEffect { authority: value, .. } => authority(value, moved),
        d::ProposalAction::Release { .. } => {}
    }
}
/// Every authority-bearing durable tasks variant, including pending policy-held proposals.
pub(super) fn remap(row: &mut d::Stored, moved: &[u64; 64]) {
    match row {
        d::Stored::Live(row) | d::Stored::Ended(row) => {
            authority(&mut row.authority, moved);
            if let Some(value) = &mut row.proposal {
                proposal(value, moved);
            }
            if let Some(value) = &mut row.recurring {
                members(&mut value.template.batch, moved);
            }
        }
        d::Stored::PersonProposal(row) | d::Stored::DecidedProposal(row) => match &mut row.action {
            d::PersonAction::Batch(batch) => members(batch, moved),
            d::PersonAction::Amend { amendment: value, .. } => amendment(value, moved),
        },
        d::Stored::History(row) => {
            if let Some(value) = &mut row.proposal {
                proposal(value, moved);
            }
            if let Some(value) = &mut row.amendment {
                authority(&mut value.before.authority, moved);
                authority(&mut value.after.authority, moved);
            }
        }
        d::Stored::Update(_)
        | d::Stored::Result(_)
        | d::Stored::EndedByParty(_)
        | d::Stored::EndedByProject(_)
        | d::Stored::EndedDelegate(_)
        | d::Stored::Delegate(_)
        | d::Stored::Activity(_)
        | d::Stored::Milestone(_)
        | d::Stored::ClosedLedger(_)
        | d::Stored::Stub(_)
        | d::Stored::Ledger(_)
        | d::Stored::Writer(_)
        | d::Stored::Pool(_) => {}
    }
}
