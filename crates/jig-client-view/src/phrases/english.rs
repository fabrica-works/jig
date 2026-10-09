//! English is checked data (client/domain/view.md, section 7).
use super::jig::{ALL, JigPhrase, shapes};
use super::{
    Form, HoleKind, Invalid, Language, Phrase, PhraseLimits, Phrasebook, PluralClass, PluralRule, Segment, SpanUnit,
    WallFormat,
};
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
/// English words for each phrase, checked with any other loaded language.
#[must_use]
#[expect(clippy::too_many_lines, reason = "English data matches the closed phrase vocabulary whole")]
pub fn language() -> Language {
    let mut phrases = List::with_capacity(100);
    for phrase in ALL {
        let segments: Box<[Segment]> = match phrase {
            JigPhrase::LoadingNavigation => Box::new([Segment::Text(copy_of(b"Loading navigation"))]),
            JigPhrase::ByNamed => {
                Box::new([Segment::Text(copy_of(b"By ")), Segment::Hole { index: 0, kind: HoleKind::Name }])
            }
            JigPhrase::Refused => Box::new([Segment::Text(copy_of(b"Refused"))]),
            JigPhrase::Leaving => Box::new([Segment::Text(copy_of(b"Leaving"))]),
            JigPhrase::Live => Box::new([Segment::Text(copy_of(b"Live"))]),
            JigPhrase::Offline => Box::new([Segment::Text(copy_of(b"Offline"))]),
            JigPhrase::Starting => Box::new([Segment::Text(copy_of(b"Connecting"))]),
            JigPhrase::Behind => Box::new([Segment::Text(copy_of(b"Behind: waiting for current facts"))]),
            JigPhrase::Needs => {
                Box::new([Segment::Hole { index: 0, kind: HoleKind::Number }, Segment::Text(copy_of(b" need you"))])
            }
            JigPhrase::MoreNeeds => Box::new([
                Segment::Text(copy_of(b"More than ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" need you")),
            ]),
            JigPhrase::Updates => Box::new([
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" updates not read")),
            ]),
            JigPhrase::MoreUpdates => Box::new([
                Segment::Text(copy_of(b"More than ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" updates not read")),
            ]),
            JigPhrase::SignedIn => {
                Box::new([Segment::Text(copy_of(b"Signed in as ")), Segment::Hole { index: 0, kind: HoleKind::Name }])
            }
            JigPhrase::SignedOut => Box::new([Segment::Text(copy_of(b"Signed out"))]),
            JigPhrase::Frame => Box::new([Segment::Text(copy_of(b"Work"))]),
            JigPhrase::Navigation => Box::new([Segment::Text(copy_of(b"Navigation"))]),
            JigPhrase::Project => {
                Box::new([Segment::Text(copy_of(b"Project: ")), Segment::Hole { index: 0, kind: HoleKind::Name }])
            }
            JigPhrase::AllProjects => Box::new([Segment::Text(copy_of(b"All projects"))]),
            JigPhrase::TaskNumber => Box::new([Segment::Text(copy_of(b"Go to task number"))]),
            JigPhrase::Inbox => Box::new([Segment::Text(copy_of(b"Inbox"))]),
            JigPhrase::Conversations => Box::new([Segment::Text(copy_of(b"Conversations"))]),
            JigPhrase::ProjectPage => Box::new([Segment::Text(copy_of(b"Project"))]),
            JigPhrase::Deployment => Box::new([Segment::Text(copy_of(b"Deployment"))]),
            JigPhrase::TaskPage | JigPhrase::ContextTask => {
                Box::new([Segment::Text(copy_of(b"Task ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            JigPhrase::Open => Box::new([Segment::Text(copy_of(b"Open"))]),
            JigPhrase::Confirming => Box::new([Segment::Text(copy_of(b"Being decided: review the confirmation"))]),
            JigPhrase::InFlight => Box::new([Segment::Text(copy_of(b"Being decided: waiting for the engine"))]),
            JigPhrase::Retrying => Box::new([Segment::Text(copy_of(b"Being decided: retrying"))]),
            JigPhrase::Parked => {
                Box::new([Segment::Text(copy_of(b"Being decided: waiting for the engine connection or your sign-in"))])
            }
            JigPhrase::Unknown => {
                Box::new([Segment::Text(copy_of(b"Outcome unknown: inspect the current object before acting again"))])
            }
            JigPhrase::LookAgain => Box::new([Segment::Text(copy_of(b"Inspect current object"))]),
            JigPhrase::Looking => Box::new([Segment::Text(copy_of(b"Finding how this ended"))]),
            JigPhrase::GoneUnknown => {
                Box::new([Segment::Text(copy_of(b"Ended; its final facts are no longer available"))])
            }
            JigPhrase::Changed => Box::new([Segment::Text(copy_of(b"Changed"))]),
            JigPhrase::Proposed => Box::new([
                Segment::Text(copy_of(b"Proposed by task ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b", proposal ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
            ]),
            JigPhrase::Done => Box::new([Segment::Text(copy_of(b"Completed"))]),
            JigPhrase::Failed => {
                Box::new([Segment::Text(copy_of(b"Failed: ")), Segment::Hole { index: 0, kind: HoleKind::Name }])
            }
            JigPhrase::Cancelled => {
                Box::new([Segment::Text(copy_of(b"Cancelled: ")), Segment::Hole { index: 0, kind: HoleKind::Name }])
            }
            JigPhrase::Withdrawn => {
                Box::new([Segment::Text(copy_of(b"Withdrawn: ")), Segment::Hole { index: 0, kind: HoleKind::Name }])
            }
            JigPhrase::ByParty => {
                Box::new([Segment::Text(copy_of(b"By party ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            JigPhrase::ByTask => {
                Box::new([Segment::Text(copy_of(b"By task ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            JigPhrase::ByDeployment => Box::new([Segment::Text(copy_of(b"By the deployment"))]),
            JigPhrase::At => {
                Box::new([Segment::Text(copy_of(b"At ")), Segment::Hole { index: 0, kind: HoleKind::Wall }])
            }
            JigPhrase::Accepted => Box::new([Segment::Text(copy_of(b"Accepted"))]),
            JigPhrase::Rejected => {
                Box::new([Segment::Text(copy_of(b"Rejected: ")), Segment::Hole { index: 0, kind: HoleKind::Name }])
            }
            JigPhrase::PassedUp => Box::new([Segment::Text(copy_of(b"Passed up"))]),
            JigPhrase::Released => Box::new([Segment::Text(copy_of(b"Released"))]),
            JigPhrase::ReleasedAgain => Box::new([Segment::Text(copy_of(b"Released: make the effect again"))]),
            JigPhrase::ReleasedWithout => Box::new([Segment::Text(copy_of(b"Released: do not make the effect again"))]),
            JigPhrase::LeftHeld => {
                Box::new([Segment::Text(copy_of(b"Left held: ")), Segment::Hole { index: 0, kind: HoleKind::Name }])
            }
            JigPhrase::DecidedFirst => Box::new([
                Segment::Text(copy_of(b"Decided first by ")),
                Segment::Hole { index: 0, kind: HoleKind::Name },
            ]),
            JigPhrase::MadeChat => {
                Box::new([Segment::Text(copy_of(b"Made chat ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            JigPhrase::MadeWords => Box::new([
                Segment::Text(copy_of(b"Sent words to task ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b", message ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
            ]),
            JigPhrase::MadeGoal => {
                Box::new([Segment::Text(copy_of(b"Made goal ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            JigPhrase::MadeTask => {
                Box::new([Segment::Text(copy_of(b"Made task ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            JigPhrase::MadeService => Box::new([
                Segment::Text(copy_of(b"Made service party ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            JigPhrase::MadeResult => Box::new([
                Segment::Text(copy_of(b"Answered person task ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            JigPhrase::MadeNote => {
                Box::new([Segment::Text(copy_of(b"Made note ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            JigPhrase::RoleMissing => Box::new([
                Segment::Text(copy_of(b"Project ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b": requires role ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
            ]),
            JigPhrase::BudgetMissing => Box::new([
                Segment::Text(copy_of(b"Your pool has ")),
                Segment::Hole { index: 0, kind: HoleKind::Amount },
                Segment::Text(copy_of(b" left; this needs ")),
                Segment::Hole { index: 1, kind: HoleKind::Amount },
            ]),
            JigPhrase::GrantMissing => Box::new([
                Segment::Text(copy_of(b"Requires effect kind ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" on connector ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
                Segment::Text(copy_of(b", resource ")),
                Segment::Hole { index: 2, kind: HoleKind::Name },
            ]),
            JigPhrase::DepthMissing => Box::new([
                Segment::Text(copy_of(b"Delegation needs depth ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b"; allowed ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
            ]),
            JigPhrase::DeadlineMissing => Box::new([
                Segment::Text(copy_of(b"Deadline ")),
                Segment::Hole { index: 0, kind: HoleKind::Wall },
                Segment::Text(copy_of(b" exceeds latest allowed ")),
                Segment::Hole { index: 1, kind: HoleKind::Wall },
            ]),
            JigPhrase::TasksMissing => Box::new([
                Segment::Text(copy_of(b"Needs ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" tasks; ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
                Segment::Text(copy_of(b" left")),
            ]),
            JigPhrase::Agent => Box::new([Segment::Text(copy_of(b"Agent"))]),
            JigPhrase::Procedure => Box::new([Segment::Text(copy_of(b"Procedure"))]),
            JigPhrase::Person => Box::new([Segment::Text(copy_of(b"Person"))]),
            JigPhrase::RoleExecutor => Box::new([Segment::Text(copy_of(b"Person holding a role"))]),
            JigPhrase::KindMissing => Box::new([Segment::Text(copy_of(b"Executor kind not allowed"))]),
            JigPhrase::LimitLive => Box::new([
                Segment::Text(copy_of(b"Project ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b": live task limit ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
            ]),
            JigPhrase::LimitInbox => Box::new([
                Segment::Text(copy_of(b"Task ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b": inbox limit ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
            ]),
            JigPhrase::LimitWords => Box::new([
                Segment::Text(copy_of(b"Words exceed ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" bytes")),
            ]),
            JigPhrase::LimitConversation => Box::new([
                Segment::Text(copy_of(b"Conversation word snapshot limit ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            JigPhrase::LimitFields => Box::new([Segment::Text(copy_of(b"Field limit reached"))]),
            JigPhrase::LimitRecords => Box::new([Segment::Text(copy_of(b"Kept record limit reached"))]),
            JigPhrase::LimitRequests => Box::new([Segment::Text(copy_of(b"Request limit reached"))]),
            JigPhrase::LimitReads => Box::new([Segment::Text(copy_of(b"Read limit reached"))]),
            JigPhrase::LimitWatches => Box::new([Segment::Text(copy_of(b"Watch limit reached"))]),
            JigPhrase::LimitObjects => Box::new([Segment::Text(copy_of(b"Working set limit reached"))]),
            JigPhrase::MovedHold => Box::new([Segment::Text(copy_of(b"The hold or its reason changed"))]),
            JigPhrase::MovedSpec => Box::new([Segment::Text(copy_of(b"The specification changed"))]),
            JigPhrase::MovedPerson => Box::new([Segment::Text(copy_of(b"The question or its options changed"))]),
            JigPhrase::MovedNote => Box::new([Segment::Text(copy_of(b"The note changed"))]),
            JigPhrase::MovedWaiting => Box::new([Segment::Text(copy_of(b"The recipient changed"))]),
            JigPhrase::Revision => Box::new([
                Segment::Text(copy_of(b"Current revision ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            JigPhrase::NoObject => Box::new([Segment::Text(copy_of(b"No visible object with that name"))]),
            JigPhrase::NotOffered => Box::new([Segment::Text(copy_of(b"This act is not offered"))]),
            JigPhrase::KeyConflict => Box::new([Segment::Text(copy_of(b"The request key names another act"))]),
            JigPhrase::Amount => Box::new([Segment::Hole { index: 0, kind: HoleKind::Amount }]),
            JigPhrase::WaitingSpan => Box::new([
                Segment::Text(copy_of(b"Waiting ")),
                Segment::Hole { index: 0, kind: HoleKind::Span },
                Segment::Text(copy_of(b" min")),
            ]),
            JigPhrase::Span => {
                Box::new([Segment::Hole { index: 0, kind: HoleKind::Span }, Segment::Text(copy_of(b" min"))])
            }
            JigPhrase::Object => Box::new([Segment::Text(copy_of(b"Object"))]),
            JigPhrase::Proposal => Box::new([Segment::Text(copy_of(b"Proposal"))]),
            JigPhrase::Escalation => Box::new([Segment::Text(copy_of(b"Escalation"))]),
            JigPhrase::Question => Box::new([Segment::Text(copy_of(b"Question"))]),
            JigPhrase::PersonTask => Box::new([Segment::Text(copy_of(b"Person task"))]),
            JigPhrase::Result => Box::new([Segment::Text(copy_of(b"Result"))]),
            JigPhrase::Note => Box::new([Segment::Text(copy_of(b"Note"))]),
            JigPhrase::ContextProject => {
                Box::new([Segment::Text(copy_of(b"In project ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
        };
        phrases
            .push(Phrase { id: phrase.id(), forms: Box::new([Form { class: PluralClass::Other, segments }]) })
            .expect("all English phrases fit");
    }
    Language {
        plural: PluralRule::English,
        span_unit: SpanUnit::Minutes,
        wall: WallFormat::Clock,
        phrases: phrases.into_boxed(),
    }
}
/// A ready English phrasebook whose names fit the client's text limit.
pub fn english(text: u32) -> Result<Phrasebook, Invalid> {
    Phrasebook::load(
        shapes(text),
        Box::new([language()]),
        &PhraseLimits { languages: 1, phrases: 100, holes: 3, segments: 1000, text_bytes: 10000, hole_bytes: text },
    )
}
