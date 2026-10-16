//! Which write last carried each request record (client/domain/client.md, 11 and 16).
use skein_lib::{Map, RequestKey};
#[derive(Debug)]
pub(super) struct Store {
    writes: u64,
    answers: u64,
    last: Map<RequestKey, u64>,
}
impl Store {
    pub(super) fn new(requests: u32) -> Self {
        Self { writes: 0, answers: 0, last: Map::with_capacity(requests) }
    }
    pub(super) fn save(&mut self, key: RequestKey) {
        self.last.insert(key, self.writes.checked_add(1).expect("write number fits")).expect("bounded requests");
    }
    pub(super) fn erase(&mut self, key: RequestKey) {
        self.last.remove(&key);
    }
    pub(super) fn write(&mut self) {
        self.writes = self.writes.checked_add(1).expect("write number fits");
    }
    pub(super) fn answer(&mut self) -> Option<u64> {
        if self.answers == self.writes {
            return None;
        }
        self.answers = self.answers.checked_add(1).expect("issued write");
        Some(self.answers)
    }
    pub(super) fn pending(&self) -> bool {
        !self.last.is_empty()
    }
    pub(super) fn take(&mut self, write: u64) -> Option<RequestKey> {
        let mut found = None;
        for (key, last) in &self.last {
            if *last == write {
                found = Some(*key);
                break;
            }
        }
        let key = found?;
        self.last.remove(&key);
        Some(key)
    }
}
