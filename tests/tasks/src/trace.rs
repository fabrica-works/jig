//! Complete boundary diagnostics, enabled for focused/failing-seed replay only.
use crate::{coverage::Coverage, referee::Seen};
use jig_core_tasks::Event;
use skein_lib::Time;
use std::sync::OnceLock;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trace {
    enabled: bool,
    pub lines: Vec<String>,
    pub coverage: Coverage,
}
impl Default for Trace {
    fn default() -> Self {
        static ENABLED: OnceLock<bool> = OnceLock::new();
        Self::with_enabled(*ENABLED.get_or_init(|| std::env::var_os("JIG_WORLD_TRACE").is_some()))
    }
}
impl Trace {
    #[must_use]
    pub fn with_enabled(enabled: bool) -> Self {
        Self { enabled, lines: Vec::new(), coverage: Coverage::default() }
    }
    pub fn observed(&mut self, now: Time, seen: &Seen) {
        self.coverage.observed(seen);
        if self.enabled {
            self.lines.push(format!("{:>16} {seen:?}", now.as_nanos()));
        }
    }
    pub fn input(&mut self, now: Time, event: &Event) {
        self.coverage.input(event);
        if self.enabled {
            self.lines.push(format!("{:>16} {event:?}", now.as_nanos()));
        }
    }
}
