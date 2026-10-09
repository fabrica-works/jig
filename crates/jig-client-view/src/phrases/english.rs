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
    let mut phrases = List::with_capacity(379);
    for phrase in ALL {
        let segments: Box<[Segment]> = match phrase {
            JigPhrase::ProposalsWaiting => Box::new([
                Segment::Text(copy_of(b"Pending proposals: ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),

            JigPhrase::RetryingPhase => Box::new([Segment::Text(copy_of(b"Retrying"))]),
            JigPhrase::ResourceScope => Box::new([
                Segment::Text(copy_of(b"Resource scope: ")),
                Segment::Hole { index: 0, kind: HoleKind::Name },
            ]),
            JigPhrase::ResultExpected => Box::new([Segment::Text(copy_of(b"Result expected"))]),
            JigPhrase::ReportExpected => Box::new([Segment::Text(copy_of(b"A report"))]),
            JigPhrase::WordsExpected => Box::new([Segment::Text(copy_of(b"Words"))]),
            JigPhrase::ApprovalExpected => Box::new([Segment::Text(copy_of(b"Approval of the named state"))]),

            JigPhrase::Overview => Box::new([Segment::Text(copy_of(b"Overview"))]),
            JigPhrase::Activity => Box::new([Segment::Text(copy_of(b"Activity"))]),
            JigPhrase::Plan => Box::new([Segment::Text(copy_of(b"Plan"))]),
            JigPhrase::History => Box::new([Segment::Text(copy_of(b"History"))]),
            JigPhrase::BudgetAuthority => Box::new([Segment::Text(copy_of(b"Budget and authority"))]),
            JigPhrase::Panels => Box::new([Segment::Text(copy_of(b"Connectors"))]),
            JigPhrase::Goal => Box::new([Segment::Text(copy_of(b"Goal"))]),
            JigPhrase::Resources => Box::new([Segment::Text(copy_of(b"Resources"))]),
            JigPhrase::Policy => Box::new([Segment::Text(copy_of(b"Policy"))]),
            JigPhrase::Roles => Box::new([Segment::Text(copy_of(b"Roles and services"))]),
            JigPhrase::Spend => Box::new([Segment::Text(copy_of(b"Spend"))]),
            JigPhrase::Recurring => Box::new([Segment::Text(copy_of(b"Recurring tasks"))]),
            JigPhrase::Settings => Box::new([Segment::Text(copy_of(b"Settings"))]),
            JigPhrase::Goals => Box::new([Segment::Text(copy_of(b"Goals"))]),
            JigPhrase::Tasks => Box::new([Segment::Text(copy_of(b"Tasks"))]),
            JigPhrase::Notes => Box::new([Segment::Text(copy_of(b"Notes"))]),
            JigPhrase::Reserved => {
                Box::new([Segment::Text(copy_of(b"Reserved: ")), Segment::Hole { index: 0, kind: HoleKind::Amount }])
            }
            JigPhrase::Left => {
                Box::new([Segment::Text(copy_of(b"Left: ")), Segment::Hole { index: 0, kind: HoleKind::Amount }])
            }
            JigPhrase::TotalBudget => Box::new([
                Segment::Text(copy_of(b"Total budget: ")),
                Segment::Hole { index: 0, kind: HoleKind::Amount },
            ]),
            JigPhrase::OwnBudget => {
                Box::new([Segment::Text(copy_of(b"Own spend: ")), Segment::Hole { index: 0, kind: HoleKind::Amount }])
            }
            JigPhrase::EndedBudget => Box::new([
                Segment::Text(copy_of(b"Ended delegate spend: ")),
                Segment::Hole { index: 0, kind: HoleKind::Amount },
            ]),
            JigPhrase::HeldBelow => {
                Box::new([Segment::Text(copy_of(b"Held below: ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            JigPhrase::FoldSubtree => Box::new([Segment::Text(copy_of(b"Fold subtree"))]),
            JigPhrase::OpenSubtree => Box::new([Segment::Text(copy_of(b"Open subtree"))]),
            JigPhrase::AllPlan => Box::new([Segment::Text(copy_of(b"All plan tasks"))]),
            JigPhrase::HeldPlan => Box::new([Segment::Text(copy_of(b"Held only"))]),
            JigPhrase::LivePlan => Box::new([Segment::Text(copy_of(b"Live only"))]),
            JigPhrase::OwnHistory => Box::new([Segment::Text(copy_of(b"This task only"))]),
            JigPhrase::SubtreeHistory => Box::new([Segment::Text(copy_of(b"This task and its subtree"))]),
            JigPhrase::BatchMade => Box::new([
                Segment::Text(copy_of(b"Made in batch ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            JigPhrase::MovedWork => Box::new([Segment::Text(copy_of(b"Requester changed"))]),
            JigPhrase::GoalWords => Box::new([Segment::Text(copy_of(b"Goal specification"))]),
            JigPhrase::GoalBudget => Box::new([Segment::Text(copy_of(b"Goal budget"))]),
            JigPhrase::GoalDeadline => Box::new([Segment::Text(copy_of(b"Goal deadline (UTC, YYYY-MM-DD HH:MM)"))]),
            JigPhrase::GoalPriority => Box::new([Segment::Text(copy_of(b"Goal priority"))]),
            JigPhrase::PriorityMeaning => {
                Box::new([Segment::Text(copy_of(b"Priority orders work competing for one resource"))])
            }
            JigPhrase::PoolLeft => Box::new([
                Segment::Text(copy_of(b"Your pool has left: ")),
                Segment::Hole { index: 0, kind: HoleKind::Amount },
            ]),
            JigPhrase::GoalPropose => Box::new([Segment::Text(copy_of(b"Beyond your pool, this will be proposed"))]),
            JigPhrase::NoteSearch => Box::new([Segment::Text(copy_of(b"Search notes by description"))]),
            JigPhrase::AskedByMe => Box::new([Segment::Text(copy_of(b"Tasks I asked for"))]),
            JigPhrase::AllTasks => Box::new([Segment::Text(copy_of(b"All tasks"))]),
            JigPhrase::Period => {
                Box::new([Segment::Text(copy_of(b"Period ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            JigPhrase::RoleNumber => {
                Box::new([Segment::Text(copy_of(b"Role ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            JigPhrase::PartyNumber => {
                Box::new([Segment::Text(copy_of(b"Party ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            JigPhrase::Allotment => {
                Box::new([Segment::Text(copy_of(b"Allotment: ")), Segment::Hole { index: 0, kind: HoleKind::Amount }])
            }
            JigPhrase::DecidesKind => {
                Box::new([Segment::Text(copy_of(b"Decides kind ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            JigPhrase::PolicySpend => Box::new([
                Segment::Text(copy_of(b"Spend per period: ")),
                Segment::Hole { index: 0, kind: HoleKind::Amount },
            ]),
            JigPhrase::HostNumber => {
                Box::new([Segment::Text(copy_of(b"Host ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            JigPhrase::HostConnected => Box::new([Segment::Text(copy_of(b"Connected"))]),
            JigPhrase::HostDisconnected => Box::new([Segment::Text(copy_of(b"Lost"))]),
            JigPhrase::SlotsUsed => {
                Box::new([Segment::Text(copy_of(b"Slots used: ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            JigPhrase::SlotsFree => {
                Box::new([Segment::Text(copy_of(b"Slots free: ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            JigPhrase::AccountNumber => {
                Box::new([Segment::Text(copy_of(b"LLM account ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            JigPhrase::AccountUsable => Box::new([Segment::Text(copy_of(b"Usable"))]),
            JigPhrase::AccountUnavailable => Box::new([Segment::Text(copy_of(b"Unavailable"))]),
            JigPhrase::Capacity => Box::new([Segment::Text(copy_of(b"Capacity"))]),
            JigPhrase::TasksLive => {
                Box::new([Segment::Text(copy_of(b"Live tasks: ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            JigPhrase::CapacityLimit => {
                Box::new([Segment::Text(copy_of(b"Task limit: ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            JigPhrase::Grace => Box::new([
                Segment::Text(copy_of(b"Grace holds until ")),
                Segment::Hole { index: 0, kind: HoleKind::Wall },
            ]),
            JigPhrase::WaitingAccount => Box::new([Segment::Text(copy_of(b"Tasks waiting for this account"))]),
            JigPhrase::ParkedResume => Box::new([Segment::Text(copy_of(b"Parked; your words resume it"))]),

            JigPhrase::OwnSpend => {
                Box::new([Segment::Text(copy_of(b"Agent spend: ")), Segment::Hole { index: 0, kind: HoleKind::Amount }])
            }
            JigPhrase::DelegateSpend => Box::new([
                Segment::Text(copy_of(b"Delegate spend: ")),
                Segment::Hole { index: 0, kind: HoleKind::Amount },
            ]),

            JigPhrase::NeedsYou => Box::new([Segment::Text(copy_of(b"Needs you"))]),
            JigPhrase::InboxUpdates => Box::new([Segment::Text(copy_of(b"Updates"))]),
            JigPhrase::AllKinds => Box::new([Segment::Text(copy_of(b"All kinds"))]),
            JigPhrase::AllGoals => Box::new([Segment::Text(copy_of(b"All goals"))]),
            JigPhrase::OnlyGoal => {
                Box::new([Segment::Text(copy_of(b"Goal ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            JigPhrase::NewUpdate => Box::new([Segment::Text(copy_of(b"New"))]),
            JigPhrase::OnCall => Box::new([Segment::Text(copy_of(b"For the person on call"))]),
            JigPhrase::PassesUp => {
                Box::new([Segment::Text(copy_of(b"Passes up at ")), Segment::Hole { index: 0, kind: HoleKind::Wall }])
            }
            JigPhrase::ReadEarlier => Box::new([Segment::Text(copy_of(b"Read earlier"))]),
            JigPhrase::ReadLater => Box::new([Segment::Text(copy_of(b"Read later"))]),
            JigPhrase::WindowStart => Box::new([Segment::Text(copy_of(b"Start of the list"))]),
            JigPhrase::WindowEnd => Box::new([Segment::Text(copy_of(b"End of the list"))]),
            JigPhrase::MoreRows => Box::new([Segment::Text(copy_of(b"More entries are available"))]),
            JigPhrase::NewChat => Box::new([Segment::Text(copy_of(b"First words"))]),
            JigPhrase::StartChat => Box::new([Segment::Text(copy_of(b"Start a conversation"))]),
            JigPhrase::Delegates => Box::new([
                Segment::Text(copy_of(b"Live delegates: ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            JigPhrase::SignInPage => Box::new([Segment::Text(copy_of(b"Sign in"))]),
            JigPhrase::SignInWith => {
                Box::new([Segment::Text(copy_of(b"Sign in with ")), Segment::Hole { index: 0, kind: HoleKind::Name }])
            }
            JigPhrase::MissingPage => Box::new([Segment::Text(copy_of(b"Page missing"))]),
            JigPhrase::UpdateClient => Box::new([Segment::Text(copy_of(
                b"Client out of date: update or reload this client to continue; pending requests are kept",
            ))]),
            JigPhrase::SomeoneElse => {
                Box::new([Segment::Text(copy_of(b"Someone else's work: sign in as its owner to continue"))])
            }

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
                Segment::Text(copy_of(b" delegates")),
            ]),
            JigPhrase::ClosingConnector => Box::new([Segment::Text(copy_of(b"and "))]),
            JigPhrase::ClosingEffects => Box::new([Segment::Text(copy_of(b"'s effects settling"))]),
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
            JigPhrase::Options | JigPhrase::MissingChoice => Box::new([Segment::Text(copy_of(b"Choose an option"))]),
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
            JigPhrase::BeforeHold => Box::new([Segment::Text(copy_of(b"Before the hold"))]),
            JigPhrase::SpentRun => Box::new([
                Segment::Text(copy_of(b"Task ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b", attempt ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
                Segment::Text(copy_of(b": spent ")),
                Segment::Hole { index: 2, kind: HoleKind::Amount },
            ]),
            JigPhrase::SpentDelegate => Box::new([
                Segment::Text(copy_of(b"Task ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b": spent ")),
                Segment::Hole { index: 1, kind: HoleKind::Amount },
            ]),
            JigPhrase::Confirmation => Box::new([Segment::Text(copy_of(b"Review before confirming"))]),
            JigPhrase::Confirm => Box::new([Segment::Text(copy_of(b"Confirm"))]),
            JigPhrase::Dismiss => Box::new([Segment::Text(copy_of(b"Dismiss"))]),
            JigPhrase::Dangerous => Box::new([Segment::Text(copy_of(b"Dangerous action: review its consequences"))]),
            JigPhrase::WillPropose => Box::new([Segment::Text(copy_of(b"This will be proposed to its holder"))]),
            JigPhrase::ChangedConfirmation => Box::new([Segment::Text(copy_of(
                b"The revision this action names changed; inspect the current facts and open it again",
            ))]),
            JigPhrase::MissingWords => Box::new([Segment::Text(copy_of(b"Required words are missing"))]),
            JigPhrase::MissingAgain => Box::new([Segment::Text(copy_of(b"Choose whether to make the effect again"))]),
            JigPhrase::MissingChange => Box::new([Segment::Text(copy_of(b"Supply the amendment"))]),
            JigPhrase::MissingDelegates => {
                Box::new([Segment::Text(copy_of(b"Choose what becomes of every live delegate"))])
            }
            JigPhrase::MissingPriority => Box::new([Segment::Text(copy_of(b"Supply a priority"))]),
            JigPhrase::MissingContext => {
                Box::new([Segment::Text(copy_of(b"Waiting for this object's project context"))])
            }
            JigPhrase::WillAccept => {
                Box::new([Segment::Text(copy_of(b"Acceptance makes or changes the following, funded as shown"))])
            }
            JigPhrase::WillReject => {
                Box::new([Segment::Text(copy_of(b"Reject this proposal and send its proposer your reason"))])
            }
            JigPhrase::WillPassUp => Box::new([Segment::Text(copy_of(b"Pass this decision to the holder above"))]),
            JigPhrase::WillLeaveHeld => Box::new([Segment::Text(copy_of(
                b"Remove this escalation from your inbox; the task stays held with your reason",
            ))]),
            JigPhrase::WillStop => Box::new([Segment::Text(copy_of(b"Stop the live run and hold its task"))]),
            JigPhrase::WillCancel => Box::new([Segment::Text(copy_of(
                b"Cancel this task and everything below it; already made effects remain",
            ))]),
            JigPhrase::WillClose => Box::new([Segment::Text(copy_of(
                b"Close this chat after disposing of every live delegate; pending effects settle",
            ))]),
            JigPhrase::WillTakeOver => {
                Box::new([Segment::Text(copy_of(b"Become this task's requester and move its funding"))])
            }
            JigPhrase::WillTake => {
                Box::new([Segment::Text(copy_of(b"Take this role task so others cannot answer it at once"))])
            }
            JigPhrase::WillHandBack => Box::new([Segment::Text(copy_of(b"Hand this task back to its role"))]),
            JigPhrase::WillWrite => Box::new([Segment::Text(copy_of(b"Send these words to the task"))]),
            JigPhrase::WillAnswer => {
                Box::new([Segment::Text(copy_of(b"Send the answer matching the task's contract"))])
            }
            JigPhrase::WillAmend => {
                Box::new([Segment::Text(copy_of(b"Replace the named specification with these changes"))])
            }
            JigPhrase::WillAmendRelease => Box::new([Segment::Text(copy_of(b"Amend and release in one decision"))]),
            JigPhrase::WillPrioritise => Box::new([Segment::Text(copy_of(b"Change this goal's priority"))]),
            JigPhrase::WillEditNote => Box::new([Segment::Text(copy_of(b"Replace the named note revision"))]),
            JigPhrase::WillDeleteNote => Box::new([Segment::Text(copy_of(b"Delete this note revision"))]),
            JigPhrase::WillGoal => Box::new([Segment::Text(copy_of(b"Make a goal from these words and options"))]),
            JigPhrase::ResumeRelease => Box::new([Segment::Text(copy_of(
                b"The next run resumes its conversation; tries count again from zero",
            ))]),
            JigPhrase::ContinueClosing => {
                Box::new([Segment::Text(copy_of(b"Releasing carries on closing with the kept result"))])
            }
            JigPhrase::ReleaseWait => Box::new([Segment::Text(copy_of(b"Release to wait again"))]),
            JigPhrase::ReleaseFresh => Box::new([Segment::Text(copy_of(b"Release for the executor to decide afresh"))]),
            JigPhrase::ReleaseProcedure => Box::new([Segment::Text(copy_of(b"Release with procedure counts reset"))]),
            JigPhrase::RepeatEffect => Box::new([Segment::Text(copy_of(b"Make the effect again"))]),
            JigPhrase::SkipEffect => {
                Box::new([Segment::Text(copy_of(b"Release without making the effect again; tell its executor so"))])
            }
            JigPhrase::EngineFollowsChoice => Box::new([Segment::Text(copy_of(
                b"Inspect the connector's system facts; the engine will do as you choose",
            ))]),
            JigPhrase::TasksWaiting => Box::new([
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" waiting tasks")),
            ]),
            JigPhrase::TasksRunning => Box::new([
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" running tasks")),
            ]),
            JigPhrase::TasksHeld => {
                Box::new([Segment::Hole { index: 0, kind: HoleKind::Number }, Segment::Text(copy_of(b" held tasks"))])
            }
            JigPhrase::TasksClosing => Box::new([
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" closing tasks")),
            ]),
            JigPhrase::TasksDone => Box::new([
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" completed tasks")),
            ]),
            JigPhrase::TasksFailed => {
                Box::new([Segment::Hole { index: 0, kind: HoleKind::Number }, Segment::Text(copy_of(b" failed tasks"))])
            }
            JigPhrase::TasksCancelled => Box::new([
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" cancelled tasks")),
            ]),
            JigPhrase::LiveRuns => {
                Box::new([Segment::Hole { index: 0, kind: HoleKind::Number }, Segment::Text(copy_of(b" live runs"))])
            }
            JigPhrase::PendingProposals => Box::new([
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" proposals will be withdrawn")),
            ]),
            JigPhrase::PendingEffects => Box::new([
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" effects not yet begun will be cancelled")),
            ]),
            JigPhrase::SettlingEffects => Box::new([
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" begun effects will settle")),
            ]),
            JigPhrase::TakeoverFunds => Box::new([
                Segment::Text(copy_of(b"Takeover reserves ")),
                Segment::Hole { index: 0, kind: HoleKind::Amount },
                Segment::Text(copy_of(b" from your pool")),
            ]),
            JigPhrase::DelegateCancel => Box::new([
                Segment::Text(copy_of(b"Cancel delegate ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b" and its subtree")),
            ]),
            JigPhrase::DelegateTake => Box::new([
                Segment::Text(copy_of(b"Take over delegate ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b", reserving ")),
                Segment::Hole { index: 1, kind: HoleKind::Amount },
            ]),
            JigPhrase::DelegateChoices => Box::new([Segment::Text(copy_of(b"Choose each delegate's outcome"))]),
            JigPhrase::WriteAndRelease => Box::new([Segment::Text(copy_of(b"Write and release together"))]),
            JigPhrase::WriteOnly => Box::new([Segment::Text(copy_of(b"Write without releasing"))]),
            JigPhrase::NewSpec => Box::new([Segment::Text(copy_of(b"New specification"))]),
            JigPhrase::NewInstructions => Box::new([Segment::Text(copy_of(b"New instructions"))]),
            JigPhrase::NewBudget => Box::new([Segment::Text(copy_of(b"New budget"))]),
            JigPhrase::NewDeadline => Box::new([Segment::Text(copy_of(b"New deadline (UTC, YYYY-MM-DD HH:MM)"))]),
            JigPhrase::DropDependency => Box::new([
                Segment::Text(copy_of(b"Drop dependency ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            JigPhrase::PriorityField => Box::new([Segment::Text(copy_of(b"New goal priority"))]),
            JigPhrase::NoteDescription => Box::new([Segment::Text(copy_of(b"Note description"))]),
            JigPhrase::NoteBody => Box::new([Segment::Text(copy_of(b"Note body"))]),
            JigPhrase::Charter => {
                Box::new([Segment::Text(copy_of(b"Charter ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            JigPhrase::Priority => {
                Box::new([Segment::Text(copy_of(b"Priority ")), Segment::Hole { index: 0, kind: HoleKind::Number }])
            }
            JigPhrase::Notice => Box::new([Segment::Text(copy_of(b"Notice"))]),
            JigPhrase::UnknownAction => {
                Box::new([Segment::Text(copy_of(b"Outcome unknown for this action; inspect its current facts"))])
            }
            JigPhrase::ResultsThrough => Box::new([
                Segment::Text(copy_of(b"Results through ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
            ]),
            JigPhrase::RepliesThrough => Box::new([
                Segment::Text(copy_of(b"Chat ")),
                Segment::Hole { index: 0, kind: HoleKind::Number },
                Segment::Text(copy_of(b", replies through ")),
                Segment::Hole { index: 1, kind: HoleKind::Number },
            ]),
            JigPhrase::FailedPhase => Box::new([Segment::Text(copy_of(b"Failed"))]),
            JigPhrase::CancelledPhase => Box::new([Segment::Text(copy_of(b"Cancelled"))]),
            JigPhrase::AddressParty => Box::new([Segment::Text(copy_of(b"Address to party number"))]),
            JigPhrase::AddressTask => Box::new([Segment::Text(copy_of(b"Address to task number"))]),
            JigPhrase::AddressRole => Box::new([Segment::Text(copy_of(b"Address to role number in this project"))]),
            JigPhrase::DuePhase => Box::new([Segment::Text(copy_of(b"Due to run"))]),
            JigPhrase::PreparingPhase => Box::new([Segment::Text(copy_of(b"Preparing a host"))]),
            JigPhrase::ClaimedPhase => Box::new([Segment::Text(copy_of(b"Claimed by a host"))]),
            JigPhrase::ActivePhase => Box::new([Segment::Text(copy_of(b"Executor active"))]),
            JigPhrase::ClosingPhase => Box::new([Segment::Text(copy_of(b"Closing"))]),
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
        &PhraseLimits { languages: 1, phrases: 379, holes: 3, segments: 2000, text_bytes: 20000, hole_bytes: text },
    )
}
