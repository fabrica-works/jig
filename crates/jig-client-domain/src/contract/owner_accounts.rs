//! Secret-free party account facts (client/domain/contract.md, 5.1; domain/accounts.md, 3–5).
use super::{DocumentText, OwnerAccountPlace, OwnerAccountState, OwnerAccountUnread};
use skein_lib::{Duration, List, Wall};
/// An account's payers at most: `jig_domain_accounts::USES` (domain/accounts.md, 3).
pub const ACCOUNT_USES: u32 = 16;
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerAccounts {
    pub accounts: List<OwnerAccountLine>,
    pub signing: List<OwnerAccountSignIn>,
    pub last_end: Option<OwnerAccountLastEnd>,
    pub usable: bool,
    pub settled: bool,
    pub cut: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerAccountLine {
    pub number: u32,
    pub place: u32,
    pub uses: Option<List<OwnerAccountUse>>,
    pub plan: bool,
    pub provider: Option<u32>,
    pub display: DocumentText,
    pub holder: OwnerAccountHolder,
    pub sources: List<OwnerAccountSource>,
    pub cut: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerAccountSource {
    pub place: u32,
    pub number: u32,
    pub kind: OwnerAccountKind,
    pub state: OwnerAccountState,
    pub usable: bool,
    pub expiring: bool,
    pub attention: bool,
    pub spent_until: Option<Wall>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerAccountSignIn {
    pub source: u32,
    pub provider: u32,
    pub holder: OwnerAccountHolder,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerAccountLastEnd {
    pub source: u32,
    pub end: OwnerAccountEnd,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OwnerAccountUse {
    Party(u64),
    Project(u32),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OwnerAccountHolder {
    Deployment,
    Party(u64),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OwnerAccountKind {
    SignIn,
    Key,
    Program,
    None,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OwnerAccountBound {
    Accounts,
    HolderAccounts,
    Sources,
    SignIns,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OwnerAccountKeyFailure {
    TooLong,
    NotStored,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OwnerAccountRefusal {
    Configured,
    NotLimited,
    Uses,
    NotKept { why: OwnerAccountKeyFailure },
    Limit { bound: OwnerAccountBound },
    Held { provider: u32 },
    Provider { provider: u32 },
    Subject,
    Unknown,
    Numbers,
    Kind { kind: OwnerAccountKind },
    Closing,
    NotStarted,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OwnerAccountFailure {
    Later { after: Option<Duration> },
    SignInAgain,
    MissingScope { scope: u32 },
    Misconfigured,
    Unloaded { at: OwnerAccountPlace, why: OwnerAccountUnread },
    Cancelled,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OwnerAccountEnd {
    Made { account: u32, declined: bool },
    Renewed { account: u32, declined: bool },
    Refused { refusal: OwnerAccountRefusal },
    Failed { failure: OwnerAccountFailure },
}
