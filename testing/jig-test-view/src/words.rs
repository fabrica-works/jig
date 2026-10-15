//! Family phrases are checked data beside jig's phrasebook (view.md, section 7).
use alloc::boxed::Box;
use jig_view::Owner;
use jig_view::ui::phrases::{
    Form, Hole, HoleKind, HoleShape, Invalid, Language, Phrase, PhraseId, PhraseLimits, PhraseShape, Phrasebook,
    PluralClass, PluralRule, Segment, SpanUnit, WallFormat,
};
use jig_view::ui::{Builder, Role, Run};

use skein_lib::{List, bytes::copy_of};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Word {
    Navigation,
    Family,
    Item,
    Loading,
    Unknown,
    Unshowable,
    Value,
    Drawing,
    Ready,
    Held,
    Ended,
    Touch,
    Parameters,
    Apply,
    Deciding,
    Retrying,
    Parked,
    Touched,
    Moved,
    Gone,
    Connector,
    Resource,
    Kind,
    Part,
    OutboxEntry,
    NotAllowed,
    SubjectEnded,
    WatchBusy,
    WatchStopping,
}
const ALL: [Word; 29] = [
    Word::Navigation,
    Word::Family,
    Word::Item,
    Word::Loading,
    Word::Unknown,
    Word::Unshowable,
    Word::Value,
    Word::Drawing,
    Word::Ready,
    Word::Held,
    Word::Ended,
    Word::Touch,
    Word::Parameters,
    Word::Apply,
    Word::Deciding,
    Word::Retrying,
    Word::Parked,
    Word::Touched,
    Word::Moved,
    Word::Gone,
    Word::Connector,
    Word::Resource,
    Word::Kind,
    Word::Part,
    Word::OutboxEntry,
    Word::NotAllowed,
    Word::SubjectEnded,
    Word::WatchBusy,
    Word::WatchStopping,
];
impl Word {
    #[must_use]
    pub const fn id(self) -> PhraseId {
        let number = match self {
            Self::Navigation => 0,
            Self::Family => 1,
            Self::Item => 2,
            Self::Loading => 3,
            Self::Unknown => 4,
            Self::Unshowable => 5,
            Self::Value => 6,
            Self::Drawing => 7,
            Self::Ready => 8,
            Self::Held => 9,
            Self::Ended => 10,
            Self::Touch => 11,
            Self::Parameters => 12,
            Self::Apply => 13,
            Self::Deciding => 14,
            Self::Retrying => 15,
            Self::Parked => 16,
            Self::Touched => 17,
            Self::Moved => 18,
            Self::Gone => 19,
            Self::Connector => 20,
            Self::Resource => 21,
            Self::Kind => 22,
            Self::Part => 23,
            Self::OutboxEntry => 24,
            Self::NotAllowed => 7800,
            Self::SubjectEnded => 7801,
            Self::WatchBusy => 7802,
            Self::WatchStopping => 7803,
        };
        PhraseId { owner: (Owner::Application { number: jig_test_domain_client::FIRST }).opaque(), number }
    }
}
#[derive(Clone, Debug)]
pub struct Words {
    pub book: Phrasebook,
    pub language: u16,
}
impl Words {
    pub fn english(text: u32) -> Result<Self, Invalid> {
        let mut shapes = List::with_capacity(29);
        let mut phrases = List::with_capacity(29);
        for word in ALL {
            let (literal, hole) = match word {
                Word::Navigation => (b"Family".as_slice(), None),
                Word::Family => (b"Family item".as_slice(), None),
                Word::Item => (b"Item ".as_slice(), Some(HoleKind::Number)),
                Word::Loading => (b"Loading family document".as_slice(), None),
                Word::Unknown => (b"Unknown family object".as_slice(), None),
                Word::Unshowable => (b"Cannot show this part".as_slice(), None),
                Word::Value => (b"Value: ".as_slice(), Some(HoleKind::Number)),
                Word::Drawing => (b"Item value".as_slice(), None),
                Word::Ready => (b"Ready".as_slice(), None),
                Word::Held => (b"Held".as_slice(), None),
                Word::Ended => (b"Ended".as_slice(), None),
                Word::Touch => (b"Touch".as_slice(), None),
                Word::Parameters => (b"Parameters".as_slice(), None),
                Word::Apply => (b"Apply parameters".as_slice(), None),
                Word::Deciding => (b"In flight".as_slice(), None),
                Word::Retrying => (b"Retrying".as_slice(), None),
                Word::Parked => (b"Parked".as_slice(), None),
                Word::Touched => (b"Touched revision ".as_slice(), Some(HoleKind::Number)),
                Word::Moved => (b"Moved to revision ".as_slice(), Some(HoleKind::Number)),
                Word::Gone => (b"Gone".as_slice(), None),
                Word::Connector => (b"Connector ".as_slice(), Some(HoleKind::Number)),
                Word::Resource => (b"Resource: ".as_slice(), Some(HoleKind::Name)),
                Word::Kind => (b"Kind ".as_slice(), Some(HoleKind::Number)),
                Word::Part => (b"Family part revision ".as_slice(), Some(HoleKind::Number)),
                Word::OutboxEntry => (b"Outbox entry ".as_slice(), Some(HoleKind::Number)),
                Word::NotAllowed => (b"Family object is not allowed".as_slice(), None),
                Word::SubjectEnded => (b"Family subject ended".as_slice(), None),
                Word::WatchBusy => (b"Family watch busy; retrying".as_slice(), None),
                Word::WatchStopping => (b"Engine stopping; retrying family watch".as_slice(), None),
            };
            let (holes, segments): (Box<[HoleShape]>, Box<[Segment]>) = match hole {
                Some(kind) => (
                    Box::new([HoleShape { kind, bytes: if kind == HoleKind::Name { text } else { 0 } }]),
                    Box::new([Segment::Text(copy_of(literal)), Segment::Hole { index: 0, kind }]),
                ),
                None => (Box::new([]), Box::new([Segment::Text(copy_of(literal))])),
            };
            shapes.push(PhraseShape { id: word.id(), holes, plural: None }).expect("closed shapes fit");
            phrases
                .push(Phrase { id: word.id(), forms: Box::new([Form { class: PluralClass::Other, segments }]) })
                .expect("closed English phrases fit");
        }
        let language = Language {
            plural: PluralRule::English,
            span_unit: SpanUnit::Seconds,
            wall: WallFormat::Seconds,
            phrases: phrases.into_boxed(),
        };
        let book = Phrasebook::load(shapes.into_boxed(), Box::new([language]), &phrase_limits(text))?;
        Ok(Self { book, language: 0 })
    }
    #[must_use]
    pub fn say(&self, word: Word, holes: &[Hole]) -> Box<[u8]> {
        self.book.say(self.language, word.id(), holes).expect("checked family phrase").text
    }
    pub fn text(&self, b: &mut Builder, word: Word, holes: &[Hole]) {
        b.text(Run::Plain, &self.say(word, holes));
    }
    pub fn status(&self, b: &mut Builder, word: Word, holes: &[Hole]) {
        let text = self.say(word, holes);
        b.open(Role::Status);
        b.name(&text);
        b.text(Run::Plain, &text);
        b.close();
    }
    #[must_use]
    pub fn capacity(&self) -> Option<u32> {
        let mut most = 0;
        for word in ALL {
            most = most.max(self.book.capacity(word.id())?);
        }
        Some(most)
    }
}
#[must_use]
pub const fn phrase_limits(text: u32) -> PhraseLimits {
    PhraseLimits { languages: 1, phrases: 29, holes: 1, segments: 58, text_bytes: 1024, hole_bytes: text }
}
