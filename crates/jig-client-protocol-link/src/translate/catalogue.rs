//! Catalogue decoding owns no account selection (contract.md, 5.1 and 7.1).
use super::accounts::{charge, checked_manage_usage, piece_count};
use super::{Limits, List, Reader, c, piece_problem, too_large, w};

pub(super) fn offers(bytes: &[u8], limits: &Limits, room: &mut u64) -> Result<List<c::ProviderOffer>, w::Problem> {
    let count = piece_count(bytes)?;
    if count > limits.client.rows {
        return Err(too_large());
    }
    let mut rows: List<c::ProviderOffer> = List::with_capacity(count);
    charge(room, List::<c::ProviderOffer>::worst_case(rows.capacity()))?;
    let mut reader = Reader::new(bytes);
    for _ in 0..count {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(item) => item,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else { return Err(piece_problem(skein_codec::Reason::Malformed)) };
        let value = w::ProviderOffer::decode(&limits.wire, &mut Reader::new(&item))?;
        if value.key_bytes() > limits.wire.keyed_value_some {
            return Err(too_large());
        }
        if value.programs().len() > limits.client.rows {
            return Err(too_large());
        }
        charge(room, List::<c::Text>::worst_case(value.programs().len()))?;
        charge(room, Some(u64::try_from(value.name().bytes().len()).or(Err(too_large()))?))?;
        let mut programs = List::with_capacity(value.programs().len());
        for name in value.programs() {
            charge(room, Some(u64::try_from(name.len()).or(Err(too_large()))?))?;
            programs
                .push(c::Text::new(name.clone(), limits.wire.provider_offer_programs_item).or(Err(too_large()))?)
                .or(Err(too_large()))?;
        }
        let address = checked_manage_usage(value.manage_usage().as_deref(), w::Path::ProviderOfferManageUsage)?;
        if let Some(address) = &address {
            charge(room, Some(u64::try_from(address.len()).or(Err(too_large()))?))?;
        }
        for old in &rows {
            if old.provider == value.provider() {
                return Err(piece_problem(skein_codec::Reason::Malformed));
            }
        }
        rows.push(c::ProviderOffer {
            provider: value.provider(),
            key_bytes: value.key_bytes(),
            name: crate::translate::facts::document_text(value.name(), limits)?,
            sign_in: value.sign_in(),
            key: value.key(),
            programs,
            programs_cut: value.programs_cut(),
            none: value.none(),
            signed_in_plan: value.signed_in_plan(),
            manage_usage: address,
        })
        .or(Err(too_large()))?;
    }
    Ok(rows)
}
pub(super) fn readiness(bytes: &[u8], limits: &Limits, room: &mut u64) -> Result<List<c::PayerReadiness>, w::Problem> {
    let count = piece_count(bytes)?;
    if count > limits.client.rows {
        return Err(too_large());
    }
    let mut rows: List<c::PayerReadiness> = List::with_capacity(count);
    charge(room, List::<c::PayerReadiness>::worst_case(rows.capacity()))?;
    let mut reader = Reader::new(bytes);
    for _ in 0..count {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(item) => item,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else { return Err(piece_problem(skein_codec::Reason::Malformed)) };
        let value = w::PayerReadiness::decode(&limits.wire, &mut Reader::new(&item))?;
        let payer = super::accounts::owner_account_use(value.payer());
        for old in &rows {
            if old.provider == value.provider() && old.payer == payer {
                return Err(piece_problem(skein_codec::Reason::Malformed));
            }
        }
        rows.push(c::PayerReadiness {
            provider: value.provider(),
            payer,
            readiness: match value.readiness() {
                w::Readiness::Usable => c::Readiness::Usable,
                w::Readiness::Unavailable => c::Readiness::Unavailable,
                w::Readiness::Unknown => c::Readiness::Unknown,
            },
            settled: *value.settled(),
        })
        .or(Err(too_large()))?;
    }
    Ok(rows)
}
