//! Person task facts are carried whole; this module supplies its typed projection.
//! Nothing computes authority or supplies words. See client/domain/objects.md, 7.
use super::{Body, Object};
use alloc::boxed::Box;
/// Reads the document without copying the working set's one object.
#[must_use]
pub fn facts(object: &Object) -> Option<&crate::contract::PersonTask> {
    match &object.body {
        Some(Body::PersonTask(document)) => Some(document),
        Some(
            Body::OwnerPersonTask(_)
            | Body::Historical(_)
            | Body::Chip(_)
            | Body::Task(_)
            | Body::Proposal(_)
            | Body::OwnerProposal(_)
            | Body::Escalation(_)
            | Body::OwnerEscalation(_)
            | Body::Question(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => None,
    }
}
/// Names only the independently decidable facts on this card.
#[must_use]
pub fn revision(object: &Object) -> Option<crate::contract::Revision> {
    if let Some(document) = owner_facts(object) {
        return Some(document.revision);
    }
    let document = facts(object)?;
    Some(document.revision)
}

/// The separate pending owner envelope in this working-set object.
#[must_use]
pub fn owner_facts(object: &Object) -> Option<&crate::contract::OwnerPersonTask> {
    match &object.body {
        Some(Body::OwnerPersonTask(document)) => Some(document),
        Some(
            Body::PersonTask(_)
            | Body::OwnerEscalation(_)
            | Body::Historical(_)
            | Body::Chip(_)
            | Body::Task(_)
            | Body::Proposal(_)
            | Body::OwnerProposal(_)
            | Body::Escalation(_)
            | Body::Question(_)
            | Body::Result(_)
            | Body::Note { .. },
        )
        | None => None,
    }
}
/// Positive forms the current person answer schema can express.
#[expect(clippy::manual_map, reason = "the strict step subset uses explicit Option matches without closures")]
#[must_use]
pub fn answer(
    document: &crate::contract::OwnerPersonTask,
    option: Option<u32>,
    fields: &crate::fields::Fields,
) -> Option<crate::contract::PersonAnswer> {
    use crate::{
        contract::{PersonAnswer, TaskContractDocument, Text},
        fields::Purpose,
    };
    match &document.contract {
        TaskContractDocument::Approval { state, .. } => Some(PersonAnswer::Approved {
            state: state.clone(),
            words: match fields.get(Purpose::Reason) {
                Some(field) => Some(field.text.clone()),
                None => None,
            },
        }),
        TaskContractDocument::Report { words } => {
            let field = fields.get(Purpose::Answer)?;
            if !field.text.fits(*words) {
                return None;
            }
            Some(PersonAnswer::Words(field.text.clone()))
        }
        TaskContractDocument::Verdict { choices, .. } => {
            let option = option?;
            for choice in choices {
                if choice.code == option && choice.fields.is_empty() {
                    let reason = match fields.get(Purpose::Reason) {
                        Some(field) => field.text.clone(),
                        None => Text::new(Box::from([]), 0).expect("empty text fits"),
                    };
                    if !reason.fits(choice.words) {
                        return None;
                    }
                    return Some(PersonAnswer::Choice { option, reason });
                }
            }
            None
        }
        TaskContractDocument::Change { .. } => None,
    }
}
