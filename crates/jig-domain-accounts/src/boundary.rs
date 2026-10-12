use skein_lib::{Duration, Time};

/// A name and the time its token has left; never the token itself.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Grant {
    pub account: u32,
    pub generation: u64,
    pub valid: Duration,
}

/// Accounts on the component's terms (protocol/llm.md, 5.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Event {
    Want,
    Component(FromComponent),
    Rejected { account: u32, generation: u64 },
    Exhausted { account: u32, retry_after: Duration },
    Close,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FromComponent {
    Granted { account: u32, generation: u64, valid: Duration },
    Expiring { account: u32, generation: u64 },
    Failed { account: u32, failure: Failure },
    Closed,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Failure {
    Later { after: Option<Duration> },
    SignInAgain,
    Misconfigured,
    Unloaded { at: Place, why: Unread },
    Cancelled,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Place {
    Directory,
    File,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Unread {
    Unreadable,
    Refused,
    Failed,
    Stalled,
    TooLarge,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ToComponent {
    Want { account: u32 },
    Rejected { account: u32, generation: u64 },
    Release { account: u32 },
    Close,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Request {
    Component(ToComponent),
    Granted { grant: Grant },
    Closed,
}

/// Secret-free account state (domain/engine.md, section 12).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum State {
    Asking,
    Granted,
    Later { until: Time },
    SignInAgain,
    Misconfigured,
    Unloaded { at: Place, why: Unread },
    Cancelled,
    Closing,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Fact {
    pub account: u32,
    pub state: State,
    pub generation: Option<u64>,
    pub usable: bool,
    pub expiring: bool,
    pub spent_until: Option<Time>,
    pub attention: bool,
}
