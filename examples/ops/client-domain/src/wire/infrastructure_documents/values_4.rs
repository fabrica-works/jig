//! Whole-value mappings for infrastructure/documents.
use crate::infrastructure::documents as d;
use crate::wire::{Invalid, w};
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn answer_encode(value: &d::Answer) -> Result<w::InfrastructureAnswer, Invalid> {
    Ok(match value {
        d::Answer::MovedEnd { environment, revision, until } => {
            let parts = w::InfrastructureAnswerMovedEndParts {
                environment: (*environment),
                revision: revision.value,
                until: (*until).as_nanos(),
            };
            let Ok(value) = w::InfrastructureAnswerMovedEnd::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureAnswer::MovedEnd(value)
        }
        d::Answer::NativeMovedEnd { environment, revision, until } => {
            let parts = w::InfrastructureAnswerNativeMovedEndParts {
                environment: crate::wire::infrastructure_native::environment_encode(environment)?,
                revision: revision.value,
                until: (*until),
            };
            let Ok(value) = w::InfrastructureAnswerNativeMovedEnd::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureAnswer::NativeMovedEnd(value)
        }
        d::Answer::NativeMoved { environment, revision, until } => {
            let parts = w::InfrastructureAnswerNativeMovedParts {
                environment: crate::wire::infrastructure_native::environment_encode(environment)?,
                revision: revision.value,
                until: (*until),
            };
            let Ok(value) = w::InfrastructureAnswerNativeMoved::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureAnswer::NativeMoved(value)
        }
        d::Answer::Moved { environment, revision, until } => {
            let parts = w::InfrastructureAnswerMovedParts {
                environment: (*environment),
                revision: revision.value,
                until: match until {
                    Some(item) => Some((*item).as_nanos()),
                    None => None,
                },
            };
            let Ok(value) = w::InfrastructureAnswerMoved::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureAnswer::Moved(value)
        }
        d::Answer::Gone => w::InfrastructureAnswer::Gone,
        d::Answer::Refused { why } => {
            let parts = w::InfrastructureAnswerRefusedParts {
                why: crate::wire::infrastructure_documents::refusal_encode(*why)?,
            };
            let Ok(value) = w::InfrastructureAnswerRefused::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureAnswer::Refused(value)
        }
        d::Answer::EntranceRefused(_) => return Err(Invalid),
    })
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn answer_decode(value: w::InfrastructureAnswer) -> d::Answer {
    match value {
        w::InfrastructureAnswer::MovedEnd(value) => {
            let value = value.into_parts();
            d::Answer::MovedEnd {
                environment: value.environment,
                revision: jig_client::contract::Revision { value: value.revision },
                until: skein_lib::Wall::from_nanos(value.until),
            }
        }
        w::InfrastructureAnswer::NativeMovedEnd(value) => {
            let value = value.into_parts();
            d::Answer::NativeMovedEnd {
                environment: crate::wire::infrastructure_native::environment_decode(value.environment),
                revision: jig_client::contract::Revision { value: value.revision },
                until: value.until,
            }
        }
        w::InfrastructureAnswer::NativeMoved(value) => {
            let value = value.into_parts();
            d::Answer::NativeMoved {
                environment: crate::wire::infrastructure_native::environment_decode(value.environment),
                revision: jig_client::contract::Revision { value: value.revision },
                until: value.until,
            }
        }
        w::InfrastructureAnswer::Moved(value) => {
            let value = value.into_parts();
            d::Answer::Moved {
                environment: value.environment,
                revision: jig_client::contract::Revision { value: value.revision },
                until: match value.until {
                    Some(item) => Some(skein_lib::Wall::from_nanos(item)),
                    None => None,
                },
            }
        }
        w::InfrastructureAnswer::Gone => d::Answer::Gone,
        w::InfrastructureAnswer::Refused(value) => {
            let value = value.into_parts();
            d::Answer::Refused { why: crate::wire::infrastructure_documents::refusal_decode(value.why) }
        }
    }
}
