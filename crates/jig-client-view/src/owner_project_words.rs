//! Checked Project owner captions; client/domain/view.md, 7.
use crate::{
    Owner,
    phrases::{
        Form, HoleKind, HoleShape, Language, Phrase, PhraseId, PhraseShape, PluralClass, PluralRule, Segment, SpanUnit,
        WallFormat,
    },
};
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectPhrase {
    Project,
    Resource,
    Owned,
    Participant,
    Unavailable,
    Context,
    Policy,
    PolicyCut,
    ProjectCut,
    EscalationRole,
    EscalationAbsent,
    PeriodAbsent,
    Member,
    Person,
    Service,
    Login,
    Name,
    Spend,
    Spent,
    Below,
    Reserved,
    Budget,
    PermissionsAbsent,
    Edit,
    Confirm,
    Review,
    Dismiss,
}
pub const ALL: [ProjectPhrase; 27] = [
    ProjectPhrase::Project,
    ProjectPhrase::Resource,
    ProjectPhrase::Owned,
    ProjectPhrase::Participant,
    ProjectPhrase::Unavailable,
    ProjectPhrase::Context,
    ProjectPhrase::Policy,
    ProjectPhrase::PolicyCut,
    ProjectPhrase::ProjectCut,
    ProjectPhrase::EscalationRole,
    ProjectPhrase::EscalationAbsent,
    ProjectPhrase::PeriodAbsent,
    ProjectPhrase::Member,
    ProjectPhrase::Person,
    ProjectPhrase::Service,
    ProjectPhrase::Login,
    ProjectPhrase::Name,
    ProjectPhrase::Spend,
    ProjectPhrase::Spent,
    ProjectPhrase::Below,
    ProjectPhrase::Reserved,
    ProjectPhrase::Budget,
    ProjectPhrase::PermissionsAbsent,
    ProjectPhrase::Edit,
    ProjectPhrase::Confirm,
    ProjectPhrase::Review,
    ProjectPhrase::Dismiss,
];
impl ProjectPhrase {
    #[must_use]
    pub const fn id(self) -> PhraseId {
        PhraseId {
            owner: Owner::Jig,
            number: match self {
                Self::Project => 9000,
                Self::Resource => 9001,
                Self::Owned => 9002,
                Self::Participant => 9003,
                Self::Unavailable => 9004,
                Self::Context => 9005,
                Self::Policy => 9006,
                Self::PolicyCut => 9007,
                Self::ProjectCut => 9008,
                Self::EscalationRole => 9009,
                Self::EscalationAbsent => 9010,
                Self::PeriodAbsent => 9011,
                Self::Member => 9012,
                Self::Person => 9013,
                Self::Service => 9014,
                Self::Login => 9015,
                Self::Name => 9016,
                Self::Spend => 9017,
                Self::Spent => 9018,
                Self::Below => 9019,
                Self::Reserved => 9020,
                Self::Budget => 9021,
                Self::PermissionsAbsent => 9022,
                Self::Edit => 9023,
                Self::Confirm => 9024,
                Self::Review => 9025,
                Self::Dismiss => 9026,
            },
        }
    }
    fn hole(self) -> Option<HoleKind> {
        match self {
            Self::Project => Some(HoleKind::Number),
            Self::Resource => None,
            Self::Owned => None,
            Self::Participant => None,
            Self::Unavailable => None,
            Self::Context => None,
            Self::Policy => None,
            Self::PolicyCut => None,
            Self::ProjectCut => None,
            Self::EscalationRole => Some(HoleKind::Number),
            Self::EscalationAbsent => None,
            Self::PeriodAbsent => None,
            Self::Member => Some(HoleKind::Number),
            Self::Person => None,
            Self::Service => None,
            Self::Login => None,
            Self::Name => None,
            Self::Spend => None,
            Self::Spent => Some(HoleKind::Amount),
            Self::Below => Some(HoleKind::Amount),
            Self::Reserved => Some(HoleKind::Amount),
            Self::Budget => Some(HoleKind::Amount),
            Self::PermissionsAbsent => None,
            Self::Edit => None,
            Self::Confirm => None,
            Self::Review => None,
            Self::Dismiss => None,
        }
    }
    fn label(self) -> &'static [u8] {
        match self {
            Self::Project => b"Project",
            Self::Resource => b"Resource role",
            Self::Owned => b"Owned",
            Self::Participant => b"Participant",
            Self::Unavailable => b"Unavailable",
            Self::Context => b"Context",
            Self::Policy => b"Policy",
            Self::PolicyCut => b"Policy is incomplete",
            Self::ProjectCut => b"Project document is incomplete",
            Self::EscalationRole => b"Escalation role: ",
            Self::EscalationAbsent => b"Escalation role not supplied",
            Self::PeriodAbsent => b"Current period not supplied",
            Self::Member => b"Member: ",
            Self::Person => b"Person",
            Self::Service => b"Service",
            Self::Login => b"Login",
            Self::Name => b"Name",
            Self::Spend => b"Spend",
            Self::Spent => b"Spent: ",
            Self::Below => b"Spent below: ",
            Self::Reserved => b"Reserved: ",
            Self::Budget => b"Budget: ",
            Self::PermissionsAbsent => b"Permission mappings not supplied",
            Self::Edit => b"Review policy change",
            Self::Confirm => b"Confirm settings change",
            Self::Review => b"Before and after this settings change",
            Self::Dismiss => b"Dismiss settings change",
        }
    }
}
#[must_use]
pub fn shapes() -> Box<[PhraseShape]> {
    let mut values = List::with_capacity(COUNT);
    for phrase in ALL {
        let holes: Box<[HoleShape]> = match phrase.hole() {
            Some(kind) => Box::new([HoleShape { kind, bytes: 0 }]),
            None => Box::new([]),
        };
        values.push(PhraseShape { id: phrase.id(), holes, plural: None }).expect("project shapes fit");
    }
    values.into_boxed()
}
#[must_use]
pub fn language() -> Language {
    let mut values = List::with_capacity(COUNT);
    for phrase in ALL {
        let segments: Box<[Segment]> = match phrase.hole() {
            Some(kind) => Box::new([Segment::Text(copy_of(phrase.label())), Segment::Hole { index: 0, kind }]),
            None => Box::new([Segment::Text(copy_of(phrase.label()))]),
        };
        values
            .push(Phrase { id: phrase.id(), forms: Box::new([Form { class: PluralClass::Other, segments }]) })
            .expect("project words fit");
    }
    Language {
        plural: PluralRule::English,
        span_unit: SpanUnit::Nanos,
        wall: WallFormat::Nanos,
        phrases: values.into_boxed(),
    }
}
const COUNT: u32 = 27;
