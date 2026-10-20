//! The English fallback for the public enum phrases.
use super::{ALL, HoldWord, RefusalWord, StatusWord, WaitingWord, Word};
use crate::contract::Phase;
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
use skein_ui::phrases::HoleKind;
use skein_ui::phrases::{Form, Language, Phrase, PluralClass, PluralRule, Segment, SpanUnit, WallFormat};
pub(super) fn language() -> Language {
    let mut phrases = List::with_capacity(u32::try_from(ALL.len()).expect("closed word count"));
    for word in ALL {
        let segments = match word {
            Word::Decision(value) => value.segments(),
            Word::Phase(_) | Word::Hold(_) | Word::Refusal(_) | Word::Waiting(_) | Word::Status(_) => {
                let (_, label, holes) = word.data();
                let mut segments = List::with_capacity(2);
                segments.push(Segment::Text(copy_of(label))).expect("word label fits");
                for kind in holes {
                    segments.push(Segment::Hole { index: 0, kind: *kind }).expect("word hole fits");
                }
                segments.into_boxed()
            }
        };
        phrases
            .push(Phrase { id: word.id(), forms: Box::new([Form { class: PluralClass::Other, segments }]) })
            .expect("all word phrases fit");
    }
    Language {
        plural: PluralRule::English,
        span_unit: SpanUnit::Seconds,
        wall: WallFormat::Clock,
        phrases: phrases.into_boxed(),
    }
}

impl Word {
    pub(super) const fn data(self) -> (u16, &'static [u8], &'static [HoleKind]) {
        match self {
            Self::Decision(value) => value.data(),
            Self::Phase(Phase::Waiting) => (0, b"Waiting", &[]),
            Self::Phase(Phase::Idle) => (1, b"Waiting for news", &[]),
            Self::Phase(Phase::Due) => (2, b"Due to run", &[]),
            Self::Phase(Phase::Preparing) => (3, b"Preparing a host", &[]),
            Self::Phase(Phase::Claimed) => (4, b"Claimed by a host", &[]),
            Self::Phase(Phase::Running) => (5, b"Running", &[]),
            Self::Phase(Phase::BackingOff) => (6, b"Retrying", &[]),
            Self::Phase(Phase::Active) => (7, b"Executor active", &[]),
            Self::Phase(Phase::Closing) => (8, b"Closing", &[]),
            Self::Phase(Phase::Held) => (9, b"Held", &[]),
            Self::Phase(Phase::Ended) => (10, b"Ended", &[]),
            Self::Phase(Phase::Done) => (11, b"Completed", &[]),
            Self::Phase(Phase::Failed) => (12, b"Failed", &[]),
            Self::Phase(Phase::Cancelled) => (13, b"Cancelled", &[]),
            Self::Hold(HoldWord::Failures) => (14, b"Held: out of tries", &[]),
            Self::Hold(HoldWord::Stalled) => (15, b"The executor stopped progressing", &[]),
            Self::Hold(HoldWord::Dependency) => (16, b"Dependency task ", &[HoleKind::Number]),
            Self::Hold(HoldWord::Stopped) => (17, b"Task stopped", &[]),
            Self::Hold(HoldWord::StoppedBy) => (18, b"Task stopped", &[]),
            Self::Hold(HoldWord::Drift) => (19, b"Required resource drifted", &[]),
            Self::Hold(HoldWord::Effects) => (20, b"Effects need a decision", &[]),
            Self::Hold(HoldWord::EffectFailed) => (21, b"An effect failed permanently", &[]),
            Self::Hold(HoldWord::Uncertain) => (22, b"Uncertain outbox entry ", &[HoleKind::Number]),
            Self::Hold(HoldWord::Budget) => (23, b"Insufficient activation budget", &[]),
            Self::Hold(HoldWord::Deadline) => (24, b"Deadline passed", &[]),
            Self::Hold(HoldWord::HoldsWaited) => (25, b"Resource holds waited too long", &[]),
            Self::Hold(HoldWord::ResourceUnavailable) => (26, b"A required connector resource is unavailable", &[]),
            Self::Hold(HoldWord::ResourceAccess) => (27, b"Current resource access forbids this action", &[]),
            Self::Hold(HoldWord::PoolLost) => (28, b"Allocation lost from pool ", &[HoleKind::Number]),
            Self::Hold(HoldWord::Procedure) => (29, b"Procedure needs a decision", &[]),
            Self::Refusal(RefusalWord::NoEscalationHolder) => (30, b"No one to escalate to", &[]),
            Self::Refusal(RefusalWord::Malformed) => (31, b"Malformed operation", &[]),
            Self::Refusal(RefusalWord::Role) => (32, b"The held role does not permit this action", &[]),
            Self::Refusal(RefusalWord::Taken) => (34, b"Held by party ", &[HoleKind::Number]),
            Self::Refusal(RefusalWord::NotWaiting) => (35, b"This object waits for someone else", &[]),
            Self::Refusal(RefusalWord::NoFurther) => (36, b"This holder cannot pass further", &[]),
            Self::Refusal(RefusalWord::Authority) => (37, b"Authority missing", &[]),
            Self::Refusal(RefusalWord::Limit) => (38, b"Limit reached", &[]),
            Self::Refusal(RefusalWord::Moved) => (39, b"The object changed", &[]),
            Self::Refusal(RefusalWord::Ended) => (40, b"The object ended", &[]),
            Self::Refusal(RefusalWord::Unknown) => (41, b"No visible object with that name", &[]),
            Self::Refusal(RefusalWord::NotOffered) => (42, b"This act is not offered", &[]),
            Self::Refusal(RefusalWord::KeyConflict) => (43, b"The request key names another act", &[]),
            Self::Refusal(RefusalWord::Llm) => (85, b"This model cannot be chosen: ", &[HoleKind::Name]),
            Self::Waiting(WaitingWord::Party) => (44, b"Waiting for a party", &[]),
            Self::Waiting(WaitingWord::Delegates) => (45, b"Waiting for delegates", &[]),
            Self::Waiting(WaitingWord::News) => (46, b"Waiting for news", &[]),
            Self::Waiting(WaitingWord::Nothing) => (47, b"Nothing awaited", &[]),
            Self::Status(StatusWord::Starting) => (48, b"Starting", &[]),
            Self::Status(StatusWord::Writing) => (49, b"Writing", &[]),
            Self::Status(StatusWord::Tool) => (50, b"Using tool ", &[HoleKind::Name]),
            Self::Status(StatusWord::Working) => (51, b"Working", &[]),
            Self::Status(StatusWord::WaitingForYou) => (52, b"Waiting for your words", &[]),
            Self::Status(StatusWord::WaitingFor) => (53, b"Waiting for another party", &[]),
            Self::Status(StatusWord::Parked) => (54, b"Parked; your words resume it", &[]),
            Self::Status(StatusWord::Retrying) => (55, b"Retrying", &[]),
            Self::Status(StatusWord::Held) => (56, b"Held", &[]),
            Self::Status(StatusWord::Closing) => (57, b"Closing", &[]),
            Self::Status(StatusWord::Ended) => (58, b"Ended", &[]),
        }
    }
}
