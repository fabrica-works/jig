//! Whole application boundary traces, enabled for diagnostic replay only.
use std::{fmt::Arguments, ops::Deref, sync::OnceLock};

/// Off in sweeps; `JIG_WORLD_TRACE` enables complete debug diagnostics on replay.
#[derive(Debug)]
pub struct Trace {
    enabled: bool,
    lines: Vec<String>,
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
        Self { enabled, lines: Vec::new() }
    }
    pub fn log(&mut self, message: Arguments<'_>) {
        if self.enabled {
            self.lines.push(message.to_string());
        }
    }
}
impl Deref for Trace {
    type Target = [String];
    fn deref(&self) -> &Self::Target {
        &self.lines
    }
}
