//! jig's phrase tables and their envelope (client/domain/view.md, section 7).

pub mod english;
pub mod jig;

use skein_ui::phrases::Phrasebook;

/// Maximum of the complete jig vocabulary, or none for a partial book.
#[must_use]
pub fn jig_capacity(book: &Phrasebook) -> Option<u32> {
    let mut largest = 0;
    for phrase in jig::ALL {
        largest = largest.max(book.capacity(phrase.id())?);
    }
    Some(largest.max(crate::owner_words::phrase_bytes(book)?))
}
