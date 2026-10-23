use alloc::boxed::Box;

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
