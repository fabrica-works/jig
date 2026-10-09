//! One-shot reads made from the owner's current document vocabulary.
//! `client/domain/contract.md`, section 6; `domain/engine.md`, 5.2.
//! A read retains no state and changes no read position or keyed request.
//! Authentication and construction finish in one step. The answer moves into
//! the root's journal, where it waits for the last commit already made.
//!
//! | Event | State | Emits |
//! | --- | --- | --- |
//! | Read, visible live object | unchanged | held document |
//! | Read, absent or invisible | unchanged | held unknown |
//! | Read, construction bound exceeded | unchanged | held busy |

use crate::documents::{self, Document, DocumentClock, Missing, Proposer, Subject, Window};
use crate::{Core, Held, Limits, Request, Requests};
use alloc::boxed::Box;
use skein_lib::{Env, Queue, ReplyTo};

/// One document named by an unkeyed authenticated read.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Read {
    /// The signed-in party's frame.
    Frame,
    /// A task's overview.
    Task { task: u64 },
    /// An immutable proposal and its routing.
    Proposal { proposer: Proposer, proposal: u64 },
    /// A held task's escalation.
    Escalation { task: u64 },
    /// A task's whole question.
    Question { task: u64, message: u64 },
    /// A task addressed to a person or role.
    PersonTask { task: u64 },
    /// A task's result while named by the live engine.
    Result { task: u64 },
    /// A project's resources, policy and funding.
    Project { project: u32 },
    /// Deployment workers, accounts and capacity.
    Deployment,
}

/// The sole terminal of one read, released after its durability barrier.
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Answer {
    /// A bounded current document.
    Document(Box<Document>),
    /// No document can be returned for this read.
    None(Why),
}

/// Small refusal facts which reveal no invisible object identity.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Why {
    /// No subject is known to this sign-in, including subjects it cannot see.
    Unknown,
    /// The answer or required load cannot fit its admission bounds.
    Busy,
    /// The requested historical value is no longer retained.
    Gone,
}

pub(crate) fn start(core: &Core, env: &Env<Limits>, reply: ReplyTo, sign_in: u64, read: Read) -> Requests {
    let answer = match core.people.person(sign_in, env.now, env.wall) {
        Some(party) => make(core, env, party, read),
        None => Answer::None(Why::Unknown),
    };
    let mut out = Queue::with_capacity(2);
    out.push(Request::Held(Box::new(Held::Read { reply, answer })));
    out.push(Request::Decided);
    Requests::Out(out)
}

fn make(core: &Core, env: &Env<Limits>, party: u64, read: Read) -> Answer {
    let subject = match read {
        Read::Frame => Subject::Frame { party },
        Read::Task { task } => Subject::Task { task },
        Read::Proposal { proposer, proposal } => Subject::Proposal { proposer, number: proposal },
        Read::Escalation { task } => Subject::Escalation { task },
        Read::Question { task, message } => Subject::Question { task, message },
        Read::PersonTask { task } => Subject::PersonTask { task },
        Read::Result { task } => Subject::Result { task },
        Read::Project { project } => Subject::Project { project },
        Read::Deployment => Subject::Deployment,
    };
    let window = Window { rows: env.limits.documents.rows, after: None };
    match documents::make_with_clock(
        core,
        &env.limits,
        party,
        &subject,
        &window,
        DocumentClock { now: env.now, wall: env.wall },
    ) {
        Ok(document) => Answer::Document(Box::new(document)),
        Err(Missing::Unknown | Missing::NotAllowed | Missing::Ended) => Answer::None(Why::Unknown),
        Err(Missing::TooLarge) => Answer::None(Why::Busy),
    }
}
