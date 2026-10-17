//! The second connector owns resource/kind words and the shared link only.
use alloc::boxed::Box;
use jig_client::Link;
use jig_client::contract::Part;
use skein_lib::List;
#[derive(Debug)]
pub struct Side {
    parts: List<Held>,
    link: Link,
    party: Option<u64>,
    shown: u64,
    text: u32,
    segments: u32,
}
#[derive(Debug)]
struct Held {
    part: Part,
}
impl Side {
    #[must_use]
    pub fn new(capacity: u32, segments: u32, text: u32, _seed: u64) -> Self {
        Self { parts: List::with_capacity(capacity), link: Link::Starting, party: None, shown: 0, text, segments }
    }
    #[must_use]
    pub const fn link(&self) -> Link {
        self.link
    }
    #[must_use]
    pub const fn party(&self) -> Option<u64> {
        self.party
    }
    pub(crate) fn link_is(&mut self, link: Link) {
        self.link = link;
        self.changed();
    }
    pub(crate) fn confirmed(&mut self, party: Option<u64>) {
        self.party = party;
        self.changed();
    }
    pub(crate) fn show(&mut self, part: Part) -> bool {
        match &part {
            Part::Resource { path, .. } => {
                if !jig_client::contract::resource_path_fits(path, self.segments, self.text) {
                    return false;
                }
            }
            Part::Connector { .. }
            | Part::Kind { .. }
            | Part::Panel { .. }
            | Part::Result { .. }
            | Part::Effect { .. }
            | Part::OwnerEffect { .. }
            | Part::Outbox { .. }
            | Part::Hold { .. }
            | Part::Project { .. }
            | Part::Deployment { .. } => {}
        }
        for held in &self.parts {
            if held.part == part {
                return true;
            }
        }
        if self.parts.push(Held { part }).is_err() {
            return false;
        }
        self.changed();
        true
    }
    pub(crate) fn left(&mut self) {
        self.parts.clear();
        self.changed();
    }
    fn changed(&mut self) {
        self.shown = self.shown.checked_add(1).expect("shown count fits");
    }
    #[must_use]
    pub const fn shown(&self) -> u64 {
        self.shown
    }
    #[must_use]
    pub fn contains(&self, part: &Part) -> bool {
        for held in &self.parts {
            if &held.part == part {
                return true;
            }
        }
        false
    }
}
#[must_use]
pub fn worst_case(parts: u32, segments: u32, text: u32) -> Option<u64> {
    List::<Held>::worst_case(parts)?.checked_mul(2)?.checked_add(u64::from(parts).checked_mul(
        List::<Box<[u8]>>::worst_case(segments)?.checked_add(u64::from(segments).checked_mul(u64::from(text))?)?,
    )?)
}
