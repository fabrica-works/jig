use alloc::boxed::Box;
use skein_lib::{Duration, Time};

/// The protocol layer's table that holds a grant (protocol/channel.md, section 8).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GrantTable {
    Accounts,
    Application,
}

/// A name and the time its token has left; never the token itself.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Grant {
    pub table: GrantTable,
    pub account: u32,
    pub generation: u64,
    pub valid: Duration,
}

/// Accounts on the component's terms (protocol/llm.md, 5.2), and what runs' agents
/// notice (domain/accounts.md, 3.5): a generation unauthorized; an account limited until
/// its reset, or, with none, until its holder resumes it; an account refused, with the
/// provider's code and reason; a run failed because its account turned limited or
/// refused. A notice names its account by the grant's number, which, with one source
/// per account, is the account's.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Event {
    Want,
    Component(FromComponent),
    Unauthorized { account: u32, generation: u64 },
    Limited { account: u32, reset: Option<Time> },
    Refused { account: u32, code: Box<[u8]>, reason: Box<[u8]> },
    RunFailed { account: u32, turned: Turned },
    Resume { account: u32 },
    Credential { number: u32, generation: u64, valid: Duration },
    Close,
}

/// What a failed run says its account turned.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Turned {
    Limited { reset: Option<Time> },
    Refused,
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

/// Secret-free account state (domain/engine.md, section 12; domain/accounts.md, 3.5):
/// limited until a time, or, with none, until its holder resumes it; refused until its
/// holder acts.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum State {
    Asking,
    Granted,
    Later { until: Time },
    Limited { until: Option<Time> },
    Refused,
    NeedsSignIn,
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
