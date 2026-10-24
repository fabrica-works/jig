//! Store requests and terminals (protocol/store.md, sections 3, 7 to 10).
use alloc::boxed::Box;
use jig_domain_core::{self as core, RestoreRange, reads};
use jig_domain_notes as notes;
use skein_io::file;
use skein_io::kernel::Fd;
use skein_lib::{Queue, Secret, Token};

/// What the engine's protocol layer asks of the component. Each request but
/// `Commit` names its asker's token; each has one terminal event.
#[derive(Debug)]
pub enum Request {
    /// Open both stores in the deployment's directory (store.md 10); the
    /// component's own rows and the core's header are checked before
    /// `Opened`. In skein-kv's memory mode, the worlds' component needs none.
    /// `installation` is the identifier the shell drew for this start
    /// (configuration.md 8): an empty store keeps it in its first commit; a
    /// store that holds one answers its own instead.
    Open { owner: Token, root: Root, installation: [u8; 16] },
    /// Commit `number` (store.md 6): its writes, in order, one skein-kv
    /// commit with the same number. Terminal: `Committed` or `Failed`.
    Commit { number: u64, writes: Box<[Write]> },
    /// A page of one restart step's ranges (store.md 7.2), from `after`.
    Restore { owner: Token, range: Restore, after: Option<Cursor>, rows: u32, bytes: u64 },
    /// A page of a child's range (store.md 7.1), from `after`.
    Load { owner: Token, range: Load, after: Option<Cursor>, rows: u32, bytes: u64 },
    /// A party's read (store.md 7.3): its scope, position, direction and bounds.
    Read { owner: Token, range: reads::Range },
    /// One record by key.
    Get { owner: Token, key: Get },
    /// A transcript's turns (store.md 8.3).
    Transcript { owner: Token, read: TranscriptRead, rows: u32, bytes: u64 },
    /// The secrets' store (store.md 9).
    Secrets { owner: Token, request: Secrets },
}

/// One write of a commit (store.md 6).
#[derive(Debug)]
pub enum Write {
    /// Copy a held settings revision to history in this commit.
    CopySettingsRevision(u32),
    /// jig's save: a put of its encoded key and record; a turn's save
    /// becomes a frame in its task's transcript and puts of its places.
    Save(core::Record),
    /// jig's erase.
    Erase(core::Key),
    /// A task's transcript dropped whole (store.md 8.5): its places erased,
    /// its file removed before the commit is submitted.
    DropTranscript { task: u64 },
    /// The application's put: its key's bytes (without the tag) and its value's.
    Put { key: Box<[u8]>, value: Box<[u8]> },
    /// The application's erase.
    Remove { key: Box<[u8]> },
}

/// What a restart step restores.
#[derive(Debug)]
pub enum Restore {
    /// jig's ranges, in order: `RestoreRange::LIVE` for `LoadCore`, the
    /// census for `LoadInbox`. Rows are handed up typed; the census's are
    /// the ended tasks its index rows name.
    Jig(Box<[RestoreRange]>),
    /// A connector's live intervals, relative to the application's tag.
    Application(Box<[skein_kv::Range]>),
}

/// A child's range to load.
#[derive(Debug)]
pub enum Load {
    /// The notes child's entry or a scope's lines.
    Notes(notes::Range),
    /// The application's intervals, relative to its tag.
    Application(Box<[skein_kv::Range]>),
}

/// A record to get.
#[derive(Debug)]
pub enum Get {
    Jig(core::Key),
    /// The application's key, relative to its tag.
    Application(Box<[u8]>),
}

/// A transcript read (store.md 8.3). Offsets are the component's, given
/// and taken back; none reads past what is committed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TranscriptRead {
    /// A resuming run's current window, from the turn the core names as its
    /// start; a window that starts at another turn is `Unread`.
    Window { task: u64, attempt: u64, turn: u32, after: Option<u64> },
    /// One turn, walked to from its attempt's first turn.
    Turn { task: u64, attempt: u64, turn: u32 },
    /// A page of a task's turns from either end; a fresh run's tail is
    /// `backwards: true, after: None`.
    Page { task: u64, backwards: bool, after: Option<u64> },
}

/// Whose secret (store.md 9).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum SecretOwner {
    /// A sign-in's digest, by the sign-in's number.
    SignIn,
    /// A service's credential's digest, by the service's number.
    Service,
    /// The application's own record, under its tag, by its number.
    Application,
    /// A signed-in source's record or an LLM key's value, by the source's number.
    Source,
}

/// The first number of the application's range that jig numbers itself,
/// for the values clients' requests carry (store.md 9); the application
/// numbers its own values below it.
pub const CARRIED: u64 = 1 << 63;

/// A request to the secrets' store (store.md, section 9).
#[derive(Debug)]
pub enum Secrets {
    Keep {
        owner: SecretOwner,
        number: u64,
        value: Secret,
    },
    /// Keep `value` in place of the value whose digest is `replaces`, none
    /// when nothing is held: `Kept` once durable, or `Stale`, nothing
    /// written, when another value is held, durable or on its way.
    Replace {
        owner: SecretOwner,
        number: u64,
        value: Secret,
        replaces: Option<[u8; 32]>,
    },
    /// Atomically move a carried value into its owner's row, conditional on
    /// the target's digest as in `Replace`. Missing carried values are stale.
    Adopt {
        carried: u64,
        owner: SecretOwner,
        number: u64,
        replaces: Option<[u8; 32]>,
    },
    /// Adopt a decoded value, conditional on both the carried file's digest
    /// and the destination's digest. Both rows change in one commit (store.md 9).
    AdoptValue {
        carried: u64,
        holds: [u8; 32],
        owner: SecretOwner,
        number: u64,
        value: Secret,
        replaces: Option<[u8; 32]>,
    },
    /// Keep a value a client's request carried, under the next number of
    /// the application's carried range, from [`CARRIED`] up (store.md 9):
    /// `Carried` once durable, the next number moved in the same commit;
    /// `Stale`, nothing written, when that number already holds a value.
    Carry {
        value: Secret,
    },
    Erase {
        owner: SecretOwner,
        number: u64,
    },
    Get {
        owner: SecretOwner,
        number: u64,
    },
    Load {
        owner: SecretOwner,
        after: Option<u64>,
    },
}

#[derive(Debug)]
pub struct SecretRow {
    pub owner: SecretOwner,
    pub number: u64,
    pub value: Secret,
}

/// Where a page stopped: the interval of the request's list, and the last
/// key it returned (store.md 7.1: the next page starts just past it).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Cursor {
    pub interval: u32,
    pub after: Box<[u8]>,
}

/// One restored or loaded row.
#[derive(Debug)]
#[expect(clippy::large_enum_variant, reason = "bounded jig records travel by value; Row queues price their full size")]
pub enum Row {
    /// jig's record, decoded.
    Jig(core::Record),
    /// The application's row, its key relative to its tag.
    Application { key: Box<[u8]>, value: Box<[u8]> },
}

/// Whose format the format row disagrees on (store.md 5.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Family {
    Settings,
    Component,
    Core,
    Tasks,
    People,
    Notes,
    Accounts,
    Application,
}

/// Why a store refuses to open, saying which (store.md 5.3, 6, 10).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OpenRefusal {
    /// The format row names another version of `family`.
    Format { family: Family, found: u32, wanted: u32 },
    /// The format row names another deployment than the header.
    Deployment,
    /// skein-kv's recovered last commit is not the header's.
    Numbers { store: u64, header: u64 },
    /// A file failed, or a store did not recover.
    Failed,
    /// A store from an older build is left untouched (store.md 2 and 10.1).
    OlderBuild { why: Older },
    /// The deployment's generations refuse (store.md 10.1).
    Generations { why: Generations },
}
/// An older build's refusal whose fix moves the generations (store.md 2), `why`
/// saying which build.
macro_rules! older {
    ($why:literal) => {
        &[
            "",
            concat!(
                " holds a store from an older build, ",
                $why,
                ", and this start has changed nothing in it. Before the first release a store is not translated, so every start refuses this one until it is moved. To start afresh, stop the service, move `active`, `generations/` and `transcripts/` out of ",
            ),
            " together, and start again: nothing is deleted, and the next start makes a new, empty store. The new store has no tasks, sign-ins, accounts or keys, and every client signs in again.",
        ]
    };
}

impl OpenRefusal {
    /// The refusal's text in pieces, in order: the deployment's directory goes
    /// between each two consecutive pieces, wherever store.md has `<deployment>`.
    /// No allocation, no formatting.
    #[must_use]
    pub fn words(&self) -> &'static [&'static str] {
        match self {
            // o/configuring-2's L gives these their words.
            Self::Format { .. } | Self::Deployment | Self::Numbers { .. } | Self::Failed => &[],
            Self::OlderBuild { why: Older::Unrecorded } => older!("made before its families' layouts were recorded"),
            Self::OlderBuild { why: Older::Layout(family) } => match family {
                Family::Settings => older!("whose settings family's layout differs from this build's"),
                Family::Component => older!("whose component family's layout differs from this build's"),
                Family::Core => older!("whose core family's layout differs from this build's"),
                Family::Tasks => older!("whose tasks family's layout differs from this build's"),
                Family::People => older!("whose people family's layout differs from this build's"),
                Family::Notes => older!("whose notes family's layout differs from this build's"),
                Family::Accounts => older!("whose accounts family's layout differs from this build's"),
                Family::Application => older!("whose application family's layout differs from this build's"),
            },
            Self::OlderBuild { why: Older::Directory } => &[
                "",
                " holds a store from an older build, in the layout before generations (`store/`, `secrets/` or `transcripts/` at its top), and this start has changed nothing in it. Before the first release a store is not translated, so every start refuses this one until it is moved. To start afresh, stop the service, move `store/`, `secrets/` and `transcripts/` out of ",
                " together, and start again: nothing is deleted, and the next start makes a new, empty store. The new store has no tasks, sign-ins, accounts or keys, and every client signs in again.",
            ],
            Self::Generations { why: Generations::Busy } => &[
                "Another process has ",
                " open, and this start has changed nothing in it. Stop that process first, then start again.",
            ],
            Self::Generations { why: Generations::Unreadable } => &[
                "The `active` file in ",
                " does not name a generation, and this start has changed nothing in it. Restore ",
                " from a backup, or write `active` again to name a generation under `generations/` that stands, and start again.",
            ],
            Self::Generations { why: Generations::Missing { .. } } => &[
                "The generation `active` names in ",
                " has lost its `store/` or `secrets/`, and this start has changed nothing in it. Restore ",
                " from a backup, or write `active` again to name a generation under `generations/` that stands, and start again.",
            ],
            Self::Generations { why: Generations::Unnamed } => &[
                "",
                " holds generations that no `active` names, and this start has changed nothing in it. Name one again from a backup of ",
                ", or write `active` to name the generation under `generations/` to open, and start again.",
            ],
        }
    }
}
/// Which older build (store.md 5.3 and 10.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Older {
    /// A store with no layouts row, made before they were recorded (store.md 5.3).
    Unrecorded,
    /// A family's fingerprint is not this build's at its version (store.md 5.3).
    Layout(Family),
    /// The layout before generations, with no active file (store.md 2).
    Directory,
}
/// A refusal while finding the deployment's generation (store.md 10.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Generations {
    /// Another open holds the deployment lock.
    Busy,
    /// Active is not decimal digits and one newline within 21 bytes.
    Unreadable,
    /// Generations stand that no active names, beyond a cut first start.
    Unnamed,
    /// The active generation's maps do not both stand as directories.
    Missing { generation: u64 },
}

/// What the component answers.
#[derive(Debug)]
pub enum Event {
    /// Both stores are open and checked; `last` is the last durable commit.
    /// `installation` is the installation's identifier (store.md 4.4): the
    /// kept one, or, in an empty store, the drawn one its first commit keeps.
    Opened {
        owner: Token,
        last: u64,
        installation: [u8; 16],
    },
    OpenRefused {
        owner: Token,
        why: OpenRefusal,
    },
    /// Commit `number` is durable, and every earlier one; `fill` is the
    /// map's bytes (skein-kv's `used`), for the core's high-water mark.
    Committed {
        number: u64,
        fill: u64,
    },
    /// Commit `number` failed or was refused; the engine stops (store.md 6).
    Failed {
        number: u64,
    },
    /// A restore's page; `next` is none once the step's ranges are done.
    Restored {
        owner: Token,
        rows: Box<[Row]>,
        next: Option<Cursor>,
    },
    /// A load's page; `next` is none once the range is done.
    Loaded {
        owner: Token,
        rows: Box<[Row]>,
        next: Option<Cursor>,
    },
    /// A read's page, busy, or gone.
    Read {
        owner: Token,
        result: reads::Loaded,
    },
    Got {
        owner: Token,
        row: Option<Row>,
    },
    /// A transcript's turns; `next` is the offset to continue from.
    Turns {
        owner: Token,
        turns: Box<[core::TurnRecord]>,
        next: Option<u64>,
    },
    /// A transcript read failed: a frame's checksum, a window at another
    /// turn, a dropped transcript (store.md 8.3, 8.5).
    Unread {
        owner: Token,
    },
    Kept {
        owner: Token,
    },
    /// A carried value kept, durable, under the application's `number`.
    Carried {
        owner: Token,
        number: u64,
    },
    /// A replace that named another value than the one held, or a carry
    /// whose next number holds one.
    Stale {
        owner: Token,
    },
    Erased {
        owner: Token,
    },
    Secret {
        owner: Token,
        value: Option<Secret>,
    },
    SecretsLoaded {
        owner: Token,
        rows: Box<[SecretRow]>,
        next: Option<u64>,
    },
    /// A stored row did not decode: the request's terminal; the engine
    /// stops (store.md 7.1).
    Corrupt {
        owner: Token,
    },
}

/// A directory adopted by the service (io.md, section 9).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Root {
    pub token: Token,
    pub fd: Fd,
}

/// Independent file drivers (store.md, section 13).
#[derive(Debug)]
pub struct Below {
    pub store: Queue<file::Request>,
    pub secrets: Queue<file::Request>,
    pub transcripts: Queue<file::Request>,
}

/// A file terminal and its descriptor, read by the service alone.
#[derive(Debug)]
pub struct Arrived {
    pub event: file::Event,
    pub fd: Option<Fd>,
}

#[derive(Debug)]
pub enum Io {
    Store(Arrived),
    Secrets(Arrived),
    Transcripts(Arrived),
}
