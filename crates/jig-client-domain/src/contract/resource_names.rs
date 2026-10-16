//! Literal segmented names (domain/authority.md, 4; client/domain/contract.md, 5.3).
use alloc::boxed::Box;
use skein_lib::List;

use super::{OwnerPattern, OwnerPatternEnd};

/// Each allocated segment and its opaque bytes fit the decoding owner's bounds.
#[must_use]
pub fn resource_path_fits(path: &List<Box<[u8]>>, rows: u32, text: u32) -> bool {
    if path.capacity() > rows {
        return false;
    }
    for segment in path {
        if segment.len() > usize::try_from(text).expect("u32 fits usize") {
            return false;
        }
    }
    true
}

/// The exact pattern for one whole nonempty supplied name; no bytes are parsed.
/// An empty path has no terminal segment, so supplies no exact pattern.
#[must_use]
pub fn exact_resource_pattern(path: &List<Box<[u8]>>) -> Option<OwnerPattern> {
    let last = path.as_slice().last()?.clone();
    let preceding = path.len().checked_sub(1)?;
    let mut segments = List::with_capacity(preceding);
    for segment in path.as_slice().get(..usize::try_from(preceding).expect("u32 fits usize"))? {
        segments.push(segment.clone()).expect("exact preceding segment count fits");
    }
    Some(OwnerPattern { segments, last: OwnerPatternEnd::Exact(last) })
}
