//! Complete host boundary trace, formatted only when diagnostics ask for it.
use jig_host::{Event, Request};
use skein_lib::Time;
use std::sync::OnceLock;

#[derive(Debug)]
pub enum Entry {
    Resumes,
    Input(Event),
    Output(Request),
}
#[derive(Default, Debug)]
pub struct Trace {
    entries: Vec<(Time, Entry)>,
    lines: OnceLock<Vec<String>>,
}
impl Trace {
    pub fn log(&mut self, now: Time, entry: Entry) {
        self.lines.take();
        self.entries.push((now, entry));
    }
    pub fn lines(&self) -> &[String] {
        self.lines.get_or_init(|| {
            self.entries
                .iter()
                .map(|(now, entry)| {
                    let detail = match entry {
                        Entry::Resumes => "host resumes".to_owned(),
                        Entry::Input(event) => format!("host <- {event:?}"),
                        Entry::Output(request) => format!("host -> {request:?}"),
                    };
                    format!("{now:?}: {detail}")
                })
                .collect()
        })
    }
}
