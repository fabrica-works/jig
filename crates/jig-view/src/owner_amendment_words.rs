//! Closed amendment-state sayings (client/domain/view.md, 7).
//! IDs7100..7110 belong to this additive fragment, independent of request editing.
use crate::Owner;
use crate::ui::phrases::{
    Form, HoleKind, HoleShape, Language, Phrase, PhraseId, PhraseShape, PluralClass, PluralRule, Segment, SpanUnit,
    WallFormat,
};
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AmendmentPhrase {
    State,
    Party,
    Role,
    AddressAbsent,
    TakenBy,
    TakerAbsent,
    Instructions,
    Specification,
    Authority,
    Dependencies,
    DependenciesCut,
}
pub const ALL: [AmendmentPhrase; 11] = [
    AmendmentPhrase::State,
    AmendmentPhrase::Party,
    AmendmentPhrase::Role,
    AmendmentPhrase::AddressAbsent,
    AmendmentPhrase::TakenBy,
    AmendmentPhrase::TakerAbsent,
    AmendmentPhrase::Instructions,
    AmendmentPhrase::Specification,
    AmendmentPhrase::Authority,
    AmendmentPhrase::Dependencies,
    AmendmentPhrase::DependenciesCut,
];
impl AmendmentPhrase {
    #[must_use]
    pub fn id(self) -> PhraseId {
        let number = match self {
            Self::State => 7100,
            Self::Party => 7101,
            Self::Role => 7102,
            Self::AddressAbsent => 7103,
            Self::TakenBy => 7104,
            Self::TakerAbsent => 7105,
            Self::Instructions => 7106,
            Self::Specification => 7107,
            Self::Authority => 7108,
            Self::Dependencies => 7109,
            Self::DependenciesCut => 7110,
        };
        PhraseId { owner: Owner::Jig.opaque(), number }
    }
    fn numbered(self) -> bool {
        match self {
            Self::Party | Self::Role | Self::TakenBy | Self::Dependencies => true,
            Self::State
            | Self::AddressAbsent
            | Self::TakerAbsent
            | Self::Instructions
            | Self::Specification
            | Self::Authority
            | Self::DependenciesCut => false,
        }
    }
    fn label(self) -> &'static [u8] {
        match self {
            Self::State => b"Amendment state",
            Self::Party => b"Addressed party: ",
            Self::Role => b"Addressed role: ",
            Self::AddressAbsent => b"Address not supplied",
            Self::TakenBy => b"Taken by party: ",
            Self::TakerAbsent => b"Taker not supplied",
            Self::Instructions => b"Instructions",
            Self::Specification => b"Specification",
            Self::Authority => b"Authority",
            Self::Dependencies => b"Dependency task: ",
            Self::DependenciesCut => b"Dependency list incomplete",
        }
    }
}
#[must_use]
pub fn shapes() -> Box<[PhraseShape]> {
    let mut values = List::with_capacity(11);
    for phrase in ALL {
        let holes: Box<[HoleShape]> =
            if phrase.numbered() { Box::new([HoleShape { kind: HoleKind::Number, bytes: 0 }]) } else { Box::new([]) };
        values.push(PhraseShape { id: phrase.id(), holes, plural: None }).expect("closed amendment shapes fit");
    }
    values.into_boxed()
}
#[must_use]
pub fn language() -> Language {
    let mut values = List::with_capacity(11);
    for phrase in ALL {
        let segments: Box<[Segment]> = if phrase.numbered() {
            Box::new([Segment::Text(copy_of(phrase.label())), Segment::Hole { index: 0, kind: HoleKind::Number }])
        } else {
            Box::new([Segment::Text(copy_of(phrase.label()))])
        };
        values
            .push(Phrase { id: phrase.id(), forms: Box::new([Form { class: PluralClass::Other, segments }]) })
            .expect("closed amendment sayings fit");
    }
    Language {
        plural: PluralRule::English,
        span_unit: SpanUnit::Nanos,
        wall: WallFormat::Nanos,
        phrases: values.into_boxed(),
    }
}
