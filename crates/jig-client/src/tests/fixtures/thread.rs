use crate::{
    contract::*,
    thread::{Limits, Thread},
};
use skein_lib::{List, Wall};
pub fn text(bytes: &[u8]) -> Text {
    Text::new(bytes.into(), 100).unwrap()
}
pub fn cursor(bytes: &[u8]) -> Cursor {
    Cursor::new(Box::from(bytes), 1024).expect("bounded opaque cursor")
}
pub fn row(position: u64) -> ActivityLine {
    ActivityLine {
        position,
        at: Wall::EPOCH,
        event: Activity::RunBegan { attempt: 1, how: RunBeginning::Fresh { tail: 3 }, why: text(b"fresh") },
    }
}
pub fn page(positions: &[u64], older: Option<&[u8]>, newer: Option<&[u8]>) -> ActivityPage {
    let mut rows = List::with_capacity(10);
    for position in positions {
        rows.push(row(*position)).unwrap();
    }
    ActivityPage {
        cut: false,
        rows,
        before: match older {
            Some(bytes) => {
                let next = cursor(bytes);
                Some(next)
            }
            None => None,
        },
        after: match newer {
            Some(bytes) => {
                let next = cursor(bytes);
                Some(next)
            }
            None => None,
        },
    }
}
pub fn positions(c: &Thread) -> Box<[u64]> {
    let mut positions = List::with_capacity(c.lines.rows.capacity());
    for row in &c.lines.rows {
        positions.push(row.position).unwrap();
    }
    positions.into_boxed()
}

pub fn model() -> Thread {
    Thread::new(super::SUBJECT_TASK, &Limits { lines: 4, bytes: 65536, text: 100 })
}
