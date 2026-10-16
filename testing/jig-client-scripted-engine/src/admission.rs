//! Whole typed protocol payload admission for real and scripted fronts
//! (client/domain/contract.md, 5.3). These helpers reuse the kit's existing
//! recursive allocation accounting; they neither interpret nor clip values.
use crate::{Bounds, Invalid, payload};
use jig_domain_client::contract::{ActivityLine, Answer, Delivery, Frame, ReadValue};

/// Admit a complete frame, including every allocated nested list and text.
pub fn admit_frame(value: &Frame, bounds: &Bounds) -> Result<(), Invalid> {
    if payload::fits(value, bounds) { Ok(()) } else { Err(Invalid::Payload) }
}

/// Admit the complete read envelope and its boxed owner document.
pub fn admit_read_value(value: &ReadValue, bounds: &Bounds) -> Result<(), Invalid> {
    if payload::fits(value, bounds) { Ok(()) } else { Err(Invalid::Payload) }
}

/// Admit the complete normal watch delivery and its boxed document.
pub fn admit_delivery(value: &Delivery, bounds: &Bounds) -> Result<(), Invalid> {
    if payload::fits(value, bounds) { Ok(()) } else { Err(Invalid::Payload) }
}

/// Admit the complete keyed answer with its actual terminal or refusal facts.
pub fn admit_answer(value: &Answer, bounds: &Bounds) -> Result<(), Invalid> {
    if payload::fits(value, bounds) { Ok(()) } else { Err(Invalid::Payload) }
}

/// Admit an immutable activity line and any separately decoded archive payload.
pub fn admit_activity_line(value: &ActivityLine, bounds: &Bounds) -> Result<(), Invalid> {
    if payload::fits(value, bounds) { Ok(()) } else { Err(Invalid::Payload) }
}
