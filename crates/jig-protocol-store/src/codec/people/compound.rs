//! Bounded compound task operations in the people v3 family.
use super::{Box, List, domain, wire};
pub(super) fn encode(
    values: &List<domain::Operation>,
    limits: &wire::Limits,
) -> Result<List<wire::Operation>, wire::Problem> {
    let mut result = List::with_capacity(values.len());
    for value in values {
        let translated = match value {
            domain::Operation::Cancel { task, reason, grace } => wire::Operation::Cancel(wire::OperationCancel::new(
                limits,
                wire::OperationCancelParts { task: *task, reason: reason.clone(), grace: encode_grace(*grace) },
            )?),
            domain::Operation::Move { task, reason } => wire::Operation::MoveTask(wire::OperationMove::new(
                limits,
                wire::OperationMoveParts { task: *task, reason: reason.clone() },
            )?),
            domain::Operation::Amend { task, revision, amendment, release } => {
                wire::Operation::Amend(Box::new(wire::OperationAmend::new(
                    limits,
                    wire::OperationAmendParts {
                        task: *task,
                        revision: *revision,
                        amendment: super::encode_amendment(amendment, limits)?,
                        release: *release,
                    },
                )?))
            }
            domain::Operation::Release { task, revision, again } => {
                wire::Operation::Release(wire::OperationRelease::new(
                    limits,
                    wire::OperationReleaseParts { task: *task, revision: *revision, again: *again },
                )?)
            }
            domain::Operation::Message { task, words } => wire::Operation::Message(wire::OperationMessage::new(
                limits,
                wire::OperationMessageParts { task: *task, words: words.clone() },
            )?),
            domain::Operation::Stop { task } => {
                wire::Operation::Stop(wire::OperationStop::new(limits, wire::OperationStopParts { task: *task })?)
            }
        };
        result.push(translated).expect("same operation count");
    }
    Ok(result)
}
pub(super) fn decode(
    values: List<wire::Operation>,
    limits: &wire::Limits,
) -> Result<List<domain::Operation>, wire::Problem> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        let translated = match value {
            wire::Operation::Cancel(value) => {
                let wire::OperationCancelParts { task, reason, grace } = value.into_parts();
                domain::Operation::Cancel { task, reason, grace: decode_grace(grace) }
            }
            wire::Operation::MoveTask(value) => {
                let wire::OperationMoveParts { task, reason } = value.into_parts();
                domain::Operation::Move { task, reason }
            }
            wire::Operation::Amend(value) => {
                let wire::OperationAmendParts { task, revision, amendment, release } = (*value).into_parts();
                domain::Operation::Amend {
                    task,
                    revision,
                    amendment: Box::new(super::decode_amendment(amendment, limits)?),
                    release,
                }
            }
            wire::Operation::Release(value) => {
                let wire::OperationReleaseParts { task, revision, again } = value.into_parts();
                domain::Operation::Release { task, revision, again }
            }
            wire::Operation::Message(value) => {
                let wire::OperationMessageParts { task, words } = value.into_parts();
                domain::Operation::Message { task, words }
            }
            wire::Operation::Stop(value) => {
                let wire::OperationStopParts { task } = value.into_parts();
                domain::Operation::Stop { task }
            }
        };
        result.push(translated).expect("same operation count");
    }
    Ok(result)
}

pub(super) fn encode_grace(grace: Option<skein_lib::Duration>) -> Option<u64> {
    Some(grace?.as_nanos())
}
pub(super) fn decode_grace(grace: Option<u64>) -> Option<skein_lib::Duration> {
    Some(skein_lib::Duration::from_nanos(grace?))
}
