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
    let mut phrases = List::with_capacity(203);
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
            JigPhrase::ChipNumber => Box::new([
                Segment::Text(copy_of(b"Task ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b": ")),
                Segment::Hole { index: 1, kind: HoleKind::Name },
            ]),
            JigPhrase::Waiting => Box::new([Segment::Text(copy_of(b"Waiting"))]),
            JigPhrase::Idle => Box::new([Segment::Text(copy_of(b"Waiting for news"))]),
            JigPhrase::WakeAny => Box::new([Segment::Text(copy_of(b"Wakes on any news"))]),
            JigPhrase::WakeAt => Box::new([Segment::Text(copy_of(b"Wakes at the next scheduled time"))]),
            JigPhrase::WakeResult => Box::new([Segment::Text(copy_of(b"Wakes on a delegate result"))]),
            JigPhrase::WakeTopic => Box::new([Segment::Text(copy_of(b"Wakes on subscribed news"))]),
            JigPhrase::WakeInbox => Box::new([Segment::Text(copy_of(b"Wakes on inbox messages"))]),
            JigPhrase::Running => Box::new([Segment::Text(copy_of(b"Running"))]),
            JigPhrase::Writing => Box::new([Segment::Text(copy_of(b"Writing"))]),
            JigPhrase::Tool => {
                Box::new([Segment::Text(copy_of(b"Using tool ")), Segment::Hole { index: 0, kind: HoleKind::Name }])
            }
            JigPhrase::Attempt => {
                Box::new([Segment::Text(copy_of(b"Attempt ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            JigPhrase::RetryAt => Box::new([
                Segment::Text(copy_of(b"Retrying at ")),
                Segment::Hole { index: 0, kind: HoleKind::Wall },
                Segment::Text(copy_of(b": try ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
                Segment::Text(copy_of(b" of ")),
                Segment::Hole { index: 2, kind: HoleKind::Number },
            ]),
            JigPhrase::WaitingWords => Box::new([Segment::Text(copy_of(b"Waiting for your words"))]),
            JigPhrase::HostLost => Box::new([Segment::Text(copy_of(b"Host lost"))]),
            JigPhrase::Temporary => Box::new([Segment::Text(copy_of(b"Temporary failure"))]),
            JigPhrase::Permanent => Box::new([Segment::Text(copy_of(b"Permanent failure"))]),
            JigPhrase::Stall => Box::new([Segment::Text(copy_of(b"Stalled"))]),
            JigPhrase::Reason => {
                Box::new([Segment::Text(copy_of(b"Reason: ")), Segment::Hole { index: 0, kind: HoleKind::Name }])
            }
            JigPhrase::Closing => Box::new([
                Segment::Text(copy_of(b"Closing: ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" delegates and ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
                Segment::Text(copy_of(b" effects settling")),
            ]),
            JigPhrase::Held => Box::new([Segment::Text(copy_of(b"Held"))]),
            JigPhrase::Ended => Box::new([Segment::Text(copy_of(b"Ended"))]),
            JigPhrase::WaitingParty => Box::new([
                Segment::Text(copy_of(b"Waiting for party ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            JigPhrase::WaitingTask => Box::new([
                Segment::Text(copy_of(b"Waiting for task ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            JigPhrase::WaitingRole => Box::new([
                Segment::Text(copy_of(b"Waiting for project ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b", role ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
            ]),
            JigPhrase::Since => {
                Box::new([Segment::Text(copy_of(b"Since ")), Segment::Hole { index: 0, kind: HoleKind::Wall }])
            }
            JigPhrase::Funds => Box::new([
                Segment::Text(copy_of(b"Funds ")),
                Segment::Hole { index: 0, kind: HoleKind::Amount },
                Segment::Text(copy_of(b"; ")),
                Segment::Hole { index: 1, kind: HoleKind::Amount },
                Segment::Text(copy_of(b" left now, ")),
                Segment::Hole { index: 2, kind: HoleKind::Amount },
                Segment::Text(copy_of(b" after")),
            ]),
            JigPhrase::Pool => Box::new([Segment::Text(copy_of(b"From this pool"))]),
            JigPhrase::ProposalWhy => Box::new([Segment::Text(copy_of(b"Why this is proposed"))]),
            JigPhrase::OwnTasks => Box::new([Segment::Text(copy_of(b"The accepted tasks will be yours"))]),
            JigPhrase::Batch => Box::new([Segment::Text(copy_of(b"Tasks acceptance would make"))]),
            JigPhrase::BatchTask => Box::new([
                Segment::Text(copy_of(b"Proposed task ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            JigPhrase::AfterTask => Box::new([
                Segment::Text(copy_of(b"After proposed task ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            JigPhrase::Budget => {
                Box::new([Segment::Text(copy_of(b"Budget ")), Segment::Hole { index: 0, kind: HoleKind::Amount }])
            }
            JigPhrase::Before => Box::new([Segment::Text(copy_of(b"Before"))]),
            JigPhrase::After => Box::new([Segment::Text(copy_of(b"After"))]),
            JigPhrase::Depth => Box::new([
                Segment::Text(copy_of(b"Delegation depth ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            JigPhrase::TasksAllowed => Box::new([
                Segment::Text(copy_of(b"Tasks allowed ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            JigPhrase::Grant => Box::new([
                Segment::Text(copy_of(b"Effect kind ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" on connector ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
                Segment::Text(copy_of(b", resource ")),
                Segment::Hole { index: 2, kind: HoleKind::Name },
            ]),
            JigPhrase::ToolAllowed => Box::new([
                Segment::Text(copy_of(b"Tool ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" allowed")),
            ]),
            JigPhrase::PassedBy => Box::new([Segment::Text(copy_of(b"Passed by this holder"))]),
            JigPhrase::Widen => Box::new([Segment::Text(copy_of(b"Widen authority"))]),
            JigPhrase::AmendTask => {
                Box::new([Segment::Text(copy_of(b"Amend task ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            JigPhrase::Effect => Box::new([Segment::Text(copy_of(b"Effect"))]),
            JigPhrase::AnswerField => Box::new([Segment::Text(copy_of(b"Your answer"))]),
            JigPhrase::ApprovalState => Box::new([
                Segment::Text(copy_of(b"State to approve: ")),
                Segment::Hole { index: 0, kind: HoleKind::Name },
            ]),
            JigPhrase::Options => Box::new([Segment::Text(copy_of(b"Choose an option"))]),
            JigPhrase::CannotDo => Box::new([Segment::Text(copy_of(b"Cannot do it"))]),
            JigPhrase::TakenBy => {
                Box::new([Segment::Text(copy_of(b"Taken by ")), Segment::Hole { index: 0, kind: HoleKind::Name }])
            }
            JigPhrase::StallSpan => Box::new([
                Segment::Text(copy_of(b"Becomes news for its requester after ")),
                Segment::Hole { index: 0, kind: HoleKind::Span },
                Segment::Text(copy_of(b" min")),
            ]),
            JigPhrase::Verdict => {
                Box::new([Segment::Text(copy_of(b"Verdict ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            JigPhrase::ChoiceResult => Box::new([
                Segment::Text(copy_of(b"Choice ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b", by ")),
                Segment::Hole { index: 1, kind: HoleKind::Name },
            ]),
            JigPhrase::ApprovalYes => Box::new([
                Segment::Text(copy_of(b"Approved ")),
                Segment::Hole { index: 0, kind: HoleKind::Name },
                Segment::Text(copy_of(b", by ")),
                Segment::Hole { index: 1, kind: HoleKind::Name },
            ]),
            JigPhrase::ApprovalNo => Box::new([
                Segment::Text(copy_of(b"Did not approve ")),
                Segment::Hole { index: 0, kind: HoleKind::Name },
                Segment::Text(copy_of(b", by ")),
                Segment::Hole { index: 1, kind: HoleKind::Name },
            ]),
            JigPhrase::FirstUnread => Box::new([
                Segment::Text(copy_of(b"Open at first reply not read: ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            JigPhrase::Reply => Box::new([Segment::Text(copy_of(b"Reply"))]),
            JigPhrase::LoadingPart => Box::new([Segment::Text(copy_of(b"Loading connector content"))]),
            JigPhrase::LoadingObject => Box::new([Segment::Text(copy_of(b"Loading object"))]),
            JigPhrase::ActWrite => Box::new([Segment::Text(copy_of(b"Write"))]),
            JigPhrase::ActAnswer => Box::new([Segment::Text(copy_of(b"Answer"))]),
            JigPhrase::ActTake => Box::new([Segment::Text(copy_of(b"Take it"))]),
            JigPhrase::ActHandBack => Box::new([Segment::Text(copy_of(b"Hand it back"))]),
            JigPhrase::ActAccept => Box::new([Segment::Text(copy_of(b"Accept"))]),
            JigPhrase::ActReject => Box::new([Segment::Text(copy_of(b"Reject"))]),
            JigPhrase::ActPassUp => Box::new([Segment::Text(copy_of(b"Pass up"))]),
            JigPhrase::ActRelease => Box::new([Segment::Text(copy_of(b"Release"))]),
            JigPhrase::ActLeaveHeld => Box::new([Segment::Text(copy_of(b"Leave it held"))]),
            JigPhrase::ActAmend => Box::new([Segment::Text(copy_of(b"Amend"))]),
            JigPhrase::ActAmendRelease => Box::new([Segment::Text(copy_of(b"Amend and release"))]),
            JigPhrase::ActStop => Box::new([Segment::Text(copy_of(b"Stop"))]),
            JigPhrase::ActCancel => Box::new([Segment::Text(copy_of(b"Cancel"))]),
            JigPhrase::ActClose => Box::new([Segment::Text(copy_of(b"Close"))]),
            JigPhrase::ActTakeOver => Box::new([Segment::Text(copy_of(b"Take over"))]),
            JigPhrase::ActSetGoal => Box::new([Segment::Text(copy_of(b"Set goal"))]),
            JigPhrase::ActPrioritise => Box::new([Segment::Text(copy_of(b"Prioritise"))]),
            JigPhrase::ActMarkSeen => Box::new([Segment::Text(copy_of(b"Mark read"))]),
            JigPhrase::ActEditNote => Box::new([Segment::Text(copy_of(b"Edit note"))]),
            JigPhrase::ActDeleteNote => Box::new([Segment::Text(copy_of(b"Delete note"))]),
            JigPhrase::ActAdopt => Box::new([Segment::Text(copy_of(b"Adopt resource"))]),
            JigPhrase::ActChangePolicy => Box::new([Segment::Text(copy_of(b"Change policy"))]),
            JigPhrase::ActSetRoles => Box::new([Segment::Text(copy_of(b"Set roles"))]),
            JigPhrase::ActMakeService => Box::new([Segment::Text(copy_of(b"Make service"))]),
            JigPhrase::ActSetPool => Box::new([Segment::Text(copy_of(b"Set pool"))]),
            JigPhrase::ChatAbout => Box::new([Segment::Text(copy_of(b"Chat about it"))]),
            JigPhrase::TaskStarting => Box::new([Segment::Text(copy_of(b"Starting"))]),
            JigPhrase::Topic => Box::new([
                Segment::Text(copy_of(b"Subscribed topic ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            JigPhrase::ElapsedSpan => Box::new([
                Segment::Text(copy_of(b"For ")),
                Segment::Hole { index: 0, kind: HoleKind::Span },
                Segment::Text(copy_of(b" min")),
            ]),
            JigPhrase::ReasonField => Box::new([Segment::Text(copy_of(b"Your reason"))]),
            JigPhrase::LoadingPhase => Box::new([Segment::Text(copy_of(b"Loading phase details"))]),
            JigPhrase::Deadline => {
                Box::new([Segment::Text(copy_of(b"Deadline ")), Segment::Hole { index: 0, kind: HoleKind::Wall }])
            }
            JigPhrase::TriesExhausted => Box::new([Segment::Text(copy_of(b"Held: out of tries"))]),
            JigPhrase::StalledOn => Box::new([
                Segment::Text(copy_of(b"Held: stalled on ")),
                Segment::Hole { index: 0, kind: HoleKind::Name },
            ]),
            JigPhrase::DependencyEnded => Box::new([
                Segment::Text(copy_of(b"Held: dependency ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" ended unsuccessfully")),
            ]),
            JigPhrase::DriftHold => Box::new([Segment::Text(copy_of(b"Held: the system changed"))]),
            JigPhrase::EffectFailedHold => Box::new([Segment::Text(copy_of(b"Held: the effect failed for good"))]),
            JigPhrase::EffectUncertainHold => Box::new([Segment::Text(copy_of(
                b"Held: the effect may or may not have been made; inspect the system before deciding",
            ))]),
            JigPhrase::BudgetHold => Box::new([
                Segment::Text(copy_of(b"Held: spent ")),
                Segment::Hole { index: 0, kind: HoleKind::Amount },
                Segment::Text(copy_of(b", budget ")),
                Segment::Hole { index: 1, kind: HoleKind::Amount },
            ]),
            JigPhrase::DeadlineHold => Box::new([
                Segment::Text(copy_of(b"Held: past deadline ")),
                Segment::Hole { index: 0, kind: HoleKind::Wall },
            ]),
            JigPhrase::StoppedHold => Box::new([
                Segment::Text(copy_of(b"Held: stopped by ")),
                Segment::Hole { index: 0, kind: HoleKind::Name },
                Segment::Text(copy_of(b" at ")),
                Segment::Hole { index: 1, kind: HoleKind::Wall },
            ]),
            JigPhrase::ResourceHold => {
                Box::new([Segment::Text(copy_of(b"Held: waiting for a resource past its bound"))])
            }
            JigPhrase::PoolHold => Box::new([Segment::Text(copy_of(b"Held: the pool lost its allocation"))]),
            JigPhrase::ProcedureHold => Box::new([Segment::Text(copy_of(b"Held: the procedure needs a decision"))]),
            JigPhrase::LoadingDependency => Box::new([
                Segment::Text(copy_of(b"Loading task ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" phase facts")),
            ]),
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
        &PhraseLimits { languages: 1, phrases: 203, holes: 3, segments: 2000, text_bytes: 20000, hole_bytes: text },
    )
}
