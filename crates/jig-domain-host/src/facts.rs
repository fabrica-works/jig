//! What the host tells whoever watches it (domain/hosts.md, sections
//! 6.2 and 6.5): a fact for each step of a hosted run's lifecycle, content-free (the
//! engine's names and classifications, never what the charter, the run or the
//! engine said), in a bounded queue the parent drains at its own pace.
//!
//! Facts are outside the boundary's flow control: they are not requests, take
//! no room in `out`, and when the queue is full they are dropped and counted.
//! Nothing the host decides depends on whether a fact was kept. An assignment
//! refused at the entrance tells nothing: no run was hosted. The agent's
//! typed activity waits in its own bounded queue until the parent can send it.

use skein_lib::{Queue, Token};

use crate::boundary::Failure;
use crate::{Activity, LinkUp, Phase};

/// Something that happened to the hosted run `run`'s attempt `attempt`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Fact {
    /// The assignment was admitted.
    Admitted { run: Token, attempt: Token },
    /// Its workspace was prepared.
    Prepared { run: Token, attempt: Token },
    /// Its agent started.
    Started { run: Token, attempt: Token },
    /// The live agent yielded while retaining its host slot.
    Waiting { run: Token, attempt: Token },
    /// An inbound event or call moved it from waiting back to active.
    Resumed { run: Token, attempt: Token },
    /// It answered: parked.
    Parked { run: Token, attempt: Token },
    /// It answered: ended with an outcome.
    Ended { run: Token, attempt: Token },
    /// It answered: failed, for `failure`.
    Failed { run: Token, attempt: Token, failure: Failure },
}

/// The facts not yet drained, and how many did not fit.
#[derive(Debug)]
pub(crate) struct Facts {
    queue: Queue<Fact>,
    activity: Queue<LinkUp>,
    activity_lost: u64,
    lost: u64,
}

impl Facts {
    pub(crate) fn with_capacity(capacity: u32, activity: u32) -> Facts {
        Facts {
            queue: Queue::with_capacity(capacity),
            activity: Queue::with_capacity(activity),
            lost: 0,
            activity_lost: 0,
        }
    }

    /// Keeps `fact` if there is room for it, and counts it otherwise.
    pub(crate) fn push(&mut self, fact: Fact) {
        if self.queue.try_push(fact).is_err() {
            self.lost = self.lost.saturating_add(1);
        }
    }

    pub(crate) fn activity(&mut self, activity: LinkUp) {
        if self.activity.try_push(activity).is_err() {
            self.activity_lost = self.activity_lost.saturating_add(1);
        }
    }

    pub(crate) fn phase(&mut self, run: Token, attempt: Token, previous: &mut Option<Phase>, phase: Phase) {
        if *previous != Some(phase) {
            *previous = Some(phase);
            self.activity(LinkUp::Activity { run, attempt, activity: Activity::Phase(phase) });
        }
    }

    pub(crate) fn pop_activity(&mut self) -> Option<LinkUp> {
        self.activity.pop()
    }
    pub(crate) const fn activity_lost(&self) -> u64 {
        self.activity_lost
    }

    pub(crate) fn pop(&mut self) -> Option<Fact> {
        self.queue.pop()
    }

    pub(crate) fn lost(&self) -> u64 {
        self.lost
    }
}
