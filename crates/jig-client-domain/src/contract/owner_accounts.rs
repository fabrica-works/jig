//! Secret-free party account facts (client/domain/contract.md, 5.1; domain/accounts.md, 3–5).
use super::{DocumentText, OwnerAccountPlace, OwnerAccountState, OwnerAccountUnread};
use alloc::boxed::Box;
use skein_lib::{Duration, List, Wall};
/// An account's payers at most: `jig_domain_accounts::USES` (domain/accounts.md, 3).
pub const ACCOUNT_USES: u32 = 16;
/// Whole public manage-usage addresses, matching configuration.md, section 3.
pub const ACCOUNT_MANAGE_USAGE: u32 = 2_048;
/// A whole ASCII HTTPS manage target safe for client rendering (contract.md, 5.1).
/// Configuration additionally checks host labels and ports; it accepts a subset.
#[must_use]
pub fn valid_manage_usage(value: &[u8]) -> bool {
    if value.len() > usize::try_from(ACCOUNT_MANAGE_USAGE).expect("the address bound fits usize") {
        return false;
    }
    let Some(host) = value.strip_prefix(b"https://") else { return false };
    if host.is_empty() || host.first() == Some(&b'/') {
        return false;
    }
    for byte in value {
        if *byte <= b' ' || *byte >= 127 || [b'@', b'?', b'#', b'\\', b'"', b'\'', b'<', b'>'].contains(byte) {
            return false;
        }
    }
    true
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[expect(clippy::struct_excessive_bools, reason = "independent aggregate and list loss facts")]
pub struct OwnerAccounts {
    pub program_hosts: List<super::OwnerProgramHost>,
    pub accounts: List<OwnerAccountLine>,
    pub signing: List<OwnerAccountSignIn>,
    pub last_end: Option<OwnerAccountLastEnd>,
    pub usable: bool,
    pub settled: bool,
    pub cut: bool,
    /// Current observation from the engine selection and host checks.
    pub readiness: List<super::PayerReadiness>,
    /// Whether additional provider/payer rows were omitted.
    pub readiness_cut: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerAccountLine {
    pub number: u32,
    pub place: u32,
    pub uses: Option<List<OwnerAccountUse>>,
    pub plan: bool,
    pub provider: Option<u32>,
    pub provider_name: Option<DocumentText>,
    pub manage_usage: Option<Box<[u8]>>,
    pub display: DocumentText,
    pub holder: OwnerAccountHolder,
    pub sources: List<OwnerAccountSource>,
    pub cut: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerAccountSource {
    pub name: Option<Box<[u8]>>,
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
