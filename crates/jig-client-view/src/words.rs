//! Typed facts said through a checked phrasebook (client/domain/view.md, section 7).
//! Keeps language data, a selected language and the person's UTC offset. It never
//! computes authority. Each saying must fit the phrasebook's checked shape.
mod held;
mod lacking;
mod refusals;
use crate::{
    Builder, Role, Run,
    phrases::{Hole, Phrasebook, jig::JigPhrase},
};
use alloc::boxed::Box;
use jig_client::{
    contract::{Actor, Decision, Ending, Executor, GoalExecutor, Made, Outcome, ProposalKey, Proposer},
    intents::Progress,
    link::Link,
};
use skein_lib::{Duration, Wall, bytes::copy_of};

/// Checked words and the person's time presentation for one tree build.
#[derive(Clone, Debug)]
pub struct Words {
    pub book: Phrasebook,
    pub language: u16,
    pub offset_minutes: i32,
}
impl Words {
    /// Say one phrase in the selected language; mis-sized facts are a caller bug.
    #[must_use]
    pub fn say(&self, phrase: JigPhrase, holes: &[Hole]) -> Box<[u8]> {
        self.book.say(self.language, phrase.id(), holes).expect("checked phrase and bounded facts").text
    }
    /// Emit a text leaf using a phrase's checked words.
    pub fn text(&self, b: &mut Builder, phrase: JigPhrase, holes: &[Hole]) {
        let text = self.say(phrase, holes);
        b.text(Run::Plain, &text);
    }
    /// Emit a named status, also exposing its words as text.
    pub fn status(&self, b: &mut Builder, phrase: JigPhrase, holes: &[Hole]) {
        let text = self.say(phrase, holes);
        b.open(Role::Status);
        b.name(&text);
        b.text(Run::Plain, &text);
        b.close();
    }
    /// Show a link's whole vocabulary.
    pub fn link(&self, b: &mut Builder, link: Link) {
        let phrase = match link {
            Link::Starting => JigPhrase::Starting,
            Link::Live => JigPhrase::Live,
            Link::Offline => JigPhrase::Offline,
        };
        self.status(b, phrase, &[]);
    }
    /// Say the request table's progress without owning its retry policy.
    pub fn progress(&self, b: &mut Builder, progress: Progress) {
        let phrase = match progress {
            Progress::Confirming => JigPhrase::Confirming,
            Progress::InFlight => JigPhrase::InFlight,
            Progress::Retrying => JigPhrase::Retrying,
            Progress::Parked => JigPhrase::Parked,
            Progress::Unknown => JigPhrase::Unknown,
        };
        self.status(b, phrase, &[]);
    }
    /// Say the executor kind; its connector content belongs in a slot.
    pub fn executor(&self, b: &mut Builder, executor: Executor) {
        let phrase = match executor {
            Executor::Agent { .. } => JigPhrase::Agent,
            Executor::Procedure { .. } => JigPhrase::Procedure,
            Executor::Party { .. } => JigPhrase::Person,
            Executor::Role { .. } => JigPhrase::RoleExecutor,
        };
        self.text(b, phrase, &[]);
    }
    /// Say the complete requested goal executor, with only supplied numbers.
    pub fn goal_executor(&self, b: &mut Builder, executor: GoalExecutor) {
        match executor {
            GoalExecutor::Charter(number) => self.text(b, JigPhrase::Charter, &[Hole::Number(u64::from(number))]),
            GoalExecutor::Procedure { connector, code } => self.text(
                b,
                JigPhrase::GoalProcedure,
                &[Hole::Number(u64::from(connector)), Hole::Number(u64::from(code))],
            ),
        }
    }
    /// Show an amount in the engine's units.
    pub fn amount(&self, b: &mut Builder, amount: u64) {
        self.text(b, JigPhrase::Amount, &[Hole::Amount(amount)]);
    }
    /// Show the elapsed wall interval, clamped when the wall clock moved backward.
    pub fn waiting(&self, b: &mut Builder, now: Wall, since: Wall) {
        self.text(
            b,
            JigPhrase::WaitingSpan,
            &[Hole::Span(Duration::from_nanos(now.as_nanos().saturating_sub(since.as_nanos())))],
        );
    }
    /// Apply the person's offset using checked arithmetic, clamped at the epoch.
    pub fn wall(&self, b: &mut Builder, at: Wall) {
        b.open(Role::Time);
        self.text(b, JigPhrase::At, &[Hole::Wall(self.adjusted_wall(at))]);
        b.close();
    }
    /// Name the deadline as such, in the person's time presentation.
    pub fn deadline(&self, b: &mut Builder, at: Wall) {
        b.open(Role::Time);
        self.text(b, JigPhrase::Deadline, &[Hole::Wall(self.adjusted_wall(at))]);
        b.close();
    }
    /// A wall instant shifted to the person's UTC offset for a phrase hole.
    #[must_use]
    pub fn adjusted_wall(&self, at: Wall) -> Wall {
        let nanos = u64::from(self.offset_minutes.unsigned_abs()).saturating_mul(60_000_000_000);
        let adjusted = if self.offset_minutes < 0_i32 {
            at.as_nanos().saturating_sub(nanos)
        } else {
            at.as_nanos().saturating_add(nanos)
        };
        Wall::from_nanos(adjusted)
    }
    /// Say who acted; party names remain unchanged when supplied by the document.
    pub fn actor(&self, b: &mut Builder, by: Actor) {
        match by {
            Actor::Party(number) => self.text(b, JigPhrase::ByParty, &[Hole::Number(number)]),
            Actor::Task(number) => self.text(b, JigPhrase::ByTask, &[Hole::Number(number)]),
            Actor::Deployment => self.text(b, JigPhrase::ByDeployment, &[]),
            Actor::ProjectDeployment { project } => {
                self.text(b, JigPhrase::ByProjectDeployment, &[Hole::Number(u64::from(project))]);
            }
        }
    }
    /// Say each possible durable decision, including the uncertain-effect choice.
    pub fn decision(&self, b: &mut Builder, how: &Decision) {
        match how {
            Decision::Accepted => self.text(b, JigPhrase::Accepted, &[]),
            Decision::Rejected { reason } => self.text(b, JigPhrase::Rejected, &[name(reason.bytes())]),
            Decision::PassedUp => self.text(b, JigPhrase::PassedUp, &[]),
            Decision::Released { again } => {
                let phrase = match again {
                    Some(true) => JigPhrase::ReleasedAgain,
                    Some(false) => JigPhrase::ReleasedWithout,
                    None => JigPhrase::Released,
                };
                self.text(b, phrase, &[]);
            }
            Decision::LeftHeld { reason } => self.text(b, JigPhrase::LeftHeld, &[name(reason.bytes())]),
        }
    }
    /// Say a durable ending and every fact that ending carries.
    pub fn ending(&self, b: &mut Builder, how: &Ending) {
        match how {
            Ending::Done => self.text(b, JigPhrase::Done, &[]),
            Ending::Failed { reason } => match reason {
                Some(reason) => self.text(b, JigPhrase::Failed, &[name(reason.bytes())]),
                None => self.text(b, JigPhrase::FailedBare, &[]),
            },
            Ending::Cancelled { by, reason, at } => {
                match reason {
                    Some(reason) => self.text(b, JigPhrase::Cancelled, &[name(reason.bytes())]),
                    None => self.text(b, JigPhrase::CancelledBare, &[]),
                }
                self.actor(b, *by);
                if let Some(at) = at {
                    self.wall(b, *at);
                }
            }
            Ending::Withdrawn { reason } => self.text(b, JigPhrase::Withdrawn, &[name(reason.bytes())]),
            Ending::Decided { by, party, how, at } => {
                if how.is_none() {
                    self.text(b, JigPhrase::DecidedBare, &[]);
                }
                match party {
                    Some(party) => self.text(b, JigPhrase::ByNamed, &[name(party.name.bytes())]),
                    None => self.actor(b, Actor::Party(*by)),
                }
                if let Some(how) = how {
                    self.decision(b, how);
                }
                self.wall(b, *at);
            }
            Ending::Deleted => self.text(b, JigPhrase::DeletedEnding, &[]),
        }
    }
    /// The named maker, without inventing a party name or another actor kind.
    pub fn proposer(&self, b: &mut Builder, proposer: Proposer) {
        match proposer {
            Proposer::Task(task) => self.actor(b, Actor::Task(task)),
            Proposer::Party(party) => self.actor(b, Actor::Party(party)),
        }
    }
    /// A proposal's maker kind and both full-width numeric identities.
    pub fn proposal(&self, b: &mut Builder, proposal: ProposalKey) {
        let (phrase, maker) = match proposal.proposer {
            Proposer::Task(task) => (JigPhrase::Proposed, task),
            Proposer::Party(party) => (JigPhrase::ProposedByParty, party),
        };
        self.text(b, phrase, &[Hole::Number(maker), Hole::Number(proposal.number)]);
    }
    /// Say all durable outcomes, preserving who decided first and when.
    pub fn outcome(&self, b: &mut Builder, outcome: &Outcome) {
        match outcome {
            Outcome::Owner(value) => crate::owner_outcome::render(b, value, self),
            Outcome::Made { what } => match what {
                Made::Chat { task } => self.text(b, JigPhrase::MadeChat, &[Hole::Number(*task)]),
                Made::Words { task, message } => {
                    self.text(b, JigPhrase::MadeWords, &[Hole::Number(*task), Hole::Number(*message)]);
                }
                Made::Goal { task } => self.text(b, JigPhrase::MadeGoal, &[Hole::Number(*task)]),
                Made::Tasks { tasks } => {
                    for task in tasks {
                        self.text(b, JigPhrase::MadeTask, &[Hole::Number(*task)]);
                    }
                }
                Made::Service { party } => self.text(b, JigPhrase::MadeService, &[Hole::Number(*party)]),
                Made::PersonResult { task } => self.text(b, JigPhrase::MadeResult, &[Hole::Number(*task)]),
                Made::Note { note, .. } => self.text(b, JigPhrase::MadeNote, &[Hole::Number(*note)]),
            },
            Outcome::Proposed { proposal } => {
                self.proposal(b, *proposal);
            }
            Outcome::Decided { by, how, at } => {
                self.actor(b, *by);
                self.decision(b, how);
                self.wall(b, *at);
            }
            Outcome::DecidedFirst { by, how, at } => {
                self.text(b, JigPhrase::DecidedFirst, &[name(by.name.bytes())]);
                self.decision(b, how);
                self.wall(b, *at);
            }
            Outcome::Changed => self.text(b, JigPhrase::Changed, &[]),
        }
    }
}
/// Copy a document name into a phrase's owned hole.
#[must_use]
pub fn name(bytes: &[u8]) -> Hole {
    Hole::Name(copy_of(bytes))
}
