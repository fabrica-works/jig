//! Ops values mapped explicitly to their versioned family records (root.md, 7).
use alloc::boxed::Box;
use jig_ops_client_protocol_wire::{codec, v1 as w};
/// A value outside the family wire's bounds or vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Invalid;
mod infrastructure_documents;
mod infrastructure_native;
mod observability_documents;
mod observability_native;
fn proposal_encode(value: &jig_client::contract::ProposalKey) -> Result<w::DecisionKey, Invalid> {
    use jig_client::contract::Proposer;
    let maker = match value.proposer {
        Proposer::Task(value) => {
            let Ok(value) = w::DecisionTask::new(&w::CEILINGS, w::DecisionTaskParts { value }) else {
                return Err(Invalid);
            };
            w::DecisionMaker::Task(value)
        }
        Proposer::Party(value) => {
            let Ok(value) = w::DecisionParty::new(&w::CEILINGS, w::DecisionPartyParts { value }) else {
                return Err(Invalid);
            };
            w::DecisionMaker::Party(value)
        }
    };
    let Ok(value) = w::DecisionKey::new(&w::CEILINGS, w::DecisionKeyParts { maker, number: value.number }) else {
        return Err(Invalid);
    };
    Ok(value)
}
fn proposal_decode(value: w::DecisionKey) -> jig_client::contract::ProposalKey {
    use jig_client::contract::Proposer;
    let value = value.into_parts();
    let proposer = match value.maker {
        w::DecisionMaker::Task(value) => Proposer::Task(value.value()),
        w::DecisionMaker::Party(value) => Proposer::Party(value.value()),
    };
    jig_client::contract::ProposalKey { proposer, number: value.number }
}
/// Infrastructure subjects and documents on the family wire.
pub mod infrastructure {
    use super::{Box, Invalid, codec, infrastructure_documents as values, w};
    use crate::infrastructure as d;
    /// Encode a whole subject.
    pub fn subject(value: &d::Subject) -> Result<Box<[u8]>, Invalid> {
        let Ok(record) = w::InfrastructureSubjectRecord::new(
            &w::CEILINGS,
            w::InfrastructureSubjectRecordParts { value: values::subject_encode(value)?.into_parts().kind },
        ) else {
            return Err(Invalid);
        };
        Ok(codec::infrastructure::subject(&record))
    }
    /// Decode a subject kind in the scope supplied by jig.
    pub fn subject_value(bytes: &[u8], scope: jig_client::contract::Scope) -> Result<d::Subject, Invalid> {
        let Ok(record) = codec::infrastructure::subject_value(bytes) else { return Err(Invalid) };
        let scope = match scope {
            jig_client::contract::Scope::Project(project) => d::Scope::Project(project),
            jig_client::contract::Scope::Deployment => d::Scope::Deployment,
            jig_client::contract::Scope::Own => return Err(Invalid),
        };
        Ok(d::Subject { scope, kind: values::kind_decode(record.into_parts().value) })
    }
    /// Encode a whole document.
    pub fn document(value: &d::Document) -> Result<Box<[u8]>, Invalid> {
        let Ok(record) = w::InfrastructureDocumentRecord::new(
            &w::CEILINGS,
            w::InfrastructureDocumentRecordParts { value: values::document_encode(value)? },
        ) else {
            return Err(Invalid);
        };
        Ok(codec::infrastructure::document_record(&record))
    }
    /// Decode a whole document.
    pub fn document_value(bytes: &[u8]) -> Result<d::Document, Invalid> {
        let Ok(record) = codec::infrastructure::document(bytes) else { return Err(Invalid) };
        Ok(values::document_decode(record.into_parts().value))
    }
    /// Encode an owner read result without losing its refusal.
    pub fn read_answer(value: &d::ReadResult) -> Result<Box<[u8]>, Invalid> {
        let value = match value {
            d::ReadResult::Value(value) => {
                let Ok(value) = w::InfrastructureReadValue::new(
                    &w::CEILINGS,
                    w::InfrastructureReadValueParts { value: values::document_encode(value)? },
                ) else {
                    return Err(Invalid);
                };
                w::InfrastructureReadResult::Value(Box::new(value))
            }
            d::ReadResult::NotAllowed => w::InfrastructureReadResult::NotAllowed,
            d::ReadResult::Unknown | d::ReadResult::Busy | d::ReadResult::SignedOut => {
                w::InfrastructureReadResult::Unknown
            }
        };
        let Ok(record) = w::InfrastructureReadAnswer::new(&w::CEILINGS, w::InfrastructureReadAnswerParts { value })
        else {
            return Err(Invalid);
        };
        Ok(codec::infrastructure::read_answer(&record))
    }
    /// Decode the owner's read result; malformed records are unknown.
    #[must_use]
    pub fn read_answer_value(bytes: &[u8]) -> d::ReadResult {
        match codec::infrastructure::read_answer_value(bytes) {
            Ok(record) => match record.into_parts().value {
                w::InfrastructureReadResult::Value(value) => {
                    d::ReadResult::Value(Box::new(values::document_decode(value.into_parts().value)))
                }
                w::InfrastructureReadResult::NotAllowed => d::ReadResult::NotAllowed,
                w::InfrastructureReadResult::Unknown => d::ReadResult::Unknown,
            },
            Err(_) => d::ReadResult::Unknown,
        }
    }
}
/// Observability subjects and documents on the family wire.
pub mod observability {
    use super::{Box, Invalid, codec, observability_documents as values, w};
    use crate::observability as d;
    /// Encode a whole subject.
    pub fn subject(value: &d::Subject) -> Result<Box<[u8]>, Invalid> {
        let Ok(record) = w::ObservabilitySubjectRecord::new(
            &w::CEILINGS,
            w::ObservabilitySubjectRecordParts { value: values::subject_encode(value)?.into_parts().kind },
        ) else {
            return Err(Invalid);
        };
        Ok(codec::observability::subject(&record))
    }
    /// Decode a subject kind in the scope supplied by jig.
    pub fn subject_value(bytes: &[u8], scope: jig_client::contract::Scope) -> Result<d::Subject, Invalid> {
        let Ok(record) = codec::observability::subject_value(bytes) else { return Err(Invalid) };
        let scope = match scope {
            jig_client::contract::Scope::Project(project) => d::Scope::Project(project),
            jig_client::contract::Scope::Deployment => d::Scope::Deployment,
            jig_client::contract::Scope::Own => return Err(Invalid),
        };
        Ok(d::Subject { scope, kind: values::kind_decode(record.into_parts().value) })
    }
    /// Encode a whole document.
    pub fn document(value: &d::Document) -> Result<Box<[u8]>, Invalid> {
        let Ok(record) = w::ObservabilityDocumentRecord::new(
            &w::CEILINGS,
            w::ObservabilityDocumentRecordParts { value: values::document_encode(value)? },
        ) else {
            return Err(Invalid);
        };
        Ok(codec::observability::document_record(&record))
    }
    /// Decode a whole document.
    pub fn document_value(bytes: &[u8]) -> Result<d::Document, Invalid> {
        let Ok(record) = codec::observability::document(bytes) else { return Err(Invalid) };
        Ok(values::document_decode(record.into_parts().value))
    }
    /// Encode an owner read result without losing its refusal.
    pub fn read_answer(value: &d::ReadResult) -> Result<Box<[u8]>, Invalid> {
        let value = match value {
            d::ReadResult::Value(value) => {
                let Ok(value) = w::ObservabilityReadValue::new(
                    &w::CEILINGS,
                    w::ObservabilityReadValueParts { value: values::document_encode(value)? },
                ) else {
                    return Err(Invalid);
                };
                w::ObservabilityReadResult::Value(Box::new(value))
            }
            d::ReadResult::NotAllowed => w::ObservabilityReadResult::NotAllowed,
            d::ReadResult::Unknown | d::ReadResult::Busy | d::ReadResult::SignedOut => {
                w::ObservabilityReadResult::Unknown
            }
        };
        let Ok(record) = w::ObservabilityReadAnswer::new(&w::CEILINGS, w::ObservabilityReadAnswerParts { value })
        else {
            return Err(Invalid);
        };
        Ok(codec::observability::read_answer(&record))
    }
    /// Decode the owner's read result; malformed records are unknown.
    #[must_use]
    pub fn read_answer_value(bytes: &[u8]) -> d::ReadResult {
        match codec::observability::read_answer_value(bytes) {
            Ok(record) => match record.into_parts().value {
                w::ObservabilityReadResult::Value(value) => {
                    d::ReadResult::Value(Box::new(values::document_decode(value.into_parts().value)))
                }
                w::ObservabilityReadResult::NotAllowed => d::ReadResult::NotAllowed,
                w::ObservabilityReadResult::Unknown => d::ReadResult::Unknown,
            },
            Err(_) => d::ReadResult::Unknown,
        }
    }
}
/// Encode an infrastructure ask.
pub fn ask(value: &crate::infrastructure::Ask) -> Result<Box<[u8]>, Invalid> {
    let Ok(record) = w::InfrastructureAskRecord::new(
        &w::CEILINGS,
        w::InfrastructureAskRecordParts { value: infrastructure_documents::ask_encode(value)? },
    ) else {
        return Err(Invalid);
    };
    Ok(codec::infrastructure::ask(&record))
}
/// Decode an infrastructure ask.
pub fn ask_value(bytes: &[u8]) -> Result<crate::infrastructure::Ask, Invalid> {
    let Ok(record) = codec::infrastructure::ask_value(bytes) else { return Err(Invalid) };
    Ok(infrastructure_documents::ask_decode(record.into_parts().value))
}
/// Encode an infrastructure answer.
pub fn answer(value: &crate::infrastructure::Answer) -> Result<Box<[u8]>, Invalid> {
    let Ok(record) = w::InfrastructureAnswerRecord::new(
        &w::CEILINGS,
        w::InfrastructureAnswerRecordParts { value: infrastructure_documents::answer_encode(value)? },
    ) else {
        return Err(Invalid);
    };
    Ok(codec::infrastructure::answer_record(&record))
}
/// Decode an infrastructure answer.
pub fn answer_value(bytes: &[u8]) -> Result<crate::infrastructure::Answer, Invalid> {
    let Ok(record) = codec::infrastructure::answer(bytes) else { return Err(Invalid) };
    Ok(infrastructure_documents::answer_decode(record.into_parts().value))
}
