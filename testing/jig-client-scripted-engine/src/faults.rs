//! Faults at a world's chosen clock instant; client/domain/testing.md, 7.
//! Busy/not-ready arm one unmade attempt; loss preserves the key; restart drops
//! undurable snapshots and provisional text, retaining durable answers by key.
use crate::{Application, Engine, Envelope, Up};
use jig_client_domain::contract::{ReadNone, ReadResult, VersionRange, WatchEnd, WatchEvent};
use skein_lib::{Duration, Time, Token};

/// Transport and engine faults chosen by a scenario or seeded world.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    Busy,
    NotReady,
    Lose {
        attempt: Token,
    },
    DropWatch {
        watch: Token,
    },
    /// Delay queued output and future responses until a zero delay clears it.
    Slow {
        span: Duration,
    },
    Restart {
        offline: Duration,
    },
    SignOut {
        sign_in: u64,
    },
    /// Replace the protocol's served range; the next stream hello supplies it.
    Versions(VersionRange),
}
impl<A: Application> Engine<A> {
    /// Applies the fault at this instant; `poll` supplies its envelopes and terminals.
    pub fn fault(&mut self, now: Time, fault: Fault) {
        self.clock = self.clock.max(now);
        match fault {
            Fault::Busy | Fault::NotReady => {
                self.request_fault = Some(Envelope::Again);
            }
            Fault::Lose { attempt } => {
                for due in &mut self.due {
                    if due.commit == 0 {
                        continue;
                    }
                    match &mut due.up {
                        Up::Answer { attempt: id, envelope, answer } if *id == attempt => {
                            *envelope = Envelope::Lost;
                            *answer = None;
                        }
                        Up::ApplicationAnswer { attempt: id, envelope, answer } if *id == attempt => {
                            *envelope = Envelope::Lost;
                            *answer = None;
                        }
                        Up::Answer { .. }
                        | Up::ApplicationAnswer { .. }
                        | Up::Read { .. }
                        | Up::ApplicationRead { .. }
                        | Up::Watch { .. }
                        | Up::App { .. }
                        | Up::Ended { .. } => {}
                    }
                }
            }
            Fault::DropWatch { watch } => {
                self.end_watch(now, watch, WatchEnd::Dropped, 0);
                self.end_application_watch(now, watch, WatchEnd::Dropped, 0);
            }
            Fault::Slow { span: by } => {
                self.slow = by;
                let delayed: std::collections::BTreeSet<_> = self
                    .due
                    .iter()
                    .filter_map(|due| match due.up {
                        Up::Watch { token, event: WatchEvent::Delivered(_), .. } | Up::App { watch: token, .. } => {
                            Some(token)
                        }
                        Up::Answer { .. }
                        | Up::ApplicationAnswer { .. }
                        | Up::Read { .. }
                        | Up::ApplicationRead { .. }
                        | Up::Watch { .. }
                        | Up::Ended { .. } => None,
                    })
                    .collect();
                self.due.retain(|due| {
                    !matches!(due.up, Up::Watch { event: WatchEvent::Delivered(_), .. } | Up::App { .. })
                });
                for due in &mut self.due {
                    due.at = now.max(due.at).saturating_add(by);
                }
                for (token, watch) in &mut self.watches {
                    watch.dirty = true;
                    if delayed.contains(token) {
                        watch.in_flight = false;
                    }
                    watch.ready = now.saturating_add(by);
                }
                for (token, watch) in &mut self.application_watches {
                    watch.dirty = true;
                    if delayed.contains(token) {
                        watch.in_flight = false;
                    }
                    watch.ready = now.saturating_add(by);
                }
            }
            Fault::Restart { offline } => self.restart(now, offline),
            Fault::SignOut { sign_in } => self.sign_out(now, sign_in),
            Fault::Versions(range) => self.versions = range,
        }
    }
    fn restart(&mut self, now: Time, offline: Duration) {
        self.advance_commits(now);
        let durable = self.commits.last().map_or(0, |commit| commit.number);
        self.pending.clear();
        self.keys.retain(|_, keyed| keyed.commit <= durable);
        self.truth = self.durable_truth.clone();
        self.application = self.durable_application.clone();
        self.next_commit = durable.checked_add(1).expect("scenario commit numbers fit");
        self.runs.clear();
        self.positions.clear();
        self.request_fault = None;
        let queued = std::mem::take(&mut self.due);
        for due in queued {
            let failed = match due.up {
                Up::Answer { attempt, .. } => Some(Up::Answer { attempt, envelope: Envelope::Lost, answer: None }),
                Up::ApplicationAnswer { attempt, .. } => {
                    Some(Up::ApplicationAnswer { attempt, envelope: Envelope::Lost, answer: None })
                }
                Up::Read { token, .. } => Some(Up::Read { token, terminal: ReadResult::None(ReadNone::Unreachable) }),
                Up::ApplicationRead { token, .. } => {
                    Some(Up::ApplicationRead { token, terminal: Err(ReadNone::Unreachable) })
                }
                Up::Ended { token, why } => Some(Up::Ended { token, why }),
                Up::Watch { .. } | Up::App { .. } => None,
            };
            if let Some(up) = failed {
                self.due.push(crate::Due { at: now, commit: 0, up });
            }
        }
        let watches: Vec<_> = self.watches.keys().chain(self.application_watches.keys()).copied().collect();
        self.watches.clear();
        self.application_watches.clear();
        for token in watches {
            self.due.push(crate::Due { at: now, commit: 0, up: Up::Ended { token, why: WatchEnd::Dropped } });
        }
        self.offline = Some(now.saturating_add(offline));
    }
    fn sign_out(&mut self, now: Time, sign_in: u64) {
        self.truth.sign_ins.remove(&sign_in);
        self.durable_truth.sign_ins.remove(&sign_in);
        for pending in &mut self.pending {
            pending.truth.sign_ins.remove(&sign_in);
        }
        for due in &mut self.due {
            match &mut due.up {
                Up::Answer { attempt, envelope, answer } if self.attempts.get(attempt) == Some(&sign_in) => {
                    *envelope = Envelope::SignedOut;
                    *answer = None;
                    due.commit = 0;
                    due.at = now;
                }
                Up::ApplicationAnswer { attempt, envelope, answer } if self.attempts.get(attempt) == Some(&sign_in) => {
                    *envelope = Envelope::SignedOut;
                    *answer = None;
                    due.commit = 0;
                    due.at = now;
                }
                Up::Read { token, terminal } if self.reads.get(token) == Some(&sign_in) => {
                    *terminal = ReadResult::None(ReadNone::SignedOut);
                    due.commit = 0;
                    due.at = now;
                }
                Up::ApplicationRead { token, terminal } if self.reads.get(token) == Some(&sign_in) => {
                    *terminal = Err(ReadNone::SignedOut);
                    due.commit = 0;
                    due.at = now;
                }
                Up::Answer { .. }
                | Up::ApplicationAnswer { .. }
                | Up::Read { .. }
                | Up::ApplicationRead { .. }
                | Up::Watch { .. }
                | Up::App { .. }
                | Up::Ended { .. } => {}
            }
        }
        let watches: Vec<_> = self
            .watches
            .iter()
            .filter(|(_, w)| w.sign_in == sign_in)
            .map(|(t, _)| *t)
            .chain(self.application_watches.iter().filter(|(_, w)| w.sign_in == sign_in).map(|(t, _)| *t))
            .collect();
        for token in watches {
            self.end_watch(now, token, WatchEnd::SignedOut, 0);
            self.end_application_watch(now, token, WatchEnd::SignedOut, 0);
        }
    }
    /// Stops all admitted subjects with their one terminal.
    pub fn stop(&mut self, now: Time) {
        let tokens: Vec<_> = self.watches.keys().chain(self.application_watches.keys()).copied().collect();
        for token in tokens {
            self.end_watch(now, token, WatchEnd::Stopping, 0);
            self.end_application_watch(now, token, WatchEnd::Stopping, 0);
        }
    }
}
