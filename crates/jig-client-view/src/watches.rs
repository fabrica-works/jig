//! Live view handles, held documents and their closing obligations.
//! The client owns reopening and silence (client/domain/client.md, 8.1).
//! A Behind event keeps the held document until a live delivery replaces it.
//!
//! | State | Event | Next | Emits |
//! |---|---|---|---|
//! | opening/live | opened/alive | unchanged | none |
//! | opening/live | delivery | live | owner receives whole document |
//! | opening/live | behind | unchanged | page marked behind |
//! | any active | leave | closing | close |
//! | closing | terminal | closed | retire |
//! | any | terminal | closed | retire |
use jig_client::contract::Subject;

/// The owner that receives a watch document.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Owner {
    /// The frame, which alone judges the link.
    Frame,
    /// The page generation that opened it.
    Page { generation: u32 },
    /// Jig objects held for an application page.
    Application { generation: u32 },
    /// Jig sections held for an application page.
    Sections { generation: u32 },
}

/// One watch's explicit lifecycle, including the old attempt's terminal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Following {
    /// Asked to open, awaiting its first document.
    Opening,
    /// Receiving whole current documents.
    Live,
    /// Closed by its owner, awaiting its terminal.
    Closing,
    /// Its terminal arrived; ready to reclaim.
    Closed,
}

/// One bounded live subject, held until its closing terminal.
#[derive(Debug)]
pub struct Watch {
    pub subject: Subject,
    pub owner: Owner,
    pub state: Following,
    pub behind: bool,
}
