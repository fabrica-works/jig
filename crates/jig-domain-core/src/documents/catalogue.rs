//! Public offers and payer readiness (client/domain/contract.md, 5.1 and 7.1).
use super::{DocumentLimits, Missing, Text};
use crate::{Core, Limits};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_fleet as fleet;
use jig_domain_tasks as tasks;
use skein_lib::List;

/// Public configuration, independent of private account identities.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[expect(clippy::struct_excessive_bools, reason = "independent configured source offers and cut facts")]
pub struct ProviderOffer {
    pub provider: u32,
    pub name: Text,
    pub sign_in: bool,
    pub key: bool,
    pub programs: List<Box<[u8]>>,
    pub programs_cut: bool,
    pub none: bool,
    pub signed_in_plan: bool,
    pub manage_usage: Option<Box<[u8]>>,
}
/// Availability is an observation, never a run reservation.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Readiness {
    Usable,
    Unavailable,
    Unknown,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PayerReadiness {
    pub provider: u32,
    pub payer: accounts::Use,
    pub readiness: Readiness,
    /// None until program check facts can establish settlement.
    pub settled: Option<bool>,
}

pub(super) fn offers(core: &Core, limits: &DocumentLimits) -> Result<(List<ProviderOffer>, bool), Missing> {
    let mut result = List::with_capacity(limits.lines);
    let mut cut = false;
    for provider in &core.settings.llm_providers.providers {
        if result.room() == 0 {
            cut = true;
            continue;
        }
        let mut programs = List::with_capacity(limits.lines);
        let mut programs_cut = false;
        for name in &provider.offers.programs {
            if programs.room() == 0 {
                programs_cut = true;
                continue;
            }
            programs.push(name.clone()).or(Err(Missing::TooLarge))?;
        }
        result
            .push(ProviderOffer {
                provider: provider.number,
                name: super::small::text(&provider.name, limits.text_bytes),
                sign_in: provider.offers.sign_in,
                key: provider.offers.key,
                programs,
                programs_cut,
                none: provider.offers.none(),
                signed_in_plan: provider.signed_in_plan,
                manage_usage: provider.manage_usage.clone(),
            })
            .or(Err(Missing::TooLarge))?;
    }
    Ok((result, cut))
}

pub(super) fn rows(core: &Core, limits: &Limits, party: u64) -> Result<(List<PayerReadiness>, bool), Missing> {
    let mut rows = List::with_capacity(limits.documents.lines);
    let mut cut = false;
    append(core, party, tasks::Payer::Party(party), &mut rows, &mut cut)?;
    let roles = core.people.project_roles(party).ok_or(Missing::Unknown)?;
    for role in &roles {
        append(core, party, tasks::Payer::Project(role.project), &mut rows, &mut cut)?;
    }
    Ok((rows, cut))
}
fn append(
    core: &Core,
    party: u64,
    payer: tasks::Payer,
    rows: &mut List<PayerReadiness>,
    cut: &mut bool,
) -> Result<(), Missing> {
    for provider in &core.settings.llm_providers.providers {
        if rows.room() == 0 {
            *cut = true;
            continue;
        }
        let readiness = readiness(core, payer, provider.number);
        let mut settled = if provider.offers.programs.is_empty() { Some(true) } else { None };
        for pending in &core.accounts.pending() {
            if pending.by == party && pending.provider == provider.number {
                settled = Some(false);
            }
        }
        rows.push(PayerReadiness {
            provider: provider.number,
            payer: crate::coverage::payer(payer),
            readiness,
            settled,
        })
        .or(Err(Missing::TooLarge))?;
    }
    Ok(())
}

/// Full-state selection precedes any document truncation. No task pin is chosen here.
fn readiness(core: &Core, payer: tasks::Payer, provider: u32) -> Readiness {
    let coverage = core.account_coverage(payer);
    let selection = core.accounts.select(
        crate::coverage::payer(payer),
        provider,
        None,
        None,
        coverage.as_slice(),
        &[],
        core.run_now,
    );
    match selection {
        accounts::Selection::Chosen { .. } => eligible_host(core, provider),
        accounts::Selection::Wait { why: accounts::WaitWhy::Host, .. } => Readiness::Unknown,
        accounts::Selection::Wait { why: accounts::WaitWhy::Source(_), .. } => Readiness::Unavailable,
        accounts::Selection::Wait { why: accounts::WaitWhy::Refreshing | accounts::WaitWhy::Empty, .. }
        | accounts::Selection::NoAccount => match core.settings.llm_providers.get(provider) {
            Some(offer) if !offer.offers.programs.is_empty() => Readiness::Unknown,
            Some(_) | None => Readiness::Unavailable,
        },
    }
}
/// Host model support does not require a free slot. Until T12, unknown personal
/// program logins exclude workers as in run_choice; the engine can still serve.
fn eligible_host(core: &Core, provider: u32) -> Readiness {
    let mut unknown = false;
    let mut unknown_logins = false;
    for source in &core.accounts.source_views() {
        if source.kind == accounts::Kind::Program {
            unknown_logins = true;
        }
    }
    for host in &core.fleet.host_views() {
        if !host.connected || host.draining || host.slots == 0 {
            continue;
        }
        match host.kind {
            fleet::HostKind::Engine => {}
            fleet::HostKind::Worker if core.fleet.awaiting_logins(host.channel)
                || !fleet::host_rule(!unknown_logins, &[], &[]) => {
                    unknown = true;
                    continue;
                }
            fleet::HostKind::Worker => {}
        }
        for model in &core.settings.models {
            if core.provider(model) == Some(provider)
                && core.fleet.supports(
                    fleet::PlacementConstraint { channel: host.channel, kind: host.kind },
                    &[crate::fleet_needs::model(model)],
                )
            {
                return Readiness::Usable;
            }
        }
    }
    if unknown { Readiness::Unknown } else { Readiness::Unavailable }
}

pub(super) fn offers_worst_case(limits: &DocumentLimits) -> Option<u64> {
    let heap = List::<Box<[u8]>>::worst_case(limits.lines)?
        .checked_add(u64::from(limits.lines).checked_mul(32)?)?
        .checked_add(u64::from(limits.text_bytes))?
        .checked_add(u64::from(accounts::MANAGE_USAGE))?;
    List::<ProviderOffer>::worst_case(limits.lines)?.checked_add(u64::from(limits.lines).checked_mul(heap)?)
}
pub(super) fn rows_worst_case(limits: &DocumentLimits) -> Option<u64> {
    List::<PayerReadiness>::worst_case(limits.lines)
}
