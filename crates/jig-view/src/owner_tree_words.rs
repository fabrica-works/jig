//! Checked subtree sayings (client/domain/view.md, 7). IDs8300–8321.
use crate::Owner;
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
use skein_ui::phrases::{
    Form, HoleKind, HoleShape, Language, Phrase, PhraseId, PhraseShape, PluralClass, PluralRule, Segment, SpanUnit,
    WallFormat,
};
/// The subtree's closed facts and independent phase counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreePhrase {
    Tree,
    Root,
    Parent,
    Depth,
    Spent,
    Dependencies,
    Proposals,
    RowCut,
    Fold,
    Held,
    Next,
    End,
    Waiting,
    Idle,
    Starting,
    Running,
    BackingOff,
    Closing,
    HeldCount,
    Done,
    Failed,
    Cancelled,
}
/// All subtree sayings in stable order.
pub const ALL: [TreePhrase; 22] = [
    TreePhrase::Tree,
    TreePhrase::Root,
    TreePhrase::Parent,
    TreePhrase::Depth,
    TreePhrase::Spent,
    TreePhrase::Dependencies,
    TreePhrase::Proposals,
    TreePhrase::RowCut,
    TreePhrase::Fold,
    TreePhrase::Held,
    TreePhrase::Next,
    TreePhrase::End,
    TreePhrase::Waiting,
    TreePhrase::Idle,
    TreePhrase::Starting,
    TreePhrase::Running,
    TreePhrase::BackingOff,
    TreePhrase::Closing,
    TreePhrase::HeldCount,
    TreePhrase::Done,
    TreePhrase::Failed,
    TreePhrase::Cancelled,
];
impl TreePhrase {
    /// Stable normal phrasebook identity.
    #[must_use]
    pub fn id(self) -> PhraseId {
        let number = match self {
            Self::Tree => 8300,
            Self::Root => 8301,
            Self::Parent => 8302,
            Self::Depth => 8303,
            Self::Spent => 8304,
            Self::Dependencies => 8305,
            Self::Proposals => 8306,
            Self::RowCut => 8307,
            Self::Fold => 8308,
            Self::Held => 8309,
            Self::Next => 8310,
            Self::End => 8311,
            Self::Waiting => 8312,
            Self::Idle => 8313,
            Self::Starting => 8314,
            Self::Running => 8315,
            Self::BackingOff => 8316,
            Self::Closing => 8317,
            Self::HeldCount => 8318,
            Self::Done => 8319,
            Self::Failed => 8320,
            Self::Cancelled => 8321,
        };
        PhraseId { owner: Owner::Jig.opaque(), number }
    }
    fn hole(self) -> Option<HoleKind> {
        match self {
            Self::Root
            | Self::Parent
            | Self::Depth
            | Self::Held
            | Self::Waiting
            | Self::Idle
            | Self::Starting
            | Self::Running
            | Self::BackingOff
            | Self::Closing
            | Self::HeldCount
            | Self::Done
            | Self::Failed
            | Self::Cancelled => Some(HoleKind::Number),
            Self::Spent => Some(HoleKind::Amount),
            Self::Tree | Self::Dependencies | Self::Proposals | Self::RowCut | Self::Fold | Self::Next | Self::End => {
                None
            }
        }
    }
}
/// Closed shapes for normal selected-language composition.
#[must_use]
pub fn shapes() -> Box<[PhraseShape]> {
    let mut result = List::with_capacity(u32::try_from(ALL.len()).expect("closed phrase count"));
    for phrase in ALL {
        let holes: Box<[HoleShape]> = match phrase.hole() {
            Some(kind) => Box::new([HoleShape { kind, bytes: 0 }]),
            None => Box::new([]),
        };
        result.push(PhraseShape { id: phrase.id(), holes, plural: None }).expect("closed shapes fit");
    }
    result.into_boxed()
}
/// English data is composed while loading, never during rendering.
#[must_use]
pub fn language() -> Language {
    let mut result = List::with_capacity(u32::try_from(ALL.len()).expect("closed phrase count"));
    for phrase in ALL {
        let bytes: &[u8] = match phrase {
            TreePhrase::Tree => b"Plan subtree",
            TreePhrase::Root => b"Root task: ",
            TreePhrase::Parent => b"Parent task: ",
            TreePhrase::Depth => b"Depth from root: ",
            TreePhrase::Spent => b"Subtree row spend: ",
            TreePhrase::Dependencies => b"Dependencies",
            TreePhrase::Proposals => b"Waiting proposals",
            TreePhrase::RowCut => b"Some dependencies are outside this row",
            TreePhrase::Fold => b"Folded descendants",
            TreePhrase::Held => b"Held below: ",
            TreePhrase::Next => b"More descendants remain in the supplied window",
            TreePhrase::End => b"The supplied window has no continuation",
            TreePhrase::Waiting => b"Waiting: ",
            TreePhrase::Idle => b"Idle: ",
            TreePhrase::Starting => b"Starting: ",
            TreePhrase::Running => b"Running: ",
            TreePhrase::BackingOff => b"Backing off: ",
            TreePhrase::Closing => b"Closing: ",
            TreePhrase::HeldCount => b"Held: ",
            TreePhrase::Done => b"Done: ",
            TreePhrase::Failed => b"Failed: ",
            TreePhrase::Cancelled => b"Cancelled: ",
        };
        let segments: Box<[Segment]> = match phrase.hole() {
            Some(kind) => Box::new([Segment::Text(copy_of(bytes)), Segment::Hole { index: 0, kind }]),
            None => Box::new([Segment::Text(copy_of(bytes))]),
        };
        result
            .push(Phrase { id: phrase.id(), forms: Box::new([Form { class: PluralClass::Other, segments }]) })
            .expect("closed words fit");
    }
    Language {
        plural: PluralRule::English,
        span_unit: SpanUnit::Nanos,
        wall: WallFormat::Nanos,
        phrases: result.into_boxed(),
    }
}
