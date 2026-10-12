//! Durable indexes for parties' reads (protocol/store.md, section 7.4).
use skein_lib::Wall;

/// A number ordered newest first by its stored complement.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Newest(u64);
impl Newest {
    #[must_use]
    pub const fn of(value: u64) -> Self {
        Self(!value)
    }
    #[must_use]
    pub const fn value(self) -> u64 {
        !self.0
    }
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

/// A result sorts before a reply of the same task and time.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum UpdateObject {
    Result,
    Reply { message: Newest, activity: Newest },
}

/// One person's result or reply, saved with the record it names.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Update {
    pub party: u64,
    pub at: Wall,
    pub task: u64,
    pub object: UpdateObject,
}
impl Update {
    #[must_use]
    pub const fn key(self) -> crate::Key {
        crate::Key::Update {
            party: self.party,
            at: Newest::of(self.at.as_nanos()),
            task: Newest::of(self.task),
            object: self.object,
        }
    }
}
