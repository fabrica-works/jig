//! Complete task inputs and referee evidence, formatted only on demand.
use crate::{coverage::Coverage, referee::Seen};
use jig_core_tasks::Event;
use skein_lib::Time;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    Input(Event),
    Observed(Seen),
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Trace {
    pub entries: Vec<(Time, Entry)>,
    pub coverage: Coverage,
}
impl Trace {
    pub fn observed(&mut self, now: Time, seen: &Seen) {
        self.coverage.observed(seen);
        self.entries.push((now, Entry::Observed(seen.clone())));
    }
    pub fn input(&mut self, now: Time, event: &Event) {
        self.coverage.input(event);
        self.entries.push((now, Entry::Input(event.clone())));
    }
}
