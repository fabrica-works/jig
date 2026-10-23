//! Live content, fenced at admission and at drain (hosts.md, section 6.5).
use alloc::boxed::Box;
use skein_lib::{Env, Id, Token};

use crate::hosted::Hosted;
use crate::{Domain, Limits, LinkUp};

/// The ask for live content: one boolean per kind (hosts.md, section 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct LiveAsk {
    pub text: bool,
    pub reasoning: bool,
    pub input: bool,
}
impl LiveAsk {
    pub const NONE: LiveAsk = LiveAsk { text: false, reasoning: false, input: false };
}

/// A piece's kind (hosts.md, 6.5); input names its call by the provider's id.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum LiveKind {
    Text,
    Reasoning,
    Input { call: Box<[u8]>, tool: Box<[u8]> },
}

/// A piece of a run's live content, with the agent's numbers (hosts.md, 6.5).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Piece {
    pub turn: u32,
    pub retry: u32,
    pub kind: LiveKind,
    pub offset: u64,
    pub bytes: Box<[u8]>,
}

pub(crate) fn reported(domain: &mut Domain, env: &Env<Limits>, owner: Token, piece: Piece) {
    let Some(hosting) = crate::activity::live(domain, owner) else {
        domain.facts.fence_piece();
        return;
    };
    let entry = domain.hosted.get(Id::<Hosted>::from_token(owner)).expect("a live attempt is hosted");
    let asked = match &piece.kind {
        LiveKind::Text => entry.live.text,
        LiveKind::Reasoning => entry.live.reasoning,
        LiveKind::Input { .. } => entry.live.input,
    };
    if !asked
        || crate::assignment::len(&piece.bytes) > u64::from(env.limits.piece_bytes)
        || piece.turn <= entry.taken_turns()
    {
        domain.facts.fence_piece();
        return;
    }
    domain.facts.piece(LinkUp::Piece { run: hosting.run, attempt: hosting.attempt, piece });
}

pub(crate) fn pop_activity(domain: &mut Domain) -> Option<LinkUp> {
    for _ in 0..domain.facts.activity_len() {
        let record = domain.facts.pop_activity()?;
        let valid = match &record {
            LinkUp::Piece { run, attempt, piece } => pending(domain, *run, *attempt, piece.turn),
            LinkUp::Activity { .. }
            | LinkUp::Logins { .. }
            | LinkUp::SignedOut { .. }
            | LinkUp::Unauthorized { .. }
            | LinkUp::Limited { .. }
            | LinkUp::AccountRefused { .. }
            | LinkUp::Call { .. }
            | LinkUp::Turn { .. }
            | LinkUp::Answer { .. } => true,
        };
        if valid {
            return Some(record);
        }
        domain.facts.fence_piece();
    }
    None
}

fn pending(domain: &Domain, run: Token, attempt: Token, turn: u32) -> bool {
    let Some(id) = domain.names.get(&run) else { return false };
    if crate::activity::live(domain, id.token()).is_none() {
        return false;
    }
    let entry = domain.hosted.get(*id).expect("a named run is hosted");
    entry.attempt == attempt && turn > entry.taken_turns()
}
