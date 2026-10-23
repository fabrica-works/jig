/// The ask the core sets for a run: text for a conversation's own task (engine.md, 7.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct LiveAsk {
    pub text: bool,
    pub reasoning: bool,
    pub input: bool,
}
impl LiveAsk {
    pub const NONE: LiveAsk = LiveAsk { text: false, reasoning: false, input: false };
}
