//! One grace per loss, stopped by a channel back (protocol/channel.md, 10.1).
use skein_lib::{Duration, Time};
#[derive(Debug)]
pub(crate) struct Grace {
    at: Option<Time>,
}
impl Grace {
    pub(crate) fn new() -> Self {
        Self { at: None }
    }
    pub(crate) fn lost(&mut self, now: Time, span: Duration) {
        if self.at.is_none() {
            self.at = Some(now.saturating_add(span));
        }
    }
    pub(crate) fn clear(&mut self) {
        self.at = None;
    }
    pub(crate) fn next(&self) -> Option<Time> {
        self.at
    }
    pub(crate) fn fire(&mut self, now: Time) -> bool {
        if let Some(at) = self.at
            && now >= at
        {
            self.clear();
            return true;
        }
        false
    }
}
