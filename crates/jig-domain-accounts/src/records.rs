//! What an account is and what the store keeps of it (accounts.md, 3.1 to 3.5; engine.md, 5.4).
//! An account and each of its sources are numbered by one counter that never gives a number
//! twice; a source's number is the number its grants and its runs' notices carry.
use alloc::boxed::Box;
use skein_lib::Wall;

/// The longest subject kept: an ID token's `sub` is at most 255 ASCII characters.
pub const SUBJECT: u32 = 255;
/// The longest display name a holder gives an account.
pub const DISPLAY: u32 = 64;
/// The longest program source's name (accounts.md, 3.4).
pub const NAME: u32 = 32;

/// Maximum explicit payers of an account (accounts.md, 4.2).
pub const USES: u32 = 16;

/// Whose work an account pays for (accounts.md, 4.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Use {
    Party(u64),
    Project(u32),
}

/// Who holds an account (accounts.md, 3.2): the deployment, or a party by its number.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Holder {
    Deployment,
    Party(u64),
}

/// A source's kind (accounts.md, 3.4). A program's source keeps its name beside its kind.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    SignIn,
    Key,
    Program,
    None,
}

/// What runs' agents noticed of a source, kept with a wall reset so that it means the same
/// after a restart (engine.md, 5.4). The component says the rest of a source's state again.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Held {
    Limited { reset: Option<Wall> },
    Refused,
    Moving,
}

/// A record of the accounts' family: the counter's one row, an account, or one of its sources.
/// An account's uses name exactly its payers, or leave the default (accounts.md, 4.2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Stored {
    /// The number the counter gives next; no number below it is given again.
    Next { number: u32 },
    /// A lifetime reservation, retained after removal; also names unresolved host cleanup.
    Issued { number: u32, holder: Holder, name: Box<[u8]>, signed_out: bool },
    /// An empty subject is none: no signed-in source gave one.
    Account {
        number: u32,
        provider: u32,
        holder: Holder,
        place: u32,
        uses: Option<Box<[Use]>>,
        subject: Box<[u8]>,
        display: Box<[u8]>,
    },
    /// A source of an account at its place in the holder's order; the name is a program's,
    /// empty for every other kind.
    Source { account: u32, number: u32, place: u32, kind: Kind, name: Box<[u8]>, held: Option<Held> },
}

/// A record's key in the store: the counter's one row, an account, and an account's source.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Key {
    Next,
    Account { number: u32 },
    Source { account: u32, number: u32 },
    Issued { number: u32 },
}

impl Stored {
    #[must_use]
    pub const fn key(&self) -> Key {
        match self {
            Stored::Next { .. } => Key::Next,
            Stored::Issued { number, .. } => Key::Issued { number: *number },
            Stored::Account { number, .. } => Key::Account { number: *number },
            Stored::Source { account, number, .. } => Key::Source { account: *account, number: *number },
        }
    }
}

/// At most `bound` bytes.
pub(crate) fn within(bytes: &[u8], bound: u32) -> bool {
    match u32::try_from(bytes.len()) {
        Ok(length) => length <= bound,
        Err(_) => false,
    }
}

/// A program source's name: 1 to 32 characters of `a-z`, `0-9` and `-` (accounts.md, 3.4).
pub(crate) fn program_name(name: &[u8]) -> bool {
    crate::SourceName::new(name).is_some()
}

/// Explicit uses are bounded and contain no repeated payer.
pub(crate) fn valid_uses(uses: Option<&[Use]>) -> bool {
    let Some(uses) = uses else { return true };
    if uses.len() > usize::try_from(USES).expect("uses bound fits usize") {
        return false;
    }
    for (index, payer) in uses.iter().enumerate() {
        for other in uses.iter().take(index) {
            if payer == other {
                return false;
            }
        }
    }
    true
}
