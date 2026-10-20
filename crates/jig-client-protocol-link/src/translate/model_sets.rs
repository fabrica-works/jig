//! Permission selectors preserve omission, literal ALL, and empty/named sets.
use super::w;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use skein_lib::List;

pub(super) fn encode(
    value: Option<&c::AuthorityModels>,
    limits: &w::Limits,
) -> Result<Option<w::ModelSet>, w::Problem> {
    Ok(match value {
        None => None,
        Some(c::AuthorityModels::All) => Some(w::ModelSet::All),
        Some(c::AuthorityModels::Named(names)) => {
            validate(names)?;
            Some(w::ModelSet::Names(w::ModelSetNames::new(limits, w::ModelSetNamesParts { names: names.clone() })?))
        }
    })
}

pub(super) fn decode(value: Option<&w::ModelSet>) -> Result<Option<c::AuthorityModels>, w::Problem> {
    Ok(match value {
        None => None,
        Some(w::ModelSet::All) => Some(c::AuthorityModels::All),
        Some(w::ModelSet::Names(value)) => {
            validate(value.names())?;
            Some(c::AuthorityModels::Named(value.names().clone()))
        }
    })
}

fn validate(names: &List<Box<[u8]>>) -> Result<(), w::Problem> {
    let bad = w::Problem { path: w::Path::ModelSetNamesNames, reason: skein_codec::Reason::Bound };
    if names.len() > c::AUTHORITY_MODELS {
        return Err(bad);
    }
    for (at, name) in names.iter().enumerate() {
        if name.is_empty()
            || name.len() > usize::try_from(c::AUTHORITY_MODEL_BYTES).expect("name bound")
            || !skein_codec::text_is_valid(name)
        {
            return Err(bad);
        }
        for earlier in names.iter().take(at) {
            if earlier == name {
                return Err(bad);
            }
        }
    }
    Ok(())
}
