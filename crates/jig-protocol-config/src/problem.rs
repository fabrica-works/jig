//! Bounded diagnostics at paths from the whole document root (configuration.md, 2).
use alloc::boxed::Box;
use skein_lib::{List, bytes};
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Segment {
    Member(Box<[u8]>),
    Index(u32),
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Path {
    pub segments: List<Segment>,
}
impl Path {
    #[must_use]
    pub fn root() -> Self {
        Self { segments: List::with_capacity(0) }
    }
    #[must_use]
    pub fn child(&self, segment: Segment) -> Self {
        let mut segments = List::with_capacity(self.segments.len().checked_add(1).expect("document path fits u32"));
        for item in &self.segments {
            segments.push(item.clone()).expect("path capacity includes ancestors");
        }
        segments.push(segment).expect("path capacity includes child");
        Self { segments }
    }
    #[must_use]
    pub fn member(&self, name: &[u8]) -> Self {
        self.child(Segment::Member(bytes::copy_of(name)))
    }
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Problem {
    pub path: Path,
    pub what: What,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum What {
    Json,
    Unknown,
    Missing,
    Duplicate,
    Kind,
    Bound,
    Form,
    Unique,
    Refers,
    Plaintext,
    Confidential,
    NoProvider,
    Digest,
    Reserve {
        charter: u32,
    },
    Budget {
        charter: u32,
    },
    Grace {
        template: u32,
    },
    Operations {
        operations: u32,
        delegates: u32,
        goals: u32,
    },
    Message,
    Silence,
    StopBound,
    Memory {
        over: u64,
    },
    Override,
    Unreadable,
    /// A relative path that is empty or has a `..` component (configuration.md, 10).
    Relative,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Problems {
    pub problems: List<Problem>,
    pub past: u32,
}
pub const PROBLEMS: u32 = 32;
impl Problems {
    #[must_use]
    pub fn empty() -> Self {
        Self { problems: List::with_capacity(PROBLEMS), past: 0 }
    }
    pub fn push(&mut self, path: Path, what: What) {
        if self.problems.push(Problem { path, what }).is_err() {
            self.past = self.past.saturating_add(1);
        }
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.problems.is_empty() && self.past == 0
    }
}
