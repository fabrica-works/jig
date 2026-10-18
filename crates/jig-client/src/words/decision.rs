//! Decision facts in the public phrasebook (client/domain/client.md, 19).
use alloc::boxed::Box;
use skein_lib::bytes::copy_of;
use skein_ui::phrases::{HoleKind, Segment};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecisionWord {
    Dangerous,
    ReleaseFresh,
    ResumeRelease,
    EngineFollowsChoice,
    TakeoverFunds,
    TasksWaiting,
    TasksRunning,
    TasksHeld,
    TasksClosing,
    TasksDone,
    TasksFailed,
    TasksCancelled,
    LiveRuns,
    PendingProposals,
    TasksEnded,
    EffectsSettle,
    Funds,
    Needed,
    Left,
    LeftAbsent,
    AfterFunding,
    AfterAbsent,
    FunderTask,
    FunderPool,
    FunderRecurring,
    FunderPeriod,
}
impl DecisionWord {
    pub(super) const fn data(self) -> (u16, &'static [u8], &'static [HoleKind]) {
        match self {
            Self::Dangerous => (59, b"", &[]),
            Self::ReleaseFresh => (60, b"", &[]),
            Self::ResumeRelease => (61, b"", &[]),
            Self::EngineFollowsChoice => (62, b"", &[]),
            Self::TakeoverFunds => (63, b"", &[HoleKind::Amount]),
            Self::TasksWaiting => (64, b"", &[HoleKind::Number]),
            Self::TasksRunning => (65, b"", &[HoleKind::Number]),
            Self::TasksHeld => (66, b"", &[HoleKind::Number]),
            Self::TasksClosing => (67, b"", &[HoleKind::Number]),
            Self::TasksDone => (68, b"", &[HoleKind::Number]),
            Self::TasksFailed => (69, b"", &[HoleKind::Number]),
            Self::TasksCancelled => (70, b"", &[HoleKind::Number]),
            Self::LiveRuns => (71, b"", &[HoleKind::Number]),
            Self::PendingProposals => (72, b"", &[HoleKind::Number]),
            Self::TasksEnded => (73, b"", &[HoleKind::Number]),
            Self::EffectsSettle => (74, b"", &[]),
            Self::Funds => (75, b"", &[HoleKind::Amount, HoleKind::Amount, HoleKind::Amount]),
            Self::Needed => (76, b"", &[HoleKind::Number]),
            Self::Left => (77, b"", &[HoleKind::Number]),
            Self::LeftAbsent => (78, b"", &[]),
            Self::AfterFunding => (79, b"", &[HoleKind::Number]),
            Self::AfterAbsent => (80, b"", &[]),
            Self::FunderTask => (81, b"", &[HoleKind::Number]),
            Self::FunderPool => (82, b"", &[HoleKind::Number, HoleKind::Number, HoleKind::Number]),
            Self::FunderRecurring => (83, b"", &[HoleKind::Number, HoleKind::Number, HoleKind::Number]),
            Self::FunderPeriod => (84, b"", &[HoleKind::Number, HoleKind::Number]),
        }
    }
    pub(super) fn segments(self) -> Box<[Segment]> {
        match self {
            Self::Dangerous => Box::new([Segment::Text(copy_of(b"Dangerous action: review its consequences"))]),
            Self::ReleaseFresh => Box::new([Segment::Text(copy_of(b"Release for the executor to decide afresh"))]),
            Self::ResumeRelease => Box::new([Segment::Text(copy_of(
                b"The next run resumes its conversation; tries count again from zero",
            ))]),
            Self::EngineFollowsChoice => Box::new([Segment::Text(copy_of(
                b"Inspect the connector's system facts; the engine will do as you choose",
            ))]),
            Self::TakeoverFunds => Box::new([
                Segment::Text(copy_of(b"Takeover reserves ")),
                Segment::Hole { index: 0, kind: HoleKind::Amount },
                Segment::Text(copy_of(b" from your pool")),
            ]),
            Self::TasksWaiting => Box::new([
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" waiting tasks")),
            ]),
            Self::TasksRunning => Box::new([
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" running tasks")),
            ]),
            Self::TasksHeld => {
                Box::new([Segment::Hole { index: 0, kind: HoleKind::Number }, Segment::Text(copy_of(b" held tasks"))])
            }
            Self::TasksClosing => Box::new([
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" closing tasks")),
            ]),
            Self::TasksDone => Box::new([
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" completed tasks")),
            ]),
            Self::TasksFailed => {
                Box::new([Segment::Hole { index: 0, kind: HoleKind::Number }, Segment::Text(copy_of(b" failed tasks"))])
            }
            Self::TasksCancelled => Box::new([
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" cancelled tasks")),
            ]),
            Self::LiveRuns => {
                Box::new([Segment::Hole { index: 0, kind: HoleKind::Number }, Segment::Text(copy_of(b" live runs"))])
            }
            Self::PendingProposals => Box::new([
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" proposals will be withdrawn")),
            ]),
            Self::TasksEnded => Box::new([
                Segment::Text(copy_of(b"Ended tasks with outcome not supplied: ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            Self::EffectsSettle => Box::new([Segment::Text(copy_of(
                b"Effects not yet begun will be withdrawn; begun effects will settle",
            ))]),
            Self::Funds => funds(),
            Self::Needed => Box::new([
                Segment::Text(copy_of(b"Funding needed: ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            Self::Left => Box::new([
                Segment::Text(copy_of(b"Funding left: ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            Self::LeftAbsent => Box::new([Segment::Text(copy_of(b"Funding left not supplied"))]),
            Self::AfterFunding => Box::new([
                Segment::Text(copy_of(b"Funding after: ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            Self::AfterAbsent => Box::new([Segment::Text(copy_of(b"Funding after not supplied"))]),
            Self::FunderTask => Box::new([
                Segment::Text(copy_of(b"Funded by task ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            Self::FunderPool => Box::new([
                Segment::Text(copy_of(b"Funded by pool in project ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b", person ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
                Segment::Text(copy_of(b", period ")),
                Segment::Hole { index: 2, kind: HoleKind::Number },
            ]),
            Self::FunderRecurring => Box::new([
                Segment::Text(copy_of(b"Funded by recurring task in project ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b", task ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
                Segment::Text(copy_of(b", period ")),
                Segment::Hole { index: 2, kind: HoleKind::Number },
            ]),
            Self::FunderPeriod => Box::new([
                Segment::Text(copy_of(b"Funded by project period ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b", period ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
            ]),
        }
    }
}

fn funds() -> Box<[Segment]> {
    Box::new([
        Segment::Text(copy_of(b"Funds ")),
        Segment::Hole { index: 0, kind: HoleKind::Amount },
        Segment::Text(copy_of(b"; ")),
        Segment::Hole { index: 1, kind: HoleKind::Amount },
        Segment::Text(copy_of(b" left now, ")),
        Segment::Hole { index: 2, kind: HoleKind::Amount },
        Segment::Text(copy_of(b" after")),
    ])
}
