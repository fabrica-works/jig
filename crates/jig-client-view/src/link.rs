//! The frame watch alone judges the link; client/domain/client.md, 8.2.
/// The connection state shared with every root child.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Link {
    /// Before the first frame delivery.
    Starting,
    /// The frame watch is live.
    Live,
    /// The frame watch dropped or went silent.
    Offline,
}
