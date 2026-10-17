//! The conversation header says the public helper's facts with existing phrases.
use super::{Words, name};
use crate::phrases::jig::JigPhrase as P;
use jig_client::words::{StatusWord, Word};
use jig_client::{
    contract::{RetryNotice, WaitedFor, Whom},
    facts::ChatStatus,
};
use skein_lib::{Duration, Wall};
use skein_ui::{Builder, phrases::Hole};
impl Words {
    pub(crate) fn chat_status(&self, b: &mut Builder, status: &ChatStatus, wall: Wall) {
        match status {
            ChatStatus::Starting => self.text(b, P::Word(Word::Status(StatusWord::Starting)), &[]),
            ChatStatus::Writing => self.text(b, P::Word(Word::Status(StatusWord::Writing)), &[]),
            ChatStatus::Tool { tool, since } => {
                self.text(b, P::Word(Word::Status(StatusWord::Tool)), &[name(tool)]);
                self.status_span(b, *since, wall);
            }
            ChatStatus::Working { since } => {
                self.text(b, P::Word(Word::status(status)), &[]);
                self.status_span(b, *since, wall);
            }
            ChatStatus::WaitingForYou { kind, entries } => {
                self.text(b, P::Word(Word::Status(StatusWord::WaitingForYou)), &[]);
                let phrase = match kind {
                    WaitedFor::Question => P::Question,
                    WaitedFor::Proposal => P::Proposal,
                    WaitedFor::Escalation => P::Escalation,
                    WaitedFor::PersonTask => P::PersonTask,
                };
                self.text(b, phrase, &[]);
                self.text(b, P::Needs, &[Hole::Number(u64::from(*entries))]);
            }
            ChatStatus::WaitingFor(whom) => match whom {
                Whom::Person(party) => self.text(b, P::WaitingParty, &[Hole::Number(*party)]),
                Whom::Role { project, role } => {
                    self.text(b, P::WaitingRole, &[Hole::Number(u64::from(*project)), Hole::Number(u64::from(*role))]);
                }
            },
            ChatStatus::Parked => self.text(b, P::Word(Word::Status(StatusWord::Parked)), &[]),
            ChatStatus::Retrying { until, notice } => {
                self.text(b, P::RunBackingOff, &[Hole::Wall(self.adjusted_wall(*until))]);
                match notice {
                    RetryNotice::OwnerFailed { class, count, allowance } => {
                        crate::owner_chip::hold_failure(b, *class, self);
                        self.text(
                            b,
                            P::RunRetryFailed,
                            &[Hole::Number(u64::from(*count)), Hole::Number(u64::from(*allowance))],
                        );
                    }
                    RetryNotice::Failed { class, count, allowance } => {
                        crate::cards::chip::failure(b, *class, self);
                        self.text(
                            b,
                            P::RunRetryFailed,
                            &[Hole::Number(u64::from(*count)), Hole::Number(u64::from(*allowance))],
                        );
                    }
                    RetryNotice::Refused { count } => {
                        self.text(b, P::RunRetryRefused, &[Hole::Number(u64::from(*count))]);
                    }
                }
            }
            ChatStatus::Held(reason) => self.hold_category(b, *reason),
            ChatStatus::Closing => self.text(b, P::Word(Word::status(status)), &[]),
            ChatStatus::Ended { at } => {
                self.text(b, P::Word(Word::status(status)), &[]);
                self.wall(b, *at);
            }
        }
    }
    fn status_span(&self, b: &mut Builder, since: Wall, wall: Wall) {
        self.text(
            b,
            P::ElapsedSpan,
            &[Hole::Span(Duration::from_nanos(wall.as_nanos().saturating_sub(since.as_nanos())))],
        );
    }
}
