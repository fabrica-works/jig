//! Live subjects, their attempt obligations and timers; client/domain/client.md, 8.1.
//! A terminal permits reopening; a silence first closes the old attempt and waits
//! for its terminal. Delivery values replace one another, with no backlog.
//!
//! | State | Event | Next | Emits |
//! |---|---|---|---|
//! | opening/live | opened/alive | unchanged | heartbeat armed |
//! | opening/live | delivery | live | owner receives whole document |
//! | opening/live | silence | reopening, waiting terminal | close |
//! | opening/live | dropped/stopping/busy | reopening, backoff | retry timer |
//! | reopening | retry after terminal | opening | open |
//! | any active | leave | closing | close if an attempt is live |
//! | closing | terminal | closed | retire |
//! | any | unknown/not allowed/subject ended | closed | retire |
use crate::contract::Subject;

/// The owner that receives a watch document.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Owner {
    /// The frame, which alone judges the link.
    Frame,
    /// The page generation that opened it.
    Page { generation: u32 },
    /// Jig objects held for an application page.
    Application { generation: u32 },
}

/// One watch's explicit lifecycle, including the old attempt's terminal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Following {
    /// Asked to open, awaiting its first document.
    Opening { attempt: u32 },
    /// Receiving whole current documents.
    Live,
    /// Dropped, awaiting an old terminal or a backoff.
    Reopening { attempt: u32, waiting: bool },
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
