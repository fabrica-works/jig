//! Public configured offers and volatile payer facts (contract.md, 5.1).
use super::{accounts, owner, pieces, w};
use alloc::boxed::Box;
use jig_domain_core::documents as d;
use skein_lib::{List, Writer};

pub(super) fn offers(values: &List<d::ProviderOffer>, bound: u32, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
    let mut rows = List::with_capacity(values.len());
    for value in values {
        let row = w::ProviderOffer::new(
            limits,
            w::ProviderOfferParts {
                skein_bools: [value.sign_in, value.key, value.programs_cut, value.none, value.signed_in_plan],
                provider: value.provider,
                key_bytes: value.key_bytes,
                name: owner::text(&value.name, limits)?,
                programs: value.programs.clone(),
                manage_usage: value.manage_usage.clone(),
            },
        )?;
        let mut out = Writer::new(usize::try_from(row.measure()).or(Err(super::problem(skein_codec::Reason::Bound)))?);
        row.encode(&mut out).or(Err(super::problem(skein_codec::Reason::Bound)))?;
        rows.push(out.finish()).or(Err(super::problem(skein_codec::Reason::Bound)))?;
    }
    pieces(&rows, bound, w::Path::MeProviders)
}
pub(super) fn readiness(
    values: &List<d::PayerReadiness>,
    bound: u32,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut rows = List::with_capacity(values.len());
    for value in values {
        let row = w::PayerReadiness::new(
            limits,
            w::PayerReadinessParts {
                provider: value.provider,
                payer: accounts::payer(value.payer, limits)?,
                readiness: match value.readiness {
                    d::Readiness::Usable => w::Readiness::Usable,
                    d::Readiness::Unavailable => w::Readiness::Unavailable,
                    d::Readiness::Unknown => w::Readiness::Unknown,
                },
                settled: value.settled,
            },
        )?;
        let mut out = Writer::new(usize::try_from(row.measure()).or(Err(super::problem(skein_codec::Reason::Bound)))?);
        row.encode(&mut out).or(Err(super::problem(skein_codec::Reason::Bound)))?;
        rows.push(out.finish()).or(Err(super::problem(skein_codec::Reason::Bound)))?;
    }
    pieces(&rows, bound, w::Path::OwnerAccountsReadiness)
}
