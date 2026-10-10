//! Complete boundary diagnostics, enabled for focused/failing-seed replay only.
use skein_lib::Time;
use std::{fmt::Arguments, sync::OnceLock};

#[derive(Debug)]
pub struct Trace {
    enabled: bool,
    trace: skein_world::domain::Trace,
}
impl Default for Trace {
    fn default() -> Self {
        static ENABLED: OnceLock<bool> = OnceLock::new();
        Self::with_enabled(*ENABLED.get_or_init(|| std::env::var_os("JIG_WORLD_TRACE").is_some()))
    }
}
impl Trace {
    pub fn with_enabled(enabled: bool) -> Self {
        Self { enabled, trace: skein_world::domain::Trace::default() }
    }
    pub fn log(&mut self, now: Time, message: Arguments<'_>) {
        if self.enabled {
            self.trace.log(now, message);
        }
    }
    pub fn lines(&self) -> &[String] {
        self.trace.lines()
    }
}
