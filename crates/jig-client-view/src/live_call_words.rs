//! Neutral controls for provisional raw input; client/domain/view.md, 7 and 9.
//! IDs 6800–6801 keep raw byte display outside bounded name holes.
use crate::{
    Owner,
    phrases::{Form, Language, Phrase, PhraseId, PhraseShape, PluralClass, PluralRule, Segment, SpanUnit, WallFormat},
};
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiveCallPhrase {
    OpenCurrentCallInput,
    FoldCurrentCallInput,
}
pub const ALL: [LiveCallPhrase; 2] = [LiveCallPhrase::OpenCurrentCallInput, LiveCallPhrase::FoldCurrentCallInput];
impl LiveCallPhrase {
    #[must_use]
    pub const fn id(self) -> PhraseId {
        let number = match self {
            Self::OpenCurrentCallInput => 6800,
            Self::FoldCurrentCallInput => 6801,
        };
        PhraseId { owner: Owner::Jig, number }
    }
}
#[must_use]
pub fn shapes() -> Box<[PhraseShape]> {
    let mut rows = List::with_capacity(2);
    for phrase in ALL {
        rows.push(PhraseShape { id: phrase.id(), holes: Box::new([]), plural: None })
            .expect("two live call shapes fit");
    }
    rows.into_boxed()
}
/// English data for the caller's normal composition, never constructed by renderers.
#[must_use]
pub fn language() -> Language {
    let mut phrases = List::with_capacity(2);
    for phrase in ALL {
        let bytes: &[u8] = match phrase {
            LiveCallPhrase::OpenCurrentCallInput => b"Open current call input",
            LiveCallPhrase::FoldCurrentCallInput => b"Fold current call input",
        };
        phrases
            .push(Phrase {
                id: phrase.id(),
                forms: Box::new([Form {
                    class: PluralClass::Other,
                    segments: Box::new([Segment::Text(copy_of(bytes))]),
                }]),
            })
            .expect("two live call sayings fit");
    }
    Language {
        plural: PluralRule::English,
        span_unit: SpanUnit::Nanos,
        wall: WallFormat::Nanos,
        phrases: phrases.into_boxed(),
    }
}
